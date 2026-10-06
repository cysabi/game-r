use bevy::{
    app::{Plugins, PluginsState},
    prelude::*,
};
use rcade_plugin_input_classic::ClassicController;
use rcade_plugin_input_spinners::{P1, P2, SpinnerState};

use crate::hook::{self, RcadePluginExt, get_offscreen_canvas};

#[derive(Default, Clone, Copy)]
pub struct ButtonState {
    pub pressed: bool,
    pub just_pressed: bool,
}

#[derive(Default, Clone, Copy)]
pub struct PlayerController {
    pub up: ButtonState,
    pub down: ButtonState,
    pub left: ButtonState,
    pub right: ButtonState,
    pub a: ButtonState,
    pub b: ButtonState,
    pub spinner: SpinnerState,
}

#[derive(Resource, Default, Clone, Copy)]
pub struct Controller {
    pub player_a: PlayerController,
    pub player_b: PlayerController,
}

pub struct RcadeApp {
    app: App,
}

impl RcadeApp {
    pub async fn new() -> Self {
        let mut app = App::new();

        let canvas = get_offscreen_canvas().unwrap();

        let controller = ClassicController::acquire().await.unwrap();

        app.add_plugins(
            DefaultPlugins
                .with_rcade(canvas.clone())
                .await
                .set(ImagePlugin::default_nearest()),
        )
        .insert_resource(Controller::default())
        .insert_non_send_resource(controller)
        .insert_non_send_resource(canvas)
        .add_systems(PreStartup, hook::setup_added_window)
        .add_systems(PreUpdate, poll_controller);

        RcadeApp { app }
    }

    pub fn add_plugins<M>(mut self, plugins: impl Plugins<M>) -> Self {
        self.app.add_plugins(plugins);
        self
    }

    pub fn app(&mut self) -> &mut App {
        &mut self.app
    }

    pub async fn run(mut self) -> ! {
        loop {
            match self.app.plugins_state() {
                PluginsState::Adding => {}
                PluginsState::Ready => {
                    self.app.finish();
                    self.app.cleanup();
                }
                PluginsState::Finished => self.app.cleanup(),
                PluginsState::Cleaned => self.app.update(),
            }
            gloo_timers::future::sleep(std::time::Duration::from_nanos(0)).await;
        }
    }
}

fn poll_controller(
    device: NonSend<ClassicController>,
    mut controller: ResMut<Controller>,
) {
    let state = device.state();

    let last_controller = controller.clone();

    controller.player_a.up.just_pressed = !last_controller.player_a.up.pressed && state.player1_up;
    controller.player_a.up.pressed = state.player1_up;
    controller.player_a.down.just_pressed =
        !last_controller.player_a.down.pressed && state.player1_down;
    controller.player_a.down.pressed = state.player1_down;
    controller.player_a.left.just_pressed =
        !last_controller.player_a.left.pressed && state.player1_left;
    controller.player_a.left.pressed = state.player1_left;
    controller.player_a.right.just_pressed =
        !last_controller.player_a.right.pressed && state.player1_right;
    controller.player_a.right.pressed = state.player1_right;
    controller.player_a.a.just_pressed = !last_controller.player_a.a.pressed && state.player1_a;
    controller.player_a.a.pressed = state.player1_a;
    controller.player_a.b.just_pressed = !last_controller.player_a.b.pressed && state.player1_b;
    controller.player_a.b.pressed = state.player1_b;
    controller.player_a.spinner = P1.read();

    controller.player_b.up.just_pressed = !last_controller.player_b.up.pressed && state.player2_up;
    controller.player_b.up.pressed = state.player2_up;
    controller.player_b.down.just_pressed =
        !last_controller.player_b.down.pressed && state.player2_down;
    controller.player_b.down.pressed = state.player2_down;
    controller.player_b.left.just_pressed =
        !last_controller.player_b.left.pressed && state.player2_left;
    controller.player_b.left.pressed = state.player2_left;
    controller.player_b.right.just_pressed =
        !last_controller.player_b.right.pressed && state.player2_right;
    controller.player_b.right.pressed = state.player2_right;
    controller.player_b.a.just_pressed = !last_controller.player_b.a.pressed && state.player2_a;
    controller.player_b.a.pressed = state.player2_a;
    controller.player_b.b.just_pressed = !last_controller.player_b.b.pressed && state.player2_b;
    controller.player_b.b.pressed = state.player2_b;
    controller.player_b.spinner = P2.read();
}
