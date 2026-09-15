## About RCade

This game is built for [RCade](https://rcade.recurse.com), a custom arcade cabinet at The Recurse Center. Learn more about the project at [github.com/fcjr/RCade](https://github.com/fcjr/RCade).

## Prerequisites

- [Rust](https://rustup.rs/)
- [Trunk](https://trunkrs.dev/) - `cargo install trunk`
- wasm32 target - `rustup target add wasm32-unknown-unknown`

## Getting Started

Start the development server and the RCade dev shell together:

```bash
cargo dev
```

This runs `trunk serve` (compiles the Rust code to WebAssembly with hot
reloading) and, once it is listening, `npx rcade@latest dev http://localhost:8080`.
Ctrl+C stops both. Extra arguments go to trunk (`cargo dev --release`), and
`PORT=9000 cargo dev` changes the port.

To run the pieces by hand instead:

```bash
trunk serve
npx rcade@latest dev http://localhost:8080
```

## Building

```bash
trunk build --release
```

Output goes to `dist/` and is ready for deployment.

## Project Structure

```
├── src/
│   └── lib.rs        # Game entry point
├── index.html        # HTML entry
└── Cargo.toml        # Rust dependencies
```

## WebAssembly Bindings

This template uses `wasm-bindgen` and `web-sys` for DOM interaction:

```rust
use wasm_bindgen::prelude::*;
use web_sys::window;

#[wasm_bindgen(start)]
pub fn main() {
    let document = window().unwrap().document().unwrap();
    let body = document.body().unwrap();
    body.set_inner_html("<h1>Hello!</h1>");
}
```

## Arcade Controls

### Development Keyboard Controls

When developing locally, keyboard inputs are mapped to arcade controls:

**Classic Controls (`@rcade/plugin-input-classic`)**

| Player   | Action           | Key |
|----------|------------------|-----|
| Player 1 | UP               | W   |
| Player 1 | DOWN             | S   |
| Player 1 | LEFT             | A   |
| Player 1 | RIGHT            | D   |
| Player 1 | A Button         | F   |
| Player 1 | B Button         | G   |
| Player 2 | UP               | I   |
| Player 2 | DOWN             | K   |
| Player 2 | LEFT             | J   |
| Player 2 | RIGHT            | L   |
| Player 2 | A Button         | ;   |
| Player 2 | B Button         | '   |
| System   | One Player Start | 1   |
| System   | Two Player Start | 2   |

**Spinner Controls (`@rcade/plugin-input-spinners`)**

| Player   | Action        | Key |
|----------|---------------|-----|
| Player 1 | Spinner Left  | C   |
| Player 1 | Spinner Right | V   |
| Player 2 | Spinner Left  | .   |
| Player 2 | Spinner Right | /   |

Spinners repeat at ~60Hz while held.

To add spinner support, add to your `Cargo.toml`:
```toml
rcade-plugin-input-spinners = "0.1.0"
```

## Deployment

First, create a new repository on GitHub:

1. Go to [github.com/new](https://github.com/new)
2. Create a new repository (can be public or private)
3. **Don't** initialize it with a README, .gitignore, or license

Then connect your local project and push:

```bash
git remote add origin git@github.com:YOUR_USERNAME/YOUR_REPO.git
git push -u origin main
```

The included GitHub Actions workflow will automatically deploy to RCade.

---

Made with <3 at [The Recurse Center](https://recurse.com)
