#![feature(mapped_lock_guards)]
#![allow(unused)]

extern crate console_error_panic_hook;

mod connection;
mod rendering;
mod simulation;
mod world;

use std::panic;
use gloo::console::console;
use web_sys::wasm_bindgen::{self, prelude::*};
use world::{get_world, init_world, World};

#[macro_export]
macro_rules! storage_get {
    ($item: expr) => {
        window().local_storage().unwrap().unwrap().get($item).unwrap_or(None)  
    };
}

#[wasm_bindgen(start)]
fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    init_world();

    console!("Running game...".to_string());
    World::init();
}