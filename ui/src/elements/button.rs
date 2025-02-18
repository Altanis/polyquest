use shared::{fuzzy_compare, lerp, lerp_angle, utils::{color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use web_sys::MouseEvent;
use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, DeletionEffects, ElementType, Events, GenerateTranslationScript, HoverEffects, UiElement}, DEBUG};

pub struct Button {
    id: String,
    transform: Interpolatable<Transform>,
    raw_transform: Transform,
    fill: Interpolatable<Color>,
    stroke: (f32, Option<Color>),
    dimensions: Interpolatable<Vector2D>,
    roundness: f32,
    angle: Interpolatable<f32>,
    events: Events,
    children: Vec<Box<dyn UiElement>>,
    z_index: i32,
    is_animating: bool,
    opacity: Interpolatable<f32>,
    destroyed: bool,

    ticks: u64
}

impl Default for Button {
    fn default() -> Self {
        Button {
            id: Default::default(),
            transform: Default::default(),
            raw_transform: Default::default(),
            fill: Default::default(),
            stroke: (-1.0, None),
            dimensions: Default::default(),
            roundness: 5.0,
            angle: Default::default(),
            events: Default::default(),
            children: Default::default(),
            z_index: Default::default(),
            is_animating: Default::default(),
            opacity: Interpolatable::new(1.0),
            destroyed: Default::default(),
            ticks: Default::default()
        }
    }
}

impl UiElement for Button {
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

    fn set_hovering(&mut self, val: bool, event: &MouseEvent) -> bool {
        self.events.is_hovering = val;
        for child in self.children.iter_mut() {
            child.set_hovering(val, event);
        }

        self.is_animating = true;

        val
    }

    fn set_clicked(&mut self, val: bool, _: &MouseEvent) {
        self.events.is_clicked = val;
    }

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
            self.fill.target = self.fill.original;
            self.dimensions.target = self.dimensions.original;
            self.angle.target = self.angle.original;
            self.opacity.target = self.opacity.original;

            if !self.fill.value.partial_eq(self.fill.target, 5.0)
                || !self.dimensions.value.partial_eq(self.dimensions.target, 1e-1)
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
        self.fill.value = *self.fill.value.blend_with(0.2, self.fill.target);
        self.angle.value = lerp_angle!(self.angle.value, self.angle.target, shake_lerp_factor);
        self.transform.value.lerp_towards(&self.transform.target, 0.2);
        self.opacity.value = lerp!(self.opacity.value, self.opacity.target, 0.2);

        context.save();
        context.transform(&self.transform.value);
        context.rotate(self.angle.value);
        context.global_alpha(self.opacity.value);

        self.raw_transform = context.get_transform();

        context.fill_style(self.fill.value);

        let stroke = if self.stroke.0 < 0.0 { self.dimensions.value.min().max(1.0).ln() * 1.5 } else { self.stroke.0 };
        if stroke != 0.0 {
            let color = if let Some(color) = self.stroke.1 {
                color
            } else {
                Color::blend_colors(
                    self.fill.value, 
                    Color::BLACK, 
                    0.25
                )
            };

            context.set_stroke_size(stroke);
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

        for child in self.children.iter_mut() {
            child.render(context, dimensions);
        }

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

impl Button {
    fn on_click(&mut self) {
        self.events.is_clicked = false;
        if let Some(click_fn) = &self.events.on_click {
            (click_fn)(Box::new(self));
        }
    }

    fn on_hover(&mut self) -> f32 {
        let mut slf = 0.25;

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
            self.is_animating = true;
        }

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


impl Button {
    pub fn new() -> Button {
        Button::default()
    }

    pub fn with_id(mut self, id: &str) -> Button {
        self.id = id.to_string();
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Button {
        self.transform = Interpolatable::new(transform);
        self
    }

    pub fn with_translation(mut self, translation: Box<dyn GenerateTranslationScript>) -> Button {
        self.transform.value.generate_translation = translation;
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

    pub fn with_stroke(mut self, stroke: (f32, Option<Color>)) -> Button {
        self.stroke = stroke;
        self
    }

    pub fn with_dimensions(mut self, dimensions: Vector2D) -> Button {
        self.dimensions = Interpolatable::new(dimensions);
        self
    }

    pub fn with_roundness(mut self, roundness: f32) -> Button {
        self.roundness = roundness;
        self
    }

    pub fn with_events(mut self, events: Events) -> Button {
        self.events = events;
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn UiElement>>) -> Button {
        self.children = children;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Button {
        self.z_index = z_index;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Button {
        self.opacity = Interpolatable::new(opacity);
        self
    }
}