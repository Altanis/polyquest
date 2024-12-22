use std::{borrow::BorrowMut, cell::RefCell, ops::DerefMut, rc::Rc, sync::{LazyLock, MappedMutexGuard, Mutex, MutexGuard}};
use send_wrapper::SendWrapper;
use gloo_utils::window;
use ui::utils::sound::{Sound, SoundHolder};
use web_sys::{wasm_bindgen::{prelude::Closure, JsCast, JsValue}, BeforeUnloadEvent, Event, KeyboardEvent, MouseEvent};

use crate::{connection::socket::Connection, register_event, rendering::{events::{self, on_resize, EventType}, renderer::Renderer}, storage_get};

pub type LockedWorld = MutexGuard<'static, Box<World>>;

pub static WORLD: Mutex<Option<Box<SendWrapper<World>>>> = Mutex::new(None);

pub fn init_world() {
    *WORLD.lock().unwrap() = Some(Box::new(SendWrapper::new(World::new())));
}

pub fn get_world() -> MappedMutexGuard<'static, Box<SendWrapper<World>>> {
    let mut world_guard = WORLD.lock().unwrap();
    MutexGuard::map(world_guard, |world_opt| {
        world_opt.as_mut().expect("WORLD has not been initialized")
    })
}

pub struct World {
    pub renderer: Renderer,
    pub sounds: SoundHolder,
    pub connection: Connection
}

impl World {
    pub fn new() -> World {
        let sounds = SoundHolder::new(vec![
            ("button_click", false),
            ("dialogue_normal", false),
            ("dialogue_unsettling", false),
            ("soundtrack_game", true),
            ("soundtrack_home", true),
            ("soundtrack_lore", true)
        ]);

        World {
            renderer: Renderer::new(),
            sounds,
            connection: Connection::new()
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
                Renderer::tick(&mut get_world(), ts);
            }) as Box<dyn FnMut(_)>);
            
            let _ = window()
                .request_animation_frame(closure.as_ref().unchecked_ref());
            
            closure.forget();
        }        
    }
}