use shared::{fuzzy_compare, lerp, lerp_angle, utils::vec2::Vector2D};
use crate::{canvas2d::{Canvas2d, Transform}, color::Color, core::{BoundingRect, Events, HoverEffects, Interpolatable, UiElement}, DEBUG};

pub enum TextEffects {
    Typewriter(usize, u64) // Typewriter(char_index, tick_interval)
}

#[derive(Default)]
pub struct Label {
    transform: Transform,
    font: Interpolatable<f32>,
    angle: Interpolatable<f32>,
    fill: Color,
    stroke: Option<Color>,
    text: String,
    dimensions: Vector2D<f32>,
    effects: Option<TextEffects>,
    events: Events,

    ticks: u64,
}

impl UiElement for Label {
    fn get_mut_events(&mut self) -> &mut Events {
        &mut self.events
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn set_hovering(&mut self, val: bool) {
        self.events.is_hovering = val;
    }

    // Text labels are meant to not have children.
    fn get_mut_children(&mut self) -> Option<&mut Vec<Box<dyn UiElement>>> { None }
    fn set_children(&mut self, _: Vec<Box<dyn UiElement>>) {}

    fn get_bounding_rect(&self) -> BoundingRect {
        let mut position = -self.dimensions * (1.0 / 2.0);
        let mut dimensions = self.dimensions;

        self.transform.transform_point(&mut position);

        let scale = self.transform.get_scale();
        dimensions.x *= scale.x;
        dimensions.y *= scale.y;

        BoundingRect::new(
            position,
            dimensions
        )
    }

    fn render(&mut self, context: &mut Canvas2d) {
        self.ticks += 1;

        let mut char_index: isize = isize::MAX;
        if let Some(TextEffects::Typewriter(idx, interval)) = &mut self.effects {
            if self.ticks % *interval == 0 {
                *idx += 1;
            }

            char_index = *idx as isize;
        }

        let mut shake_lerp_factor = 0.25;

        if self.events.is_hovering {
            shake_lerp_factor = self.on_hover();
        } else {
            self.font.target = self.font.original;
            self.angle.target = self.angle.original;
        }

        self.font.value = lerp!(self.font.value, self.font.target, 0.2);
        self.angle.value = lerp_angle!(self.angle.value, self.angle.target, shake_lerp_factor);

        let text: Vec<_> = self.text.split("\n").collect();
        let stroke_size = self.stroke.map_or(0.0, |_| self.font.value / 5.0);
        let margin = self.font.value + stroke_size;

        let text_len = text.len();
        let half_up = text_len as f32 * margin / 2.0;
        let (mut width, mut height) = (0.0, 0.0);

        for (i, mut partial) in text.into_iter().enumerate() {
            char_index -= partial.len() as isize;
            if char_index < 0 {
                partial = &partial[0..(partial.len() - char_index.unsigned_abs())];
            }

            context.save();
            context.set_transform(&self.transform);

            if text_len != 1 {
                context.translate(
                    0.0,
                    ((i + 1) as f32 * margin) - half_up
                );
            }

            context.rotate(self.angle.value);
    
            context.set_miter_limit(2.0);
            context.fill_style(self.fill);
            context.set_font(&format!("bold {}px Ubuntu", self.font.value as u32));
            context.set_text_align("center");
    
            if stroke_size != 0.0 {
                context.stroke_style(self.stroke.unwrap());
                context.set_stroke_size(stroke_size as f64);
                context.stroke_text(partial);
            }
    
            let metrics = context.measure_text(partial);
            let text_width = metrics.width() as f32;
            let text_height = margin;
    
            if text_width > width {
                width = text_width;
            }

            height += text_height;
    
            context.fill_text(partial);
            context.restore();
    
            if DEBUG {
                self.get_bounding_rect().render(context);
            }

            if char_index < 0 {
                break;
            }
        }

        self.dimensions = Vector2D::new(width, height);
    }
}

impl Label {
    pub fn on_hover(&mut self) -> f32 {
        let mut slf = 0.25;

        if let Some(inflation) = self.events.hover_effects.iter().find_map(
        |item| match item {
            HoverEffects::Inflation(a) => Some(*a),
            _ => None,
        }) {
            self.font.target = self.font.original * inflation;
        }

        if let Some((degrees, infinite, lf)) = self.events.hover_effects.iter().find_map(
        |item| match item {
            HoverEffects::Shake(a, b, c) => Some((*a, *b, *c)),
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

            slf = lf;
        }

        slf
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

impl Label {
    pub fn new() -> Label {
        Label::default()
    }

    pub fn with_transform(mut self, transform: Transform) -> Label {
        self.transform = transform;
        self
    }

    pub fn with_font(mut self, font: f32) -> Label {
        self.font = Interpolatable::new(font);
        self
    }

    pub fn with_angle(mut self, angle: f32) -> Label {
        self.angle = Interpolatable::new(angle);
        self
    }

    pub fn with_fill(mut self, fill: Color) -> Label {
        self.fill = fill;
        self
    }

    pub fn with_stroke(mut self, stroke: Color) -> Label {
        self.stroke = Some(stroke);
        self
    }

    pub fn with_text(mut self, text: String) -> Label {
        self.text = text;
        self
    }

    pub fn with_dimensions(mut self, dimensions: Vector2D<f32>) -> Label {
        self.dimensions = dimensions;
        self
    }

    pub fn with_effects(mut self, effects: TextEffects) -> Label {
        self.effects = Some(effects);
        self
    }

    pub fn with_events(mut self, events: Events) -> Label {
        self.events = events;
        self
    }
}