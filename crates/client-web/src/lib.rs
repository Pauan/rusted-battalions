use wasm_bindgen::prelude::*;

mod renderer;
mod app;

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    console_log::init_with_level(log::Level::Warn).unwrap();

    let app = app::App::new();
    dominator::append_dom(&dominator::body(), app::App::render(&app));

    Ok(())
}
