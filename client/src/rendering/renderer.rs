use std::{collections::VecDeque};

use gloo::console::{self, console};
use gloo_utils::{document, window};
use shared::{fuzzy_compare, game::theme::OUTBOUNDS_FILL, lerp, utils::{color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use ui::{canvas2d::{Canvas2d, Transform}, core::{ElementType, Events, HoverEffects, UiElement}, elements::{body::Body, button::Button, label::{Label, TextEffects}}, get_element_by_id_and_cast, gl::webgl::WebGl, translate};
use web_sys::{wasm_bindgen::{prelude::Closure, JsCast}, HtmlDivElement, Performance};

use crate::{connection::socket::ConnectionState, world::{self, get_world, World}, SHADERS_ENABLED};

use super::phases::GamePhase;

#[derive(Default)]
pub struct TimeInformation {
    pub ticks: u32,
    start_time: f64,
    last_render: f64,
    deltas: VecDeque<f64>
}

pub struct Renderer {
    pub canvas2d: Canvas2d,
    pub gl: WebGl,
    pub mouse: Vector2D,
    pub time: TimeInformation,
    pub phase: GamePhase,
    previous_phase: GamePhase,
    pub body: Body,
    pub backdrop_opacity: Interpolatable<f32>,
    pub fps_counter: Label,

    pub phase_switch: Option<GamePhase>,
    phase_switch_radius: Interpolatable<f32>
}

impl Renderer {
    pub fn new() -> Renderer {
        let mut phase_switch_radius = Interpolatable::new(0.0);
        phase_switch_radius.direction = 0.0;

        Renderer {
            canvas2d: Canvas2d::new("offscreen_canvas"),
            gl: WebGl::new(),
            mouse: Vector2D::ZERO,
            time: TimeInformation::default(),
            phase: GamePhase::default(),
            previous_phase: GamePhase::default(),
            body: Body::default()
                .with_fill(Color::from_numeric(0x2C3E50)),
            backdrop_opacity: Interpolatable::new(0.0),
            fps_counter: Label::new()
                .with_text("165.0 FPS".to_string())
                .with_fill(Color::WHITE)
                .with_font(28.0)
                .with_stroke(Color::BLACK)
                .with_transform(translate!(10.0, 35.0))
                .with_events(Events::default().with_hoverable(false))
                .with_align("left"),
            phase_switch: None,
            phase_switch_radius
        }
    }

    pub fn change_phase(&mut self, phase: GamePhase) {
        if phase == GamePhase::Death {
            self.phase = phase;
            return;
        }

        if !self.phase.same_phase(&phase) {
            self.phase_switch = Some(phase);
        }
    }

    pub fn tick(world: &mut World, timestamp: f64) {
        world.sounds.tick();
        world.renderer.previous_phase = world.renderer.phase.clone();
        world.renderer.canvas2d.resize();

        let dimensions = world.renderer.canvas2d.get_dimensions();

        if window().navigator().user_activation().has_been_active() {
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
                GamePhase::Game | GamePhase::Death => {
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

        let dt = (delta_average / 16.66).clamp(0.0, 1.0) as f32;

        world.renderer.phase_switch_radius.value = lerp!(
            world.renderer.phase_switch_radius.value,
            world.renderer.phase_switch_radius.target,
            0.1 * dt
        );

        if let Some(new_phase) = &world.renderer.phase_switch {
            if fuzzy_compare!(world.renderer.phase_switch_radius.value, world.renderer.phase_switch_radius.target, 5.0) {
                world.renderer.phase_switch_radius.target = if world.renderer.phase_switch_radius.direction == 1.0 {
                    world.renderer.phase = new_phase.clone();
                    world.renderer.phase_switch_radius.direction = -1.0;
                    dimensions.x
                } else {
                    world.renderer.phase_switch_radius.direction = 0.0;
                    world.renderer.phase_switch = None;
                    dimensions.x
                };
            }

            world.renderer.phase_switch_radius.target = if world.renderer.phase_switch_radius.direction == 0.0 {
                world.renderer.phase_switch_radius.value = dimensions.x;
                world.renderer.phase_switch_radius.direction = 1.0;
                0.0
            } else if world.renderer.phase_switch_radius.direction == 1.0 {
                0.0
            } else {
                dimensions.x
            };
        } else {
            world.renderer.phase_switch_radius.direction = 0.0;
        }

        world.renderer.canvas2d.clear_rect();

        world.renderer.canvas2d.save();
        world.renderer.canvas2d.set_line_join("round");

        let elements = match world.renderer.phase {
            GamePhase::Lore(_) => GamePhase::generate_lore_elements(world),
            GamePhase::Home(_) => GamePhase::generate_homescreen_elements(world),
            GamePhase::Game => GamePhase::generate_game_elements(world),
            GamePhase::Death => GamePhase::generate_death_elements(world),
        };
        
        let mut element_ids: Vec<String> = elements.iter().map(|e| e.get_id()).collect();
        
        elements.into_iter().for_each(|mut element| {
            match world.renderer.body.get_element_by_id(&element.get_id()) {
                Some((_, el)) if !el.has_animation_state() => {
                    world.renderer.body.delete_element_by_id(&element.get_id(), false);
                    world.renderer.body.get_mut_children().push(element);
                },
                None => world.renderer.body.get_mut_children().push(element),
                _ => {}
            }
        });
        
        let stale_elements: Vec<&mut Box<dyn UiElement>> = world.renderer.body.get_mut_children()
            .iter_mut()
            .filter(|e| e.get_identity() != ElementType::Modal && !element_ids.contains(&e.get_id()))
            .collect();
        
        stale_elements.into_iter().for_each(|element| element.destroy());
        
        match world.renderer.phase {
            GamePhase::Lore(_) => Renderer::render_lore(world, delta_average),
            GamePhase::Home(_) => Renderer::render_homescreen(world, delta_average),
            GamePhase::Game => Renderer::render_game(world, delta_average, false),
            GamePhase::Death => Renderer::render_game(world, delta_average, true)
        }

        world.connection.latency.value = lerp!(world.connection.latency.value, world.connection.latency.target, 0.15 * dt as f64);
        world.connection.mspt.value = lerp!(world.connection.mspt.value, world.connection.mspt.target, 0.15 * dt);

        world.renderer.fps_counter.set_text(
            format!(
                "{:.1} FPS / {:.1} ms / {:.1} mspt", 
                1000.0 / delta_average,
                world.connection.latency.value,
                world.connection.mspt.value
            )
        );

        if world.renderer.phase_switch_radius.direction != 0.0 {
            world.renderer.canvas2d.save();
            world.renderer.canvas2d.reset_transform();
            
            world.renderer.canvas2d.save();
            world.renderer.canvas2d.global_composite_operation("destination-in");
            world.renderer.canvas2d.fill_style(Color::WHITE);
            world.renderer.canvas2d.begin_arc(dimensions.x / 2.0, dimensions.y / 2.0, world.renderer.phase_switch_radius.value, std::f32::consts::TAU);
            world.renderer.canvas2d.fill();
            world.renderer.canvas2d.restore();
    
            world.renderer.canvas2d.global_composite_operation("destination-over");
            world.renderer.canvas2d.fill_style(Color::BLACK);
            world.renderer.canvas2d.fill_rect(0.0, 0.0, dimensions.x, dimensions.y);
            world.renderer.canvas2d.restore();
        }

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
        let dt = (delta_average / 16.66).clamp(0.0, 1.0) as f32;

        get_element_by_id_and_cast!("text_input_container", HtmlDivElement)
            .style()
            .set_property("display", "none");

        let dimensions = world.renderer.body.get_bounding_rect().dimensions;

        world.renderer.canvas2d.save();
        world.renderer.body.render(&mut world.renderer.canvas2d, dimensions);
        world.renderer.body.render_children(&mut world.renderer.canvas2d);
        world.renderer.canvas2d.restore();

        world.renderer.fps_counter.render(&mut world.renderer.canvas2d, dimensions);
    }

    pub fn render_homescreen(world: &mut World, delta_average: f64) {
        let dt = (delta_average / 16.66).clamp(0.0, 1.0) as f32;

        let modal_exists = world.renderer.body.get_mut_children()
            .iter_mut()
            .any(|child| child.get_identity() == ElementType::Modal);
        let should_display_textbox = !modal_exists
            && world.connection.state == ConnectionState::Connected 
            && (matches!(world.renderer.phase_switch, Some(GamePhase::Home(_))) || world.renderer.phase_switch.is_none());
        
        get_element_by_id_and_cast!("text_input_container", HtmlDivElement)
            .style()
            .set_property("display", if should_display_textbox { "block" } else { "none" });

        let dimensions = world.renderer.body.get_bounding_rect().dimensions;

        world.renderer.canvas2d.save();
        world.renderer.body.render(&mut world.renderer.canvas2d, dimensions);
        GamePhase::render_homescreen(world);
        world.renderer.body.render_children(&mut world.renderer.canvas2d);
        world.renderer.canvas2d.restore();

        world.renderer.fps_counter.render(&mut world.renderer.canvas2d, dimensions);
    }

    pub fn render_game(world: &mut World, delta_average: f64, is_dead: bool) {
        let dt = (delta_average / 16.66).clamp(0.0, 1.0) as f32;

        get_element_by_id_and_cast!("text_input_container", HtmlDivElement)
            .style()
            .set_property("display", "none");

        world.renderer.canvas2d.save();
        

        world.renderer.body.dimensions = world.renderer.canvas2d.get_dimensions();
        world.renderer.canvas2d.translate(world.renderer.body.dimensions.x / 2.0, world.renderer.body.dimensions.y / 2.0);
        let factor = window().device_pixel_ratio() as f32;
        world.renderer.canvas2d.scale(factor, factor);
        world.renderer.canvas2d.translate(-world.renderer.body.dimensions.x / (2.0 * factor), -world.renderer.body.dimensions.y / (2.0 * factor));

        world.renderer.canvas2d.save();
        world.renderer.canvas2d.reset_transform();
        GamePhase::render_game(world, delta_average, is_dead, dt);
        world.renderer.canvas2d.restore();

        world.renderer.body.render_children(&mut world.renderer.canvas2d);
        
        world.renderer.canvas2d.scale(0.5, 0.5);
        world.renderer.fps_counter.render(&mut world.renderer.canvas2d, world.renderer.body.dimensions);
        
        world.renderer.canvas2d.restore();
    }
}