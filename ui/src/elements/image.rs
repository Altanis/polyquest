use shared::{fuzzy_compare, lerp, lerp_angle, utils::{interpolatable::Interpolatable, vec2::Vector2D}};
use web_sys::{HtmlImageElement, MouseEvent};
use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, DeletionEffects, ElementType, Events, GenerateTranslationScript, HoverEffects, UiElement}, DEBUG};

pub struct Image {
    id: String,
    transform: Interpolatable<Transform>,
    raw_transform: Transform,
    dimensions: Interpolatable<Vector2D>,
    angle: Interpolatable<f32>,
    events: Events,
    z_index: i32,
    is_animating: bool,
    opacity: Interpolatable<f32>,
    destroyed: bool,
    children: Vec<Box<dyn UiElement>>,
    image: HtmlImageElement,

    ticks: u64
}

impl Default for Image {
    fn default() -> Self {
        Image {
            id: Default::default(),
            transform: Default::default(),
            raw_transform: Default::default(),
            dimensions: Default::default(),
            angle: Default::default(),
            events: Default::default(),
            z_index: Default::default(),
            is_animating: Default::default(),
            opacity: Interpolatable::new(1.0),
            destroyed: Default::default(),
            children: vec![],
            image: HtmlImageElement::new().unwrap(),
            ticks: Default::default()
        }
    }
}

impl UiElement for Image {
    fn get_identity(&self) -> crate::core::ElementType {
        ElementType::Button    
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
        self.transform.value = transform.clone();
    }

    fn get_transform(&self) -> &Transform {
        &self.transform.value
    }

    fn set_opacity(&mut self, opacity: f32) {
        self.opacity.target = opacity;
    }
    
    fn get_z_index(&self) -> i32 {
        self.z_index
    }

    fn set_hovering(&mut self, val: bool, _: &MouseEvent) -> bool {
        self.events.is_hovering = val;
        val
    }

    fn set_clicked(&mut self, val: bool, _: &MouseEvent) {
        self.events.is_clicked = val;
    }

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
        let mut position = -self.dimensions.value * (1.0 / 2.0);
        let mut dimensions = self.dimensions.value;

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
        self.ticks += 1;
        self.is_animating = false;

        let mut shake_lerp_factor = 0.25;

        if self.events.is_hovering {
            shake_lerp_factor = self.on_hover();

            for child in self.events.hovering_elements.iter_mut() {
                if self.events.is_hovering {
                    child.render(context, dimensions);
                } else {
                    child.destroy();
                }
            }
        } else if !self.destroyed {
            self.dimensions.target = self.dimensions.original;
            self.angle.target = self.angle.original;
            self.opacity.target = self.opacity.original;

            if !self.dimensions.value.partial_eq(self.dimensions.target, 1e-1)
                || !fuzzy_compare!(self.angle.value, self.angle.target, 1e-1)
                || !fuzzy_compare!(self.opacity.value, self.opacity.target, 1e-1) {
                    self.is_animating = true;
                }
        }

        if self.events.is_clicked {
            self.on_click();
        }

        if let Some(t) = (self.transform.value.generate_translation)(dimensions) {
            self.transform.value.set_translation(t);
        }

        self.dimensions.value.lerp_towards(self.dimensions.target, 0.2);
        self.angle.value = lerp_angle!(self.angle.value, self.angle.target, shake_lerp_factor);
        self.transform.value.lerp_towards(&self.transform.target, 0.2);
        self.opacity.value = lerp!(self.opacity.value, self.opacity.target, 0.2);

        context.save();
        context.set_transform(&self.transform.value);
        context.rotate(self.angle.value);
        context.global_alpha(self.opacity.value);

        self.raw_transform = context.get_transform();

        let position = -self.dimensions.value * (1.0 / 2.0);
        context.draw_image(&self.image, position.x, position.y, self.dimensions.value.x, self.dimensions.value.y);

        context.fill();
        context.stroke();

        if let Some(tooltip) = &mut self.events.tooltip {
            if !self.events.is_hovering {
                tooltip.destroy();
            }

            tooltip.render(context, dimensions);
        }

        context.restore();

        if DEBUG {
            context.save();
            context.reset_transform();
            self.get_bounding_rect().render(context);
            context.restore();
        }

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

        for child in self.events.hovering_elements.iter_mut() {
            child.destroy();
        }

        if let Some(tooltip) = &mut self.events.tooltip {
            tooltip.destroy();
        }
    }

    fn has_animation_state(&self) -> bool {
        self.is_animating
    }
}

impl Image {
    fn on_click(&mut self) {
        self.events.is_clicked = false;
        if let Some(click_fn) = &self.events.on_click {
            (click_fn)(Box::new(self));
        }
    }

    fn on_hover(&mut self) -> f32 {
        let mut slf = 0.25;

        if let Some(inflation) = self.events.hover_effects.iter().find_map(
        |item| match item {
            HoverEffects::Inflation(a) => Some(*a),
            _ => None,
        }) {
            self.dimensions.target = self.dimensions.original * inflation;
            self.is_animating = true;
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

                self.is_animating = true;
            }
            
            if self.angle.target != 0.01 {
                self.angle.target = (degrees * (std::f32::consts::PI / 180.0)) * self.angle.direction;
            } else {
                self.angle.direction = 1.0;
            }

            slf = lf;
        }

        if let Some(hover_opacity) = self.events.hover_effects.iter().find_map(
            |item| match item {
                HoverEffects::Opacity(a) => Some(*a),
                _ => None,
            })
        {
            self.opacity.target = hover_opacity;
        }

        slf
    }
}


impl Image {
    pub fn new() -> Image {
        Image::default()
    }

    pub fn with_id(mut self, id: &str) -> Image {
        self.id = id.to_string();
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Image {
        self.transform = Interpolatable::new(transform);
        self
    }

    pub fn with_translation(mut self, translation: Box<dyn GenerateTranslationScript>) -> Image {
        self.transform.value.generate_translation = translation;
        self
    }

    pub fn with_angle(mut self, angle: f32) -> Image {
        self.angle = Interpolatable::new(angle);
        self
    }

    pub fn with_dimensions(mut self, dimensions: Vector2D) -> Image {
        self.dimensions = Interpolatable::new(dimensions);
        self
    }

    pub fn with_events(mut self, events: Events) -> Image {
        self.events = events;
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn UiElement>>) -> Image {
        self.children = children;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Image {
        self.z_index = z_index;
        self
    }

    pub fn with_image_url(self, image_url: &str) -> Image {
        self.image.set_src(image_url);
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Image {
        self.opacity = Interpolatable::new(opacity);
        self
    }
}