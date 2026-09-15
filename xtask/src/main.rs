//! `cargo dev` — run `trunk serve` and the RCade dev shell together.
//!
//! Equivalent to running these two commands in separate terminals:
//!
//! ```text
//! trunk serve --port 8080
//! npx rcade@latest dev http://localhost:8080
//! ```
//!
//! The rcade shell is started only once trunk is actually listening, so it
//! doesn't open onto a connection-refused page while the first (slow) bevy
//! build is still running. When either process exits, or you press Ctrl+C,
//! both process trees are torn down.
//!
//! Extra arguments are forwarded to `trunk serve`, e.g. `cargo dev --release`.
//! Set `PORT` to use a port other than 8080.

use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::process::{Child, Command, ExitCode, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

static INTERRUPTED: AtomicBool = AtomicBool::new(false);

fn main() -> ExitCode {
    install_ctrl_c_handler();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);
    let url = format!("http://localhost:{port}");
    let trunk_args: Vec<String> = std::env::args().skip(1).collect();

    // Run from the workspace root so trunk picks up Trunk.toml regardless of
    // which directory `cargo dev` was invoked from.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the workspace root");

    let mut trunk = match Command::new("trunk")
        .current_dir(root)
        .arg("serve")
        .arg("--port")
        .arg(port.to_string())
        .args(&trunk_args)
        .stdin(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            eprintln!("[dev] failed to start `trunk serve`: {err}");
            eprintln!("[dev] install it with `cargo install trunk`");
            return ExitCode::FAILURE;
        }
    };

    // Wait for trunk to accept connections before launching the rcade shell.
    let addr = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
    eprintln!("[dev] waiting for trunk on {url} ...");
    loop {
        if interrupted() {
            eprintln!("[dev] interrupted; stopping trunk");
            kill_tree(&mut trunk);
            return ExitCode::SUCCESS;
        }
        if let Some(status) = poll(&mut trunk) {
            eprintln!("[dev] trunk exited ({status}) before serving; giving up");
            return ExitCode::FAILURE;
        }
        if TcpStream::connect_timeout(&addr, Duration::from_millis(500)).is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    eprintln!("[dev] trunk is up, starting rcade shell");

    // `npx` is a .cmd shim on Windows; std handles the cmd.exe indirection
    // when given the full file name.
    let npx = if cfg!(windows) { "npx.cmd" } else { "npx" };
    let mut rcade = match Command::new(npx)
        .current_dir(root)
        .args(["rcade@latest", "dev", &url])
        .stdin(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(err) => {
            eprintln!("[dev] failed to start `npx rcade@latest dev`: {err}");
            eprintln!("[dev] is Node.js installed and on PATH?");
            kill_tree(&mut trunk);
            return ExitCode::FAILURE;
        }
    };

    // Babysit both; whichever dies first takes the other down with it.
    loop {
        if interrupted() {
            eprintln!("[dev] interrupted; stopping trunk and rcade");
            kill_tree(&mut rcade);
            kill_tree(&mut trunk);
            return ExitCode::SUCCESS;
        }
        if let Some(status) = poll(&mut trunk) {
            eprintln!("[dev] trunk exited ({status}); stopping rcade");
            kill_tree(&mut rcade);
            return exit_code(status);
        }
        if let Some(status) = poll(&mut rcade) {
            eprintln!("[dev] rcade exited ({status}); stopping trunk");
            kill_tree(&mut trunk);
            return exit_code(status);
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// Non-blocking check for whether a child has exited.
fn poll(child: &mut Child) -> Option<ExitStatus> {
    match child.try_wait() {
        Ok(status) => status,
        Err(err) => {
            eprintln!("[dev] error waiting on child process: {err}");
            None
        }
    }
}

fn exit_code(status: ExitStatus) -> ExitCode {
    if status.success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Kill a child and everything it spawned.
///
/// `Child::kill` only hits the direct child. For `npx` that is a `cmd.exe`
/// shim (Windows) or a node wrapper (unix); the actual rcade cabinet lives a
/// couple of levels further down and would be orphaned.
fn kill_tree(child: &mut Child) {
    if child.try_wait().ok().flatten().is_some() {
        return; // already gone
    }
    let pid = child.id().to_string();
    if cfg!(windows) {
        let _ = Command::new("taskkill")
            .args(["/T", "/F", "/PID", &pid])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    } else {
        // Best effort: terminate direct children first, then the child itself.
        let _ = Command::new("pkill")
            .args(["-TERM", "-P", &pid])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
    let _ = child.kill();
    let _ = child.wait();
}

// ---------------------------------------------------------------------------
// Ctrl+C handling (no external crates)
//
// We handle Ctrl+C ourselves rather than letting the default handler kill this
// process, so that we get a chance to tear down both process trees. On Windows
// this also suppresses cmd.exe's "Terminate batch job (Y/N)?" prompt, which
// would otherwise appear because npx.cmd is a batch file.
// ---------------------------------------------------------------------------

#[cfg(windows)]
fn install_ctrl_c_handler() {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn SetConsoleCtrlHandler(
            handler: Option<unsafe extern "system" fn(ctrl_type: u32) -> i32>,
            add: i32,
        ) -> i32;
    }
    unsafe extern "system" fn handler(_ctrl_type: u32) -> i32 {
        INTERRUPTED.store(true, Ordering::SeqCst);
        1 // handled; don't terminate the process
    }
    // SAFETY: plain Win32 call with a valid `extern "system"` callback.
    unsafe {
        SetConsoleCtrlHandler(Some(handler), 1);
    }
}

#[cfg(unix)]
fn install_ctrl_c_handler() {
    const SIGINT: i32 = 2;
    const SIGTERM: i32 = 15;
    unsafe extern "C" {
        fn signal(signum: i32, handler: usize) -> usize;
    }
    extern "C" fn handler(_signum: i32) {
        // Only an atomic store: async-signal-safe.
        INTERRUPTED.store(true, Ordering::SeqCst);
    }
    // SAFETY: installing an async-signal-safe handler via libc `signal`.
    unsafe {
        signal(SIGINT, handler as usize);
        signal(SIGTERM, handler as usize);
    }
}

#[cfg(not(any(windows, unix)))]
fn install_ctrl_c_handler() {}
