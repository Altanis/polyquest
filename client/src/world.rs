use std::{borrow::BorrowMut, cell::RefCell, ops::DerefMut, rc::Rc, sync::{LazyLock, Mutex, MutexGuard}};

use gloo_utils::window;
use web_sys::{wasm_bindgen::{prelude::Closure, JsCast, JsValue}, BeforeUnloadEvent, Event, KeyboardEvent, MouseEvent};

use crate::{core::rendering::{events::{self, on_resize, EventType}, renderer::Renderer}, register_event};

pub type LockedWorld = MutexGuard<'static, Option<Box<World>>>;

pub static mut WORLD: Mutex<Option<Box<World>>> = Mutex::new(None);

pub fn init_world() {
    unsafe {
        *WORLD.borrow_mut() = Some(Box::new(World::new())).into();
    }
}

pub fn get_world() -> &'static mut World {
    unsafe {
        let mut world_guard = WORLD.lock().unwrap();
        if let Some(ref mut world) = *world_guard {
            std::mem::transmute::<&mut World, &'static mut World>(world.deref_mut())
        } else {
            panic!("world accessed without it being set");
        }
    }
}

pub struct World {
    pub renderer: Renderer
}

impl World {
    pub fn new() -> World {
        World {
            renderer: Renderer::new()
        }
    }

    pub fn setup(&self) {
        register_event!("beforeunload");
        register_event!("contextmenu");
        register_event!("resize");
        register_event!("mousedown");
        register_event!("mouseup");
        register_event!("mousemove");
        register_event!("keydown");
        register_event!("keyup");

        self.renderer.canvas2d.resize(&window());
    }

    pub fn init() {
        {
            let mut world = get_world();
            world.setup();
        }

        {
            let closure = Closure::wrap(Box::new(move |ts: f64| {
                Renderer::tick(get_world(), ts);
            }) as Box<dyn FnMut(_)>);
            
            let _ = window()
                .request_animation_frame(closure.as_ref().unchecked_ref());
            
            closure.forget();
        }        
    }
}