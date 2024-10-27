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

#[wasm_bindgen(start)]
fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    console!("Running game...".to_string());
    
    init_world();
    World::init();
}