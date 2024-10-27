use std::{collections::VecDeque};

use gloo::console::console;
use gloo_utils::{document, window};
use shared::utils::vec2::Vector2D;
use ui::{canvas2d::{Canvas2d, Transform}, core::{Events, HoverEffects, UiElement}, elements::{body::Body, button::Button, label::{Label, TextEffects}}, gl::webgl::WebGl, translate, utils::{color::Color, sound::Sound}};
use web_sys::{wasm_bindgen::{prelude::Closure, JsCast}, Performance};

use crate::world::{self, get_world, World};

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
            body: Body::default()
                .with_fill(Color(14, 14, 14))
                .with_rendering_script(GamePhase::render_homescreen),
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
        if !world.soundtrack.is_playing() {
            world.soundtrack.play();
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

        match world.renderer.phase {
            GamePhase::Home(_) => Renderer::render_homescreen(world, delta_average),
            GamePhase::Game => Renderer::render_game(world, delta_average),
            _ => ()
        }

        world.renderer.canvas2d.restore();
        world.renderer.gl.render(
            &world.renderer.canvas2d, 
            (timestamp - world.renderer.time.start_time) / 1000.0
        );

        let closure = Closure::once(move |ts: f64| {
            Renderer::tick(get_world(), ts);
        });

        let _ = window()
            .request_animation_frame(closure.as_ref().unchecked_ref());

        closure.forget();
    }

    pub fn render_homescreen(world: &mut World, delta_average: f64) {
        let context = &mut world.renderer.canvas2d;

        if world.renderer.body.get_mut_children().unwrap().is_empty() {
            world.renderer.body.set_children(GamePhase::generate_homescreen_elements());
        }

        Renderer::render_ui(world, delta_average);
    }

    pub fn render_game(world: &mut World, delta_average: f64) {
    }

    pub fn render_ui(world: &mut World, delta_average: f64) {
        let context = &mut world.renderer.canvas2d;

        world.renderer.body.render(context);
        world.renderer.fps_counter.render(context);

        world.renderer.fps_counter.set_text(format!("{:.1} FPS", 1000.0 / delta_average));
    }
}