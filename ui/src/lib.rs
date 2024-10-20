#![allow(clippy::inherent_to_string)]

use gloo::utils::window;
use wasm_bindgen::JsValue;
use web_sys::js_sys::Reflect;

pub const DEBUG: bool = false;

pub fn get_debug_window_props() -> Result<(JsValue, JsValue), JsValue> {
    let window = window();
    let starlight = Reflect::get(&window, &JsValue::from_str("starlight"))?;
    let moonshine = Reflect::get(&window, &JsValue::from_str("moonshine"))?;

    Ok((starlight, moonshine))
}

pub mod core;
pub mod color;
pub mod canvas2d;
pub mod label;
pub mod button;