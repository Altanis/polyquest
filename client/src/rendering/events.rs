use gloo::console::console;
use gloo_utils::{body, document, window};
use shared::{connection::packets::{Inputs, ServerboundPackets}, game::entity::{Notification, MAX_STAT_INVESTMENT}, utils::{color::Color, vec2::Vector2D}};
use ui::{core::{ElementType, UiElement}, get_element_by_id_and_cast};
use web_sys::{wasm_bindgen::JsCast, BeforeUnloadEvent, HtmlInputElement, KeyboardEvent, MouseEvent, WheelEvent};
use crate::{connection::packets, world::World};

use self::packets::form_stats_packet;

use super::{phases::GamePhase, renderer::ModalType};
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
    Wheel(WheelEvent)
}

#[derive(Debug, num_enum::TryFromPrimitive)]
#[repr(u32)]
pub enum KeyCode {
    Shift       = 16,
    Enter       = 13,
    Escape      = 27,
    Space       = 32,
    KeyE        = 69,
    KeyW        = 87,
    ArrowUp     = 38,
    KeyA        = 65,
    ArrowLeft   = 37,
    KeyS        = 83,
    ArrowDown   = 40,
    KeyD        = 68,
    ArrowRight  = 39,
    KeyK        = 75,
    One         = 49,
    Two         = 50,
    Three       = 51,
    Four        = 52,
    Five        = 53,
    Six         = 54,
    Seven       = 55,
    Eight       = 56,
    Backslash       = 220
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
        EventType::Wheel(event) => on_wheel(world, event)
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
                "on_wheel" => EventType::Wheel(event.dyn_into::<WheelEvent>().unwrap()),
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
    world.renderer.canvas2d.resize();
}

pub fn on_mousedown(world: &mut World, event: MouseEvent) {
    match event.which() {
        1 => world.game.self_entity.physics.inputs.set_flag(Inputs::Shoot),
        3 => world.game.self_entity.physics.inputs.set_flag(Inputs::Repel),
        _ => {}
    }
}

pub fn on_mouseup(world: &mut World, event: MouseEvent) {
    match event.which() {
        1 => world.game.self_entity.physics.inputs.clear_flag(Inputs::Shoot),
        3 => world.game.self_entity.physics.inputs.clear_flag(Inputs::Repel),
        _ => {}
    }

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

    for ui_element in world.renderer.body.get_mut_children().iter_mut().rev() {
        let hovering = ui_element.get_mut_events().hoverable &&
            ui_element.get_bounding_rect().intersects(point);

        let should_hover = ui_element.set_hovering(hovering, &event);
        if !is_hovering && should_hover {
            is_hovering = true;
        }
    }

    let context = &mut world.renderer.canvas2d;
    context.set_cursor(if is_hovering { "pointer" } else { "default" });

    world.game.self_entity.physics.mouse = point - (world.renderer.canvas2d.get_dimensions() * (1.0 / 2.0));
    world.game.self_entity.physics.angle.value = world.game.self_entity.physics.mouse.angle();
}

pub fn on_keydown(world: &mut World, event: KeyboardEvent) {
    let is_modal_open = world.renderer.body.get_mut_children().iter()
        .any(|e| e.get_identity() == ElementType::Modal);

    if !world.game.self_entity.display.typing && !is_modal_open {
        match event.key_code().try_into() {
            Ok(KeyCode::KeyW) | Ok(KeyCode::ArrowUp) => world.game.self_entity.physics.inputs.set_flag(Inputs::Up),
            Ok(KeyCode::KeyA) | Ok(KeyCode::ArrowLeft) => world.game.self_entity.physics.inputs.set_flag(Inputs::Left),
            Ok(KeyCode::KeyS) | Ok(KeyCode::ArrowDown) => world.game.self_entity.physics.inputs.set_flag(Inputs::Down),
            Ok(KeyCode::KeyD) | Ok(KeyCode::ArrowRight) => world.game.self_entity.physics.inputs.set_flag(Inputs::Right),
            Ok(KeyCode::KeyK) => world.game.self_entity.physics.inputs.set_flag(Inputs::LevelUp),
            Ok(KeyCode::Space) => world.game.self_entity.physics.inputs.set_flag(Inputs::Shoot),
            Ok(KeyCode::Shift) => world.game.self_entity.physics.inputs.set_flag(Inputs::Repel),
            Ok(KeyCode::Backslash) => world.game.self_entity.physics.inputs.set_flag(Inputs::Switch),
            Ok(KeyCode::KeyE) => {
                world.game.self_entity.physics.auto_fire = !world.game.self_entity.physics.auto_fire;
                world.game.self_entity.display.notifications.push(Notification {
                    message: format!("Auto Fire: {}", if world.game.self_entity.physics.auto_fire { "ON" } else { "OFF" }),
                    color: Color::BLUE,
                    lifetime: 150,
                    ..Default::default()
                });
                
                if !world.game.self_entity.physics.auto_fire {
                    world.game.self_entity.physics.inputs.clear_flag(Inputs::Shoot);
                }
            },
            _ => ()
        }
    }
}

