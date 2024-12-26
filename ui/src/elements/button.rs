use shared::{fuzzy_compare, lerp_angle, utils::{interpolatable::Interpolatable, vec2::Vector2D}};
use web_sys::MouseEvent;
use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, ElementType, Events, GenerateTranslationScript, HoverEffects, UiElement}, utils::color::Color, DEBUG};

#[derive(Default)]
pub struct Button
{
    id: String,
    transform: Interpolatable<Transform>,
    raw_transform: Transform,
    fill: Interpolatable<Color>,
    dimensions: Interpolatable<Vector2D<f32>>,
    angle: Interpolatable<f32>,
    events: Events,
    children: Vec<Box<dyn UiElement>>,
    z_index: i32,

    ticks: u64
}

impl UiElement for Button {
    fn get_identity(&self) -> crate::core::ElementType {
        ElementType::Button    
    }

    fn get_id(&self) -> String {
        self.id.clone()
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
    
    fn render(&mut self, context: &mut Canvas2d, dimensions: Vector2D<f32>) -> bool {
        self.ticks += 1;

        let mut shake_lerp_factor = 0.25;

        if self.events.is_hovering {
            shake_lerp_factor = self.on_hover();
        } else {
            self.fill.target = self.fill.original;
            self.dimensions.target = self.dimensions.original;
            self.angle.target = self.angle.original;
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

        context.save();
        context.set_transform(&self.transform.value);
        context.rotate(self.angle.value);

        self.raw_transform = context.get_transform();

        context.fill_style(self.fill.value);

        let stroke = self.dimensions.value.min() / 10.0;
        if stroke != 0.0 {
            let color = Color::blend_colors(
                self.fill.value, 
                Color::BLACK, 
                0.25
            );

            context.set_stroke_size(stroke);
            context.stroke_style(color);
        }

        let position = -self.dimensions.value * (1.0 / 2.0);

        context.begin_round_rect(
            position.x,
            position.y,
            self.dimensions.value.x,
            self.dimensions.value.y,
            5.0
        );

        context.fill();
        context.stroke();

        for child in self.children.iter_mut() {
            child.render(context, dimensions);
        }

        context.restore();

        if DEBUG {
            context.save();
            context.reset_transform();
            self.get_bounding_rect().render(context);
            context.restore();
        }

        false
    }

    fn destroy(&mut self) {}
}

impl Button {
    fn on_click(&mut self) {
        self.events.is_clicked = false;
        if let Some(click_fn) = &self.events.on_click {
            (click_fn)();
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
        }

        if let Some(inflation) = self.events.hover_effects.iter().find_map(
        |item| match item {
            HoverEffects::Inflation(a) => Some(*a),
            _ => None,
        }) {
            self.dimensions.target = self.dimensions.original * inflation;
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

    pub fn with_dimensions(mut self, dimensions: Vector2D<f32>) -> Button {
        self.dimensions = Interpolatable::new(dimensions);
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
}