#![feature(let_chains)]
#![feature(mapped_lock_guards)]
#![feature(const_vec_string_slice)]
#![allow(unused)]
#![allow(clippy::neg_cmp_op_on_partial_ord)]

extern crate console_error_panic_hook;

mod connection;
mod rendering;
mod simulation;
mod world;
mod game;

use std::panic;
use gloo::console::console;
use gloo_utils::window;
use web_sys::{js_sys::Reflect, wasm_bindgen::{self, prelude::*}};
use world::{get_world, init_world, World};
use web_sys::js_sys;

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

pub const SHADERS_ENABLED: bool = false;

#[wasm_bindgen(start)]
fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    init_world();

    console!("Running game...".to_string());
    World::init();
}