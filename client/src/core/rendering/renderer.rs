use std::collections::VecDeque;

use gloo::console::console;
use gloo_utils::{document, window};
use shared::utils::vec2::Vector2D;
use ui::{canvas2d::Canvas2d, color::Color, label::{Alignment, Label}, UiElement};
use web_sys::{wasm_bindgen::{prelude::Closure, JsCast}, Performance};

use crate::world::{get_world, World};

#[derive(Default)]
pub struct TimeInformation {
    ticks: u32,
    last_render: f64,
    deltas: VecDeque<f64>
}

pub enum GamePhase {
    Home,
    Game,
    Death
}

pub struct Renderer {
    pub canvas2d: Canvas2d,
    pub ui_elements: Vec<Box<dyn UiElement>>,
    pub mouse: Vector2D<u32>,
    pub time: TimeInformation,
    pub phase: GamePhase
}

impl Renderer {
    pub fn new() -> Renderer {
        let canvas2d = Canvas2d::new(&document());

        Renderer {
            canvas2d,
            ui_elements: Vec::new(),
            mouse: Vector2D::INTEGER_ZERO,
            time: TimeInformation::default(),
            phase: GamePhase::Home
        }
    }

    pub fn tick(world: &mut World, timestamp: f64) {
        let delta_average = {
            let time = &mut world.renderer.time;
            time.ticks += 1;
        
            // let timestamp = window().performance().unwrap().now();
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
        
        match world.renderer.phase {
            GamePhase::Home => Renderer::render_homescreen(world, delta_average),
            GamePhase::Game => Renderer::render_game(world, delta_average),
            _ => ()
        }

        world.renderer.canvas2d.restore();

        let closure = Closure::once(move |ts: f64| {
            Renderer::tick(get_world(), ts);
        });

        let _ = window()
            .request_animation_frame(closure.as_ref().unchecked_ref());

        closure.forget();
    }

    pub fn render_homescreen(world: &mut World, delta_average: f64) {
        let mut dimensions = world.renderer.canvas2d.get_dimensions().to_float();
        world.renderer.canvas2d.translate(dimensions.x / 2.0, dimensions.y / 2.0);

        let factor = (dimensions.x / 1920.0).max(dimensions.y / 1080.0);
        dimensions *= 1.0 / factor;

        world.renderer.canvas2d.scale(factor, factor);

        if world.renderer.ui_elements.is_empty() {
            let label = Label::new()
                .align(Alignment::Center)
                .fill((108, Color::BLACK))
                .stroke(Some(Color(255, 0, 0)))
                .text("Hello world.".to_string());

            world.renderer.ui_elements.push(Box::new(label));
        }

        for element in world.renderer.ui_elements.iter() {
            element.render(&mut world.renderer.canvas2d);
        }
    }

    pub fn render_game(world: &mut World, delta_average: f64) {
    }
}