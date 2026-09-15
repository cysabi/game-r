pub mod components;
pub mod game;
pub mod hook;
pub mod map;
pub mod rcade;

use crate::{game::Game, rcade::RcadeApp};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn start() {
    console_error_panic_hook::set_once();

    RcadeApp::new().await.add_plugins(Game).run().await;
}
