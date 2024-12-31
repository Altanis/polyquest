use gloo::console::console;
use shared::{fuzzy_compare, lerp, utils::{interpolatable::Interpolatable, vec2::Vector2D}};
use web_sys::MouseEvent;

use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, DeletionEffects, ElementType, Events, UiElement}, utils::color::Color};

#[derive(Default)]
pub struct Rect {
    id: String,
    transform: Transform,
    raw_transform: Transform,
    fill: Color,
    stroke: f32,
    roundness: f32,
    events: Events,
    children: Vec<Box<dyn UiElement>>,
    dimensions: Vector2D<f32>,
    destroyed: bool,
    opacity: Interpolatable<f32>
}


impl UiElement for Rect {
    fn get_identity(&self) -> ElementType {
        ElementType::Rect
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_mut_events(&mut self) -> &mut Events {
        &mut self.events
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform.clone();
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_z_index(&self) -> i32 {
        -1
    }

    fn set_hovering(&mut self, _: bool, _: &MouseEvent) -> bool {
        false
    }
    
    fn set_clicked(&mut self, _: bool, _: &MouseEvent) {}

    fn get_mut_children(&mut self) -> &mut Vec<Box<dyn UiElement>> {
        &mut self.children
    }

    fn get_element_by_id(&mut self, _: &str) -> Option<(usize, &mut Box<dyn UiElement>)> {
        None
    }

    fn delete_element_by_id(&mut self, _: &str, _: bool) {}

    fn set_children(&mut self, children: Vec<Box<dyn UiElement>>) {
        self.children = children;
    }

    fn get_bounding_rect(&self) -> BoundingRect {
        let mut position = -self.dimensions * (1.0 / 2.0);
        let mut dimensions = self.dimensions;

        self.raw_transform.transform_point(&mut position);

        let scale = self.raw_transform.get_scale();
        dimensions.x *= scale.x;
        dimensions.y *= scale.y;

        BoundingRect::new(
            position,
            dimensions
        )
    }

    fn render(&mut self, context: &mut Canvas2d, _: Vector2D<f32>) -> bool {
        context.save();
        context.set_transform(&self.transform);
        context.fill_style(self.fill);
        if self.stroke != 0.0 {
            context.stroke_style(Color::blend_colors(self.fill, Color::BLACK, 0.2));
            context.set_stroke_size(self.stroke);
        }

        self.opacity.value = lerp!(self.opacity.value, self.opacity.target, 0.2);
        context.global_alpha(self.opacity.value as f64);
        
        self.raw_transform = context.get_transform();

        context.begin_round_rect(0.0, 0.0, self.dimensions.x as f64, self.dimensions.y as f64, self.roundness as f64);
        context.fill();
        context.stroke();

        context.restore();

        self.destroyed && fuzzy_compare!(self.opacity.value, self.opacity.target, 1e-1)
    }
    
    fn destroy(&mut self) {
        self.destroyed = true;
        if self.events.deletion_effects.contains(&DeletionEffects::Disappear) {
            self.opacity.target = 0.0;
        }

        for child in self.children.iter_mut() {
            child.destroy();
        }
    }

    fn has_animation_state(&self) -> bool {
        false
    }
}

impl Rect {
    pub fn new() -> Rect {
        Rect::default()
    }

    pub fn with_id(mut self, id: &str) -> Rect {
        self.id = id.to_string();
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Rect {
        self.transform = transform;
        self
    }

    pub fn with_fill(mut self, fill: Color) -> Rect {
        self.fill = fill;
        self
    }

    pub fn with_stroke(mut self, stroke: f32) -> Rect {
        self.stroke = stroke;
        self
    }

    pub fn with_roundness(mut self, roundness: f32) -> Rect {
        self.roundness = roundness;
        self
    }

    pub fn with_dimensions(mut self, dimensions: Vector2D<f32>) -> Rect {
        self.dimensions = dimensions;
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn UiElement>>) -> Rect {
        self.children = children;
        self
    }

    pub fn with_events(mut self, events: Events) -> Rect {
        self.events = events;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Rect {
        self.opacity = Interpolatable::new(opacity);
        self
    }
}