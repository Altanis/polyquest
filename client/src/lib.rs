#![feature(mapped_lock_guards)]
#![allow(unused)]

extern crate console_error_panic_hook;

mod connection;
mod rendering;
mod simulation;
mod world;

use std::panic;
use gloo::console::console;
use gloo_utils::window;
use web_sys::{js_sys::Reflect, wasm_bindgen::{self, prelude::*}};
use world::{get_world, init_world, World};

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

pub fn get_debug_window_props() -> Result<(JsValue, JsValue), JsValue> {
    let window = window();
    let starlight = Reflect::get(&window, &JsValue::from_str("starlight"))?;
    let moonshine = Reflect::get(&window, &JsValue::from_str("moonshine"))?;

    Ok((starlight, moonshine))
}

#[wasm_bindgen(start)]
fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    init_world();

    console!("Running game...".to_string());
    World::init();
}