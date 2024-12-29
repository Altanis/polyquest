use std::{collections::VecDeque};

use gloo::console::console;
use gloo_utils::{document, window};
use shared::utils::vec2::Vector2D;
use ui::{canvas2d::{Canvas2d, Transform}, core::{ElementType, Events, HoverEffects, UiElement}, elements::{body::Body, button::Button, label::{Label, TextEffects}}, get_element_by_id_and_cast, gl::webgl::WebGl, translate, utils::{color::Color, sound::Sound}};
use web_sys::{wasm_bindgen::{prelude::Closure, JsCast}, HtmlDivElement, Performance};

use crate::{connection::socket::ConnectionState, world::{self, get_world, World}, SHADERS_ENABLED};

use super::phases::GamePhase;

#[derive(Default)]
pub struct TimeInformation {
    ticks: u32,
    start_time: f64,
    last_render: f64,
    deltas: VecDeque<f64>
}

pub struct Renderer {
    pub canvas2d: Canvas2d,
    pub gl: WebGl,
    pub mouse: Vector2D<f32>,
    pub time: TimeInformation,
    pub phase: GamePhase,
    previous_phase: GamePhase,
    pub body: Body,
    pub fps_counter: Label
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer {
            canvas2d: Canvas2d::new("offscreen_canvas"),
            gl: WebGl::new(),
            mouse: Vector2D::ZERO,
            time: TimeInformation::default(),
            phase: GamePhase::default(),
            previous_phase: GamePhase::default(),
            body: Body::default()
                .with_fill(Color(14, 14, 14)),
            fps_counter: Label::new()
                .with_text("165.0 FPS".to_string())
                .with_fill(Color::WHITE)
                .with_font(28.0)
                .with_stroke(Color::BLACK)
                .with_transform(translate!(75.0, 35.0))
                .with_events(Events::default().with_hoverable(false))
        }
    }

    pub fn tick(world: &mut World, timestamp: f64) {
        world.sounds.tick();

        if !world.renderer.phase.same_phase(&world.renderer.previous_phase) {
            world.renderer.body.set_children(vec![]);
        }
        world.renderer.previous_phase = world.renderer.phase.clone();

        if !world.sounds.can_play && window().navigator().user_activation().has_been_active() {
            world.sounds.can_play = true;

            match world.renderer.phase {
                GamePhase::Lore(_) => {
                    world.sounds.get_mut_sound("soundtrack_lore").play();
                    world.sounds.get_mut_sound("soundtrack_home").stop();
                    world.sounds.get_mut_sound("soundtrack_game").stop();
                },
                GamePhase::Home(_) => {
                    world.sounds.get_mut_sound("soundtrack_lore").stop();
                    world.sounds.get_mut_sound("soundtrack_home").play();
                    world.sounds.get_mut_sound("soundtrack_game").stop();
                },
                GamePhase::Game => {
                    world.sounds.get_mut_sound("soundtrack_lore").stop();
                    world.sounds.get_mut_sound("soundtrack_home").stop();
                    world.sounds.get_mut_sound("soundtrack_game").play();
                },
                _ => ()
            }
        }

        if world.renderer.time.start_time == 0.0 {
            world.renderer.time.start_time = timestamp;
        }

        let delta_average = {
            let time = &mut world.renderer.time;
            time.ticks += 1;
        
            let delta = timestamp - time.last_render;
            time.last_render = timestamp;
        
            time.deltas.push_back(delta);
            if time.deltas.len() > 100 {
                time.deltas.pop_front();
            }
        
            time.deltas.iter().sum::<f64>() / time.deltas.len() as f64
        };

        world.renderer.canvas2d.clear_rect();

        world.renderer.canvas2d.save();
        world.renderer.canvas2d.set_line_join("round");

        let elements = match world.renderer.phase {
            GamePhase::Lore(_) => GamePhase::generate_lore_elements(world),
            GamePhase::Home(_) => GamePhase::generate_homescreen_elements(world),
            GamePhase::Game => GamePhase::generate_game_elements(world),
            GamePhase::Death => vec![],
        };
        
        let mut element_ids: Vec<String> = elements.iter().map(|e| e.get_id()).collect();
        
        elements.into_iter().for_each(|element| {
            match world.renderer.body.get_element_by_id(&element.get_id()) {
                Some((_, el)) if !el.has_animation_state() => {
                    world.renderer.body.delete_element_by_id(&element.get_id(), false);
                    world.renderer.body.get_mut_children().push(element);
                },
                None => world.renderer.body.get_mut_children().push(element),
                _ => {}
            }
        });
        
        let stale_ids: Vec<String> = world.renderer.body.get_mut_children()
            .iter()
            .filter(|e| !element_ids.contains(&e.get_id()))
            .map(|e| e.get_id())
            .collect();
        
        stale_ids.into_iter().for_each(|id| {
            world.renderer.body.delete_element_by_id(&id, true);
        });
        
        match world.renderer.phase {
            GamePhase::Lore(_) => Renderer::render_lore(world, delta_average),
            GamePhase::Home(_) => Renderer::render_homescreen(world, delta_average),
            GamePhase::Game => Renderer::render_game(world, delta_average),
            GamePhase::Death => ()
        }

        world.renderer.canvas2d.restore();

        if SHADERS_ENABLED {
            world.renderer.gl.render(
                &world.renderer.canvas2d, 
                (timestamp - world.renderer.time.start_time) / 1000.0
            );
        }

        let closure = Closure::once(move |ts: f64| {
            Renderer::tick(&mut get_world(), ts);
        });

        let _ = window()
            .request_animation_frame(closure.as_ref().unchecked_ref());

        closure.forget();
    }

    pub fn render_lore(world: &mut World, delta_average: f64) {        
        get_element_by_id_and_cast!("text_input_container", HtmlDivElement)
            .style()
            .set_property("display", "none");

        let dimensions = world.renderer.body.get_bounding_rect().dimensions;

        world.renderer.canvas2d.save();
        world.renderer.body.render(&mut world.renderer.canvas2d, dimensions);
        world.renderer.body.render_children(&mut world.renderer.canvas2d);
        world.renderer.canvas2d.restore();

        world.renderer.fps_counter.set_text(format!("{:.1} FPS", 1000.0 / delta_average));
        world.renderer.fps_counter.render(&mut world.renderer.canvas2d, dimensions);
    }

    pub fn render_homescreen(world: &mut World, delta_average: f64) {
        let modal_exists = world.renderer.body.get_mut_children()
            .iter_mut()
            .any(|child| child.get_identity() == ElementType::Modal);
        let should_display_textbox = !modal_exists && world.connection.state == ConnectionState::Connected;
        
        get_element_by_id_and_cast!("text_input_container", HtmlDivElement)
            .style()
            .set_property("display", if should_display_textbox { "block" } else { "none" });

        let dimensions = world.renderer.body.get_bounding_rect().dimensions;

        world.renderer.canvas2d.save();
        world.renderer.body.render(&mut world.renderer.canvas2d, dimensions);
        GamePhase::render_homescreen(world);
        world.renderer.body.render_children(&mut world.renderer.canvas2d);
        world.renderer.canvas2d.restore();

        world.renderer.fps_counter.set_text(format!("{:.1} FPS", 1000.0 / delta_average));
        world.renderer.fps_counter.render(&mut world.renderer.canvas2d, dimensions);
    }

    pub fn render_game(world: &mut World, delta_average: f64) {
        get_element_by_id_and_cast!("text_input_container", HtmlDivElement)
            .style()
            .set_property("display", "none");
        
        let dimensions = world.renderer.body.get_bounding_rect().dimensions;

        world.renderer.canvas2d.save();
        world.renderer.body.render(&mut world.renderer.canvas2d, dimensions);
        
        world.renderer.canvas2d.save();
        world.renderer.canvas2d.reset_transform();
        GamePhase::render_game(world, delta_average);
        world.renderer.canvas2d.restore();

        world.renderer.body.render_children(&mut world.renderer.canvas2d);
        world.renderer.fps_counter.set_text(format!("{:.1} FPS", 1000.0 / delta_average));
        world.renderer.fps_counter.render(&mut world.renderer.canvas2d, dimensions);
        
        world.renderer.canvas2d.restore();
    }
}