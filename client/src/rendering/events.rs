use gloo::console::console;
use gloo_utils::{body, document, window};
use shared::{connection::packets::Inputs, game::entity::MAX_STAT_INVESTMENT, utils::vec2::Vector2D};
use ui::{core::{ElementType, UiElement}, get_element_by_id_and_cast};
use web_sys::{wasm_bindgen::JsCast, BeforeUnloadEvent, HtmlInputElement, KeyboardEvent, MouseEvent};
use crate::{connection::packets, world::World};

use self::packets::form_stats_packet;

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

pub enum KeyCode {
    Enter,
    Escape,
    KeyW, ArrowUp,
    KeyA, ArrowLeft,
    KeyS, ArrowDown,
    KeyD, ArrowRight,
    KeyK,
    One, Two, Three, Four, Five, Six, Seven, Eight
}

impl TryInto<KeyCode> for u32 {
    type Error = ();

    fn try_into(self) -> Result<KeyCode, Self::Error> {
        match self {
            13 => Ok(KeyCode::Enter),      // Enter
            27 => Ok(KeyCode::Escape),     // Escape
            87 => Ok(KeyCode::KeyW),       // 'W' key
            65 => Ok(KeyCode::KeyA),       // 'A' key
            83 => Ok(KeyCode::KeyS),       // 'S' key
            68 => Ok(KeyCode::KeyD),       // 'D' key
            38 => Ok(KeyCode::ArrowUp),    // ArrowUp key
            37 => Ok(KeyCode::ArrowLeft),  // ArrowLeft key
            40 => Ok(KeyCode::ArrowDown),  // ArrowDown key
            39 => Ok(KeyCode::ArrowRight), // ArrowRight key
            75 => Ok(KeyCode::KeyK),       // 'K' key
            49 => Ok(KeyCode::One),
            50 => Ok(KeyCode::Two),
            51 => Ok(KeyCode::Three),
            52 => Ok(KeyCode::Four),
            53 => Ok(KeyCode::Five),
            54 => Ok(KeyCode::Six),
            55 => Ok(KeyCode::Seven),
            56 => Ok(KeyCode::Eight),            
            _ => Err(()),
        }
    }
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
    let mut point = Vector2D::new(event.client_x() as f32, event.client_y() as f32);
    point *= window().device_pixel_ratio() as f32;

    let mut z_index = -999;
    for ui_element in world.renderer.body.get_mut_children().iter_mut().rev() {
        if ui_element.get_z_index() < z_index {
            break;
        } else {
            z_index = ui_element.get_z_index();
        }
        
        let hovering = ui_element.get_mut_events().hoverable &&
            ui_element.get_bounding_rect().intersects(point);

        ui_element.set_clicked(hovering, &event);
    }
}

pub fn on_mousemove(world: &mut World, event: MouseEvent) {
    let mut is_hovering = false;
    let mut point = Vector2D::new(event.client_x() as f32, event.client_y() as f32);
    point *= window().device_pixel_ratio() as f32;

    let mut z_index = -999;
    for ui_element in world.renderer.body.get_mut_children().iter_mut().rev() {
        if ui_element.get_z_index() < z_index {
            break;
        } else {
            z_index = ui_element.get_z_index();
        }

        let hovering = ui_element.get_mut_events().hoverable &&
            ui_element.get_bounding_rect().intersects(point);

        let should_hover = ui_element.set_hovering(hovering, &event);
        if !is_hovering && should_hover {
            is_hovering = true;
        }
    }

    let context = &mut world.renderer.canvas2d;
    is_hovering = true;
    context.set_cursor(if is_hovering { "pointer" } else { "default" });

    world.game.self_entity.physics.mouse = point - (world.renderer.canvas2d.get_dimensions() * (1.0 / 2.0));
    world.game.self_entity.physics.angle.value = world.game.self_entity.physics.mouse.angle();
}

pub fn on_keydown(world: &mut World, event: KeyboardEvent) {
    match event.key_code().try_into() {
        Ok(KeyCode::KeyW) | Ok(KeyCode::ArrowUp) => world.game.self_entity.physics.inputs.set_flag(Inputs::Up),
        Ok(KeyCode::KeyA) | Ok(KeyCode::ArrowLeft) => world.game.self_entity.physics.inputs.set_flag(Inputs::Left),
        Ok(KeyCode::KeyS) | Ok(KeyCode::ArrowDown) => world.game.self_entity.physics.inputs.set_flag(Inputs::Down),
        Ok(KeyCode::KeyD) | Ok(KeyCode::ArrowRight) => world.game.self_entity.physics.inputs.set_flag(Inputs::Right),
        Ok(KeyCode::KeyK) => world.game.self_entity.physics.inputs.set_flag(Inputs::LevelUp),
        _ => ()
    }
}

pub fn on_keyup(world: &mut World, event: KeyboardEvent) {
    match event.key_code().try_into() {
        Ok(KeyCode::Escape) => {
            let mut deletion_indices = Vec::new();
            for (i, child) in world.renderer.body.get_mut_children().iter_mut().enumerate() {
                if child.get_identity() == ElementType::Modal {
                    deletion_indices.push(i);
                }
            }
        
            for index in deletion_indices {
                world.renderer.body.get_mut_children()[index]
                    .destroy();
            }
        },
        Ok(KeyCode::Enter) => {
            let name = get_element_by_id_and_cast!("text_input", HtmlInputElement)
                .value();

            if let GamePhase::Home(_) = world.renderer.phase && !name.is_empty() {
                world.sounds.get_mut_sound("button_click").play();
                world.connection.send_message(packets::form_spawn_packet(name));
            }
        },
        Ok(KeyCode::KeyW) | Ok(KeyCode::ArrowUp) => world.game.self_entity.physics.inputs.clear_flag(Inputs::Up),
        Ok(KeyCode::KeyA) | Ok(KeyCode::ArrowLeft) => world.game.self_entity.physics.inputs.clear_flag(Inputs::Left),
        Ok(KeyCode::KeyS) | Ok(KeyCode::ArrowDown) => world.game.self_entity.physics.inputs.clear_flag(Inputs::Down),
        Ok(KeyCode::KeyD) | Ok(KeyCode::ArrowRight) => world.game.self_entity.physics.inputs.clear_flag(Inputs::Right),
        Ok(KeyCode::KeyK) => world.game.self_entity.physics.inputs.clear_flag(Inputs::LevelUp),
        Ok(KeyCode::One) | Ok(KeyCode::Two) | Ok(KeyCode::Three) | Ok(KeyCode::Four) | Ok(KeyCode::Five) | Ok(KeyCode::Six) | Ok(KeyCode::Seven) | Ok(KeyCode::Eight)
        => {
            let i = (event.key_code() as u8 - b'0') as usize - 1;

            if world.game.self_entity.display.stat_investments[i] < MAX_STAT_INVESTMENT
                && world.game.self_entity.display.available_stat_points > 0 
            {
                world.connection.send_message(form_stats_packet(i));
            }
        },
        _ => ()
    }
}

// touchstart, touchend, etc.