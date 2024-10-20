use gloo::console::console;
use shared::{fuzzy_compare, lerp_angle, utils::vec2::Vector2D};

use crate::{canvas2d::{Canvas2d, Transform}, color::Color, core::{BoundingRect, Events, HoverEffects, Interpolatable, UiElement}, label::Label, DEBUG};

#[derive(Default)]
pub struct Button
{
    transform: Interpolatable<Transform>,
    fill: Interpolatable<Color>,
    stroke: f32,
    roundness: f32,
    dimensions: Interpolatable<Vector2D<f32>>,
    angle: Interpolatable<f32>,
    label: Label,
    events: Events,

    ticks: u64
}

impl UiElement for Button {
    fn get_mut_events(&mut self) -> &mut Events {
        &mut self.events
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform.value = transform.clone();
        self.label.set_transform(transform);
    }

    fn get_transform(&self) -> &Transform {
        &self.transform.value
    }

    fn set_hovering(&mut self, val: bool) {
        self.events.is_hovering = val;
        self.label.set_hovering(val);
    }

    fn get_bounding_rect(&self) -> BoundingRect {
        let mut position = -self.dimensions.value * (1.0 / 2.0);
        let mut dimensions = self.dimensions.value;

        self.transform.value.transform_point(&mut position);

        let scale = self.transform.value.get_scale();
        dimensions.x *= scale.x;
        dimensions.y *= scale.y;

        BoundingRect::new(
            position,
            dimensions
        )
    }
    
    fn render(&mut self, context: &mut Canvas2d) {
        self.ticks += 1;

        if self.events.is_hovering {
            if let Some(brightness) = self.events.hover_effects.iter().find_map(
            |item| match item {
                HoverEffects::AdjustBrightness(a) => Some(*a),
                _ => None,
            }) {
                let blender = Color::blend_colors(
                    Color::BLACK, 
                    Color::WHITE,
                    brightness
                );

                self.fill.target = Color::blend_colors(self.fill.original, blender, 0.3);
            }

            if let Some(inflation) = self.events.hover_effects.iter().find_map(
            |item| match item {
                HoverEffects::Inflation(a) => Some(*a),
                _ => None,
            }) {
                self.dimensions.target = self.dimensions.original * inflation;
            }

            if let Some((degrees, infinite)) = self.events.hover_effects.iter().find_map(
            |item| match item {
                HoverEffects::Shake(a, b) => Some((*a, *b)),
                _ => None,
            }) {
                if fuzzy_compare!(self.angle.value, self.angle.target, 1e-1) {
                    self.angle.direction *= -1.0;
                    if self.angle.direction == 1.0 && !infinite {
                        self.angle.target = 0.01;
                    }
                }
                
                if self.angle.target != 0.01 {
                    self.angle.target = (degrees * (std::f32::consts::PI / 180.0)) * self.angle.direction;
                } else {
                    self.angle.direction = 1.0;
                }
            }
        } else {
            self.fill.target = self.fill.original;
            self.dimensions.target = self.dimensions.original;
            self.angle.target = self.angle.original;
            // self.transform.target = self.transform.original.clone();
        }

        self.dimensions.value.lerp_towards(self.dimensions.target, 0.2);
        self.fill.value = *self.fill.value.blend_with(0.2, self.fill.target);
        self.angle.value = lerp_angle!(self.angle.value, self.angle.target, 0.25);
        self.transform.value.lerp_towards(&self.transform.target, 0.2);

        context.save();
        
        context.reset_transform();
        context.set_transform(&self.transform.value);
        context.rotate(self.angle.value);

        context.fill_style(self.fill.value);
        if self.stroke != 0.0 {
            let color = Color::blend_colors(
                self.fill.value, 
                Color::BLACK, 
                0.25
            );

            context.set_stroke_size(self.stroke);
            context.stroke_style(color);
        }

        let position = -self.dimensions.value * (1.0 / 2.0);

        context.begin_round_rect(
            position.x,
            position.y,
            self.dimensions.value.x,
            self.dimensions.value.y,
            self.roundness
        );

        context.fill();
        context.stroke();

        self.label.render(context);

        context.restore();

        if DEBUG {
            context.save();
            context.reset_transform();
            self.get_bounding_rect().render(context);
            context.restore();
        }
    }
}

impl Button {
    pub fn new() -> Button {
        Button::default()
    }

    pub fn with_transform(mut self, transform: Transform) -> Button {
        self.transform = Interpolatable::new(transform);
        self
    }

    pub fn with_angle(mut self, angle: f32) -> Button {
        self.angle = Interpolatable::new(angle);
        self
    }

    pub fn with_fill(mut self, fill: Color) -> Button {
        self.fill = Interpolatable::new(fill);
        self
    }

    pub fn with_stroke(mut self, stroke: f32) -> Button {
        self.stroke = stroke;
        self
    }

    pub fn with_roundness(mut self, roundness: f32) -> Button {
        self.roundness = roundness;
        self
    }

    pub fn with_label(mut self, label: Label) -> Button {
        self.label = label;
        self
    }

    pub fn with_dimensions(mut self, dimensions: Vector2D<f32>) -> Button {
        self.dimensions = Interpolatable::new(dimensions);
        self
    }

    pub fn with_events(mut self, events: Events) -> Button {
        self.events = events;
        self
    }
}