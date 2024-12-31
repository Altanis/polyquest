use gloo::console::console;
use shared::{fuzzy_compare, lerp, utils::{interpolatable::Interpolatable, vec2::Vector2D}};
use web_sys::{MouseEvent, UiEvent};

use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, DeletionEffects, ElementType, Events, HoverEffects, UiElement}, utils::color::Color, DEBUG};

/// This element is used to render tanks with specific bodies and weaponry.
pub struct Tank {
    id: String,
    transform: Transform,
    raw_transform: Transform,
    fill: Color,
    radius: f32,
    z_index: i32,
    children: Vec<Box<dyn UiElement>>,
    events: Events,
    is_animating: bool,
    opacity: Interpolatable<f32>,
    destroyed: bool,
    ticks: u64
}

impl Default for Tank {
    fn default() -> Self {
        Self {
            id: String::default(),
            transform: Default::default(),
            raw_transform: Default::default(),
            fill: Color::default(),
            radius: Default::default(),
            z_index: Default::default(),
            children: Default::default(),
            events: Default::default(),
            is_animating: Default::default(),
            opacity: Interpolatable::new(1.0),
            destroyed: Default::default(),
            ticks: Default::default(),
        }
    }
}

impl UiElement for Tank {
    fn get_identity(&self) -> ElementType {
        ElementType::Tank
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_mut_events(&mut self) -> &mut Events {
        &mut self.events
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn get_z_index(&self) -> i32 {
        self.z_index
    }

    fn set_hovering(&mut self, val: bool, event: &MouseEvent) -> bool {
        self.events.is_hovering = val;
        for child in self.children.iter_mut() {
            child.set_hovering(val, event);
        }

        val
    }

    fn set_clicked(&mut self, val: bool, event: &MouseEvent) {
        self.events.is_clicked = val;
    }

    // Checkboxes are not meant to have children.
    fn get_mut_children(&mut self) -> &mut Vec<Box<dyn UiElement>> {
        &mut self.children
    }

    fn get_element_by_id(&mut self, id: &str) -> Option<(usize, &mut Box<dyn UiElement>)> {
        self.children
            .iter_mut()
            .enumerate()
            .find(|(_, child)| child.get_id() == id)
    }

    fn delete_element_by_id(&mut self, id: &str, destroy: bool) {
        if let Some((i, child)) = self.children
            .iter_mut()
            .enumerate()
            .find(|(_, child)| child.get_id() == id) 
        {
            if destroy {
                child.destroy();
            } else {
                self.children.remove(i);
            }
        }
    }

    fn set_children(&mut self, _: Vec<Box<dyn UiElement>>) {}

    fn get_bounding_rect(&self) -> BoundingRect {
        let mut position = Vector2D::from_scalar(-self.radius);
        let mut dimensions = Vector2D::from_scalar(self.radius * 2.0);

        self.raw_transform.transform_point(&mut position);

        let scale = self.raw_transform.get_scale();
        dimensions.x *= scale.x;
        dimensions.y *= scale.y;

        BoundingRect::new(
            position,
            dimensions
        )
    }

    fn render(&mut self, context: &mut Canvas2d, dimensions: Vector2D<f32>) -> bool {
        self.ticks += 1;
        self.is_animating = false;

        if let Some(t) = (self.transform.generate_translation)(dimensions) {
            self.transform.set_translation(t);
        }

        if self.events.is_hovering {
            self.on_hover();
        } else if !self.destroyed {
            self.opacity.target = self.opacity.original;
            if !fuzzy_compare!(self.opacity.value, self.opacity.target, 1e-1) {
                self.is_animating = true;
            }
        }

        self.opacity.value = lerp!(self.opacity.value, self.opacity.target, 0.2);

        context.save();
        context.set_transform(&self.transform);
        self.raw_transform = context.get_transform();
        context.global_alpha(self.opacity.value as f64);
        
        if DEBUG {
            context.save();
            context.reset_transform();
            self.get_bounding_rect().render(context);
            context.restore();
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
        self.is_animating
    }
}

impl Tank {
    pub fn new() -> Tank {
        Tank::default()
    }
    
    pub fn on_hover(&mut self) {
        if let Some(hover_opacity) = self.events.hover_effects.iter().find_map(
            |item| match item {
                HoverEffects::Opacity(a) => Some(*a),
                _ => None,
            })
        {
            self.opacity.target = hover_opacity;
        }
    }

    pub fn with_id(mut self, id: &str) -> Tank {
        self.id = id.to_string();
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Tank {
        self.transform = transform;
        self
    }

    pub fn with_raw_transform(mut self, raw_transform: Transform) -> Tank {
        self.raw_transform = raw_transform;
        self
    }

    pub fn with_fill(mut self, fill: Color) -> Tank {
        self.fill = fill;
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Tank {
        self.radius = radius;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Tank {
        self.z_index = z_index;
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn UiElement>>) -> Tank {
        self.children = children;
        self
    }

    pub fn with_events(mut self, events: Events) -> Tank {
        self.events = events;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Tank {
        self.opacity = Interpolatable::new(opacity);
        self
    }
}