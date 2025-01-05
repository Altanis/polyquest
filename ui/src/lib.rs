#![allow(clippy::inherent_to_string)]
#![allow(clippy::new_without_default)]
#![allow(clippy::neg_cmp_op_on_partial_ord)]
#![feature(let_chains)]

use gloo::utils::window;
use wasm_bindgen::JsValue;
use web_sys::js_sys::Reflect;

pub const DEBUG: bool = false;

#[macro_export]
macro_rules! storage_get {
    ($item: expr) => {
        window().local_storage().unwrap().unwrap().get($item).unwrap_or(None)  
    };
}

#[macro_export]
macro_rules! storage_set {
    ($item: expr, $value: expr) => {
        let _ = window().local_storage().unwrap().unwrap().set($item, $value);
    };
}

#[macro_export]
macro_rules! get_element_by_id_and_cast {
    ($id:expr, $type:ty) => {{
        document()
            .get_element_by_id($id)
            .expect("element with given ID should exist")
            .dyn_into::<$type>()
            .expect("element should be of the expected type")
    }};
}

pub fn get_debug_window_props() -> Result<(JsValue, JsValue), JsValue> {
    let window = window();
    let starlight = Reflect::get(&window, &JsValue::from_str("starlight"))?;
    let moonshine = Reflect::get(&window, &JsValue::from_str("moonshine"))?;

    Ok((starlight, moonshine))
}

pub mod core;
pub mod canvas2d;
pub mod gl;
pub mod utils;
pub mod elements;