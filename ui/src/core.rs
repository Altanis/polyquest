use shared::utils::vec2::Vector2D;
use crate::{canvas2d::{Canvas2d, Transform}, color::Color};

pub trait UiElement {
    fn get_mut_events(&mut self) -> &mut Events;

    fn set_transform(&mut self, transform: Transform);
    fn get_transform(&self) -> &Transform;

    fn get_mut_children(&mut self) -> Option<&mut Vec<Box<dyn UiElement>>>;
    fn set_children(&mut self, children: Vec<Box<dyn UiElement>>);

    fn set_hovering(&mut self, val: bool);

    fn get_bounding_rect(&self) -> BoundingRect;
    fn render(&mut self, context: &mut Canvas2d);
}

#[derive(Debug, Clone)]
pub struct BoundingRect {
    pub position: Vector2D<f32>,
    pub dimensions: Vector2D<f32>
}

impl BoundingRect {
    pub fn new(position: Vector2D<f32>, dimensions: Vector2D<f32>) -> BoundingRect {
        BoundingRect {
            position,
            dimensions
        }
    }

    pub fn intersects(&self, point: Vector2D<f32>) -> bool {
        let within_x = point.x >= self.position.x && point.x <= self.position.x + self.dimensions.x;
        let within_y = point.y >= self.position.y && point.y <= self.position.y + self.dimensions.y;
        within_x && within_y
    }

    pub fn render(&self, context: &mut Canvas2d) {
        context.save();
        context.translate(self.position.x, self.position.y);
        context.fill_style(Color(255, 0, 0));
        context.stroke_style(Color(255, 0, 0));

        context.begin_arc(0.0, 0.0, 5.0, std::f64::consts::TAU);
        context.fill();

        context.stroke_rect(0.0, 0.0, self.dimensions.x, self.dimensions.y);

        context.restore();
    }
}

pub struct Events {
    pub hoverable: bool,
    pub is_hovering: bool,
    pub hover_effects: Vec<HoverEffects>,
    pub on_click: fn()
}

impl Default for Events {
    fn default() -> Events {
        Events {
            hoverable: true,
            is_hovering: false,
            hover_effects: Vec::new(),
            on_click: || {}
        }
    }
}

impl Events {
    pub fn with_hoverable(hoverable: bool) -> Events {
        Events {
            hoverable,
            is_hovering: false,
            hover_effects: Vec::new(),
            on_click: || {}
        }
    }

    pub fn with_hover_effects(effects: Vec<HoverEffects>) -> Events {
        Events {
            hoverable: true,
            is_hovering: false,
            hover_effects: effects,
            on_click: || {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HoverEffects {
    Inflation(f32), // Inflation(blowup_factor)
    AdjustBrightness(f32), // AdjustBrightness(brightness)
    Shake(f32, bool, f32), // Shake(+/- angle, infinite, factor)
}

pub struct Interpolatable<T: Default + Clone> {
    pub original: T,
    pub value: T,
    pub target: T,
    pub direction: f32
}

impl<T: Default + Clone> Default for Interpolatable<T> {
    fn default() -> Self {
        Interpolatable {
            original: T::default(),
            value: T::default(),
            target: T::default(),
            direction: 1.0
        }
    }
}

impl<T: Default + Clone> Interpolatable<T> {
    pub fn new(value: T) -> Self {
        Self {
            original: value.clone(),
            target: value.clone(),
            value,
            direction: 1.0
        }
    }
}

pub type RenderingScript = Box<dyn FnMut(&mut Canvas2d)>;