use std::{collections::VecDeque};

use gloo::console::console;
use gloo_utils::{document, window};
use shared::utils::vec2::Vector2D;
use ui::{button::Button, canvas2d::{Canvas2d, Transform}, color::Color, core::{Events, HoverEffects, UiElement}, label::Label};
use web_sys::{wasm_bindgen::{prelude::Closure, JsCast}, Performance};

use crate::world::{self, get_world, World};

use super::phases::{GamePhase, HomescreenElements};

#[derive(Default)]
pub struct TimeInformation {
    ticks: u32,
    last_render: f64,
    deltas: VecDeque<f64>
}

pub struct Renderer {
    pub canvas2d: Canvas2d,
    pub mouse: Vector2D<f32>,
    pub time: TimeInformation,
    pub phase: GamePhase
}

impl Renderer {
    pub fn new() -> Renderer {
        let canvas2d = Canvas2d::new(&document());

        Renderer {
            canvas2d,
            mouse: Vector2D::ZERO,
            time: TimeInformation::default(),
            phase: GamePhase::default()
        }
    }

    pub fn tick(world: &mut World, timestamp: f64) {
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
        
        match world.renderer.phase {
            GamePhase::Home(_) => Renderer::render_homescreen(world, delta_average),
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
        let context = &world.renderer.canvas2d;

        context.save();
        context.fill_style(Color(14, 14, 14));
        context.fill_rect(0, 0, context.get_width(), context.get_height());
        context.restore();

        let mut dimensions = context.get_dimensions();
        context.translate(dimensions.x / 2.0, dimensions.y / 2.0);

        let factor = (dimensions.x / 1920.0).max(dimensions.y / 1080.0);
        dimensions *= 1.0 / factor;

        context.scale(factor, factor);

        if world.renderer.time.ticks == 1 {
            HomescreenElements::setup(world);
        } else {
            HomescreenElements::render(world, delta_average);
        }
    }

    pub fn render_game(world: &mut World, delta_average: f64) {
    }
}