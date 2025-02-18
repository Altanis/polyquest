use shared::{fuzzy_compare, lerp, utils::{color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use web_sys::MouseEvent;
use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, ElementType, Events, GenerateTranslationScript, UiElement}, DEBUG};

pub struct Tooltip {
    id: String,
    transform: Transform,
    dimensions: Vector2D,
    events: Events,
    z_index: i32,
    children: Vec<Box<dyn UiElement>>,
    is_animating: bool,
    opacity: Interpolatable<f32>,
    destroyed: bool,

    ticks: u64,
}

impl Default for Tooltip {
    fn default() -> Self {
        Self {
            id: String::default(),
            transform: Default::default(),
            dimensions: Default::default(),
            events: Default::default(),
            z_index: Default::default(),
            children: Default::default(),
            is_animating: Default::default(),
            opacity: Interpolatable::new(0.0),
            destroyed: Default::default(),
            ticks: Default::default(),
        }
    }
}

impl UiElement for Tooltip {
    fn get_identity(&self) -> ElementType {
        ElementType::Tooltip
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
        self.transform = transform;
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn set_opacity(&mut self, opacity: f32) {
        self.opacity.target = opacity;
    }

    fn get_z_index(&self) -> i32 {
        self.z_index
    }

    fn set_hovering(&mut self, _: bool, _: &MouseEvent) -> bool {
        false
    }

    fn set_clicked(&mut self, _: bool, _: &MouseEvent) {}

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

    fn set_children(&mut self, children: Vec<Box<dyn UiElement>>) {
        self.children = children;
    }

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

    fn render(&mut self, context: &mut Canvas2d, dimensions: Vector2D) -> bool {
        self.ticks += 1;
        self.is_animating = false;

        if let Some(t) = (self.transform.generate_translation)(dimensions) {
            self.transform.set_translation(t);
        }

        self.opacity.target = if self.destroyed { 0.0 } else { 1.0 };
        self.opacity.value = lerp!(self.opacity.value, self.opacity.target, 0.2);

        // TOOD: Make tooltips get a bounding rect of every child combined.
        if let Some(child) = self.children.first() {
            self.dimensions = child.get_bounding_rect().dimensions;
        }

        context.save();
        context.transform(&self.transform);
        context.global_alpha(self.opacity.value);

        context.save();
        context.global_alpha(self.opacity.value * 0.6);
        context.fill_style(Color::BLACK);
        context.begin_round_rect(-self.dimensions.x / 2.0, -self.dimensions.y / 2.0, self.dimensions.x, self.dimensions.y, 5.0);
        context.fill();
        context.restore();

        for child in self.children.iter_mut() {
            child.set_opacity(self.opacity.value);
            child.render(context, dimensions);
        }

        context.restore();
    
        if DEBUG {
            context.save();
            context.reset_transform();
            self.get_bounding_rect().render(context);
            context.restore();
        }

        let to_destroy = self.destroyed && fuzzy_compare!(self.opacity.value, 0.0, 1e-1);
        if to_destroy {
            self.destroyed = false;
            self.opacity.target = 1.0;
        }

        to_destroy
    }

    fn destroy(&mut self) {
        self.destroyed = true;
        self.opacity.target = 0.0;

        for child in self.children.iter_mut() {
            child.destroy();
        }
    }

    fn has_animation_state(&self) -> bool {
        self.is_animating || self.children.iter().any(|c| c.has_animation_state())
    }
}

impl Tooltip {
    pub fn new() -> Tooltip {
        Tooltip::default()
    }

    pub fn with_id(mut self, id: &str) -> Tooltip {
        self.id = id.to_string();
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Tooltip {
        self.transform = transform;
        self
    }

    pub fn with_translation(mut self, translation: Box<dyn GenerateTranslationScript>) -> Tooltip {
        self.transform.generate_translation = translation;
        self
    }

    pub fn with_dimensions(mut self, dimensions: Vector2D) -> Tooltip {
        self.dimensions = dimensions;
        self
    }

    pub fn with_events(mut self, events: Events) -> Tooltip {
        self.events = events;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Tooltip {
        self.z_index = z_index;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Tooltip {
        self.opacity = Interpolatable::new(opacity);
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn UiElement>>) -> Tooltip {
        self.children = children;
        self
    }
}