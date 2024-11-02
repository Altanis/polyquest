use gloo::console::console;
use gloo_utils::{body, document, window};
use shared::utils::vec2::Vector2D;
use ui::core::UiElement;
use web_sys::{wasm_bindgen::JsCast, BeforeUnloadEvent, KeyboardEvent, MouseEvent};
use crate::world::World;

use super::phases::GamePhase;
#[derive(Debug)]

pub enum EventType {
    BeforeUnload(BeforeUnloadEvent),
    ContextMenu(MouseEvent),
    Resize,
    MouseDown(MouseEvent),
    MouseUp(MouseEvent),
    MouseMove(MouseEvent),
    KeyDown(KeyboardEvent),
    KeyUp(KeyboardEvent),
}

pub fn handle_event(world: &mut World, event_type: EventType) {
    match event_type {
        EventType::BeforeUnload(event) => {let _ = on_beforeunload(world, event);},
        EventType::ContextMenu(event) => on_contextmenu(world, event),
        EventType::Resize => on_resize(world),
        EventType::MouseDown(event) => on_mousedown(world, event),
        EventType::MouseUp(event) => on_mouseup(world, event),
        EventType::MouseMove(event) => on_mousemove(world, event),
        EventType::KeyDown(event) => on_keydown(world, event),
        EventType::KeyUp(event) => on_keyup(world, event),
    }
}

#[macro_export]
macro_rules! register_event {
    ($event:expr) => {{
        let event_name_fn = concat!("on_", $event);

        let closure = Closure::wrap(Box::new(move |event: Event| {
            let mut world = get_world();

            let event_type = match event_name_fn {
                "on_beforeunload" => EventType::BeforeUnload(event.dyn_into::<BeforeUnloadEvent>().unwrap()),
                "on_contextmenu" => EventType::ContextMenu(event.dyn_into::<MouseEvent>().unwrap()),
                "on_resize" => EventType::Resize,
                "on_mousedown" => EventType::MouseDown(event.dyn_into::<MouseEvent>().unwrap()),
                "on_mouseup" => EventType::MouseUp(event.dyn_into::<MouseEvent>().unwrap()),
                "on_mousemove" => EventType::MouseMove(event.dyn_into::<MouseEvent>().unwrap()),
                "on_keydown" => EventType::KeyDown(event.dyn_into::<KeyboardEvent>().unwrap()),
                "on_keyup" => EventType::KeyUp(event.dyn_into::<KeyboardEvent>().unwrap()),
                _ => unimplemented!("Event not implemented: {}", event_name_fn),
            };

            events::handle_event(&mut world, event_type);
        }) as Box<dyn FnMut(_)>);

        let _ = gloo_utils::window()
            .add_event_listener_with_callback(
                $event,
                closure.as_ref().unchecked_ref(),
            );
        
        closure.forget();
    }};
}

pub fn on_beforeunload(world: &mut World, event: BeforeUnloadEvent) -> BeforeUnloadEvent {
    event.set_return_value("Are you sure you want to leave? You will lose your progress.");
    event
}

pub fn on_contextmenu(world: &mut World, event: MouseEvent) {
    event.prevent_default();
}

pub fn on_resize(world: &mut World) {
    world.renderer.canvas2d.resize(&window());
}

pub fn on_mousedown(world: &mut World, event: MouseEvent) {}
pub fn on_mouseup(world: &mut World, event: MouseEvent) {
    let mut is_hovering = false;
    let mut point = Vector2D::new(event.client_x() as f32, event.client_y() as f32);
    point *= window().device_pixel_ratio() as f32;

    let children = world.renderer.body.get_mut_children();

    for ui_element in children.iter_mut() {
        let hovering = ui_element.get_mut_events().hoverable &&
            ui_element.get_bounding_rect().intersects(point);

        if hovering {
            ui_element.set_clicked(true);
        }
    }
}

pub fn on_mousemove(world: &mut World, event: MouseEvent) {
    let mut is_hovering = false;
    let mut point = Vector2D::new(event.client_x() as f32, event.client_y() as f32);
    point *= window().device_pixel_ratio() as f32;

    let children = world.renderer.body.get_mut_children();

    for ui_element in children.iter_mut() {
        let hovering = ui_element.get_mut_events().hoverable &&
            ui_element.get_bounding_rect().intersects(point);
        ui_element.set_hovering(hovering);

        if !is_hovering && hovering {
            is_hovering = hovering;
        }
    }

    let context = &mut world.renderer.gl;
    context.set_cursor(if is_hovering { "pointer" } else { "default "});
}

pub fn on_keydown(world: &mut World, event: KeyboardEvent) {}
pub fn on_keyup(world: &mut World, event: KeyboardEvent) {}

// touchstart, touchend, etc.