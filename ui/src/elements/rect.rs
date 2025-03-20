use gloo::{console::console, utils::window};
use shared::{fuzzy_compare, lerp, utils::{color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use web_sys::MouseEvent;

use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, DeletionEffects, ElementType, Events, UiElement}};

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
    dimensions: Vector2D,
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

    fn get_events(&self) -> &Events {
        &self.events
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

    fn set_opacity(&mut self, opacity: f32) {
        self.opacity.target = opacity;
    }

    fn get_z_index(&self) -> i32 {
        -1
    }

    fn set_hovering(&mut self, val: bool, event: &MouseEvent) -> bool {
        self.events.is_hovering = val;

        let mut is_hovering = false;
        let mut point = Vector2D::new(event.client_x() as f32, event.client_y() as f32);
        point *= window().device_pixel_ratio() as f32;
    
        for ui_element in self.get_mut_children().iter_mut() {
            let hovering = ui_element.get_mut_events().hoverable &&
                ui_element.get_bounding_rect().intersects(point);
    
            let should_hover = ui_element.set_hovering(hovering, event);
    
            if !is_hovering && should_hover {
                is_hovering = should_hover;
            }
        }
    
        is_hovering
    }
    
    fn set_clicked(&mut self, _: bool, event: &MouseEvent) {
        let mut point = Vector2D::new(event.client_x() as f32, event.client_y() as f32);
        point *= window().device_pixel_ratio() as f32;
    
        for ui_element in self.get_mut_children().iter_mut() {
            let hovering = ui_element.get_mut_events().hoverable &&
                ui_element.get_bounding_rect().intersects(point);
    
            ui_element.set_clicked(hovering, event);
        }
    }

    fn get_mut_children(&mut self) -> &mut Vec<Box<dyn UiElement>> {
        self.children.sort_by(|a, b| {
            let z_index_cmp = a.get_z_index().cmp(&b.get_z_index());
            if z_index_cmp == std::cmp::Ordering::Equal {
                let a_hovering = a.get_events().is_hovering;
                let b_hovering = b.get_events().is_hovering;
                a_hovering.cmp(&b_hovering)
            } else {
                z_index_cmp
            }
        });

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

    fn render(&mut self, context: &mut Canvas2d, dimensions: Vector2D) -> bool {
        context.save();
        context.transform(&self.transform);
        context.fill_style(self.fill);
        if self.stroke != 0.0 {
            context.stroke_style(Color::blend_colors(self.fill, Color::BLACK, 0.2));
            context.set_stroke_size(self.stroke);
        }

        self.opacity.value = lerp!(self.opacity.value, self.opacity.target, 0.2);
        context.global_alpha(self.opacity.value);
        
        self.raw_transform = context.get_transform();

        context.begin_round_rect(0.0, 0.0, self.dimensions.x, self.dimensions.y, self.roundness);
        context.fill();
        context.stroke();

        for child in self.get_mut_children().iter_mut() {
            child.render(context, dimensions);
        }

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
        self.children.iter().any(|e| e.has_animation_state())
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

    pub fn with_dimensions(mut self, dimensions: Vector2D) -> Rect {
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