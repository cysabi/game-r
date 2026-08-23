pub mod hook;

use crate::hook::{RcadePluginExt, get_offscreen_canvas};
use bevy::{app::PluginsState, prelude::*};
use rcade_plugin_input_classic::ClassicController;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn start() {
    console_error_panic_hook::set_once();

    let mut app = Game::new().await;

    loop {
        app.update_plugins();
    }
}

#[wasm_bindgen]
pub struct Game {
    app: App,
}

impl Game {
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
        .insert_non_send_resource(controller)
        .insert_non_send_resource(canvas)
        .add_systems(PreStartup, hook::setup_added_window);

        Game { app }
    }

    pub fn update_plugins(&mut self) {
        if self.app.plugins_state() != PluginsState::Cleaned {
            if self.app.plugins_state() == PluginsState::Ready {
                self.app.finish();

                self.app.cleanup();
            }
        } else {
            self.app.update();
        }
    }
}