pub fn on_keyup(world: &mut World, event: KeyboardEvent) {
    let is_modal_open = world.renderer.body.get_mut_children().iter()
        .any(|e| e.get_identity() == ElementType::Modal);

    match event.key_code().try_into() {
        Ok(KeyCode::Escape) => {
            if world.renderer.phase == GamePhase::Game {
                world.connection.send_message(packets::form_chat_packet(
                    Some(false),
                    String::new()
                ), ServerboundPackets::Chat);
            }

            let mut deletion_indices = Vec::new();
            for (i, child) in world.renderer.body.get_mut_children().iter_mut().enumerate() {
                if child.get_identity() == ElementType::Modal {
                    if let Some(modal_idx) = world.renderer.modals.iter().position(|modal| {
                        match child.get_id().as_str() {
                            s if s.contains("settings") => matches!(modal, ModalType::Settings(_)),
                            s if s.contains("clans") => matches!(modal, ModalType::Clans(_)),
                            s if s.contains("clan-create") => matches!(modal, ModalType::ClanCreate(_)),
                            _ => false,
                        }
                    }) {
                        world.renderer.modals.remove(modal_idx);
                    }
            
                    deletion_indices.push(i);
                }
            }            
        
            for index in deletion_indices {
                world.renderer.body.get_mut_children()[index]
                    .destroy();
            }
        },
        Ok(KeyCode::Enter) => {
            match world.renderer.phase {
                GamePhase::Home(_) => {
                    let name = get_element_by_id_and_cast!("text_input", HtmlInputElement)
                        .value();

                    world.sounds.get_mut_sound("button_click").play();
                    world.connection.send_message(packets::form_spawn_packet(if name.is_empty() {
                        "unnamed".to_string()
                    } else {
                        name
                    }), ServerboundPackets::Spawn);
                },
                GamePhase::Game => {
                    if !is_modal_open {
                        if world.game.self_entity.display.typing && !get_element_by_id_and_cast!("chat_input", HtmlInputElement).value().is_empty() {
                            world.connection.send_message(packets::form_chat_packet(
                                None,
                                get_element_by_id_and_cast!("chat_input", HtmlInputElement).value().chars().take(72).collect::<String>()
                            ), ServerboundPackets::Chat);
    
                            get_element_by_id_and_cast!("chat_input", HtmlInputElement).set_value("");
                        } else {
                            world.connection.send_message(packets::form_chat_packet(
                                Some(true),
                                String::new()
                            ), ServerboundPackets::Chat);
                        }   
                    }
                },
                GamePhase::Death => world.renderer.change_phase(GamePhase::Home(Box::default())),
                _ => {}
            }
        },
        _ => {}
    }

    if !world.game.self_entity.display.typing && !is_modal_open {
        match event.key_code().try_into() {
            Ok(KeyCode::KeyW) | Ok(KeyCode::ArrowUp) => world.game.self_entity.physics.inputs.clear_flag(Inputs::Up),
            Ok(KeyCode::KeyA) | Ok(KeyCode::ArrowLeft) => world.game.self_entity.physics.inputs.clear_flag(Inputs::Left),
            Ok(KeyCode::KeyS) | Ok(KeyCode::ArrowDown) => world.game.self_entity.physics.inputs.clear_flag(Inputs::Down),
            Ok(KeyCode::KeyD) | Ok(KeyCode::ArrowRight) => world.game.self_entity.physics.inputs.clear_flag(Inputs::Right),
            Ok(KeyCode::KeyK) => world.game.self_entity.physics.inputs.clear_flag(Inputs::LevelUp),
            Ok(KeyCode::Space) => world.game.self_entity.physics.inputs.clear_flag(Inputs::Shoot),
            Ok(KeyCode::Shift) => world.game.self_entity.physics.inputs.clear_flag(Inputs::Repel),
            Ok(KeyCode::Backslash) => world.game.self_entity.physics.inputs.clear_flag(Inputs::Switch),
            Ok(KeyCode::One) | Ok(KeyCode::Two) | Ok(KeyCode::Three) | Ok(KeyCode::Four) | Ok(KeyCode::Five) | Ok(KeyCode::Six) | Ok(KeyCode::Seven) | Ok(KeyCode::Eight)
            => {
                let i = (event.key_code() as u8 - b'0') as usize - 1;
    
                if world.game.self_entity.display.stat_investments[i] < MAX_STAT_INVESTMENT
                    && world.game.self_entity.display.available_stat_points > 0 
                {
                    world.connection.send_message(form_stats_packet(i), ServerboundPackets::Stats);
                }
            },
            _ => ()
        }
    }
}

pub fn on_wheel(world: &mut World, event: WheelEvent) {
    event.prevent_default();
}

// touchstart, touchend, etc.