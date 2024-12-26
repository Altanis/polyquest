use gloo::console::console;
use shared::utils::{interpolatable::Interpolatable, vec2::Vector2D};
use web_sys::MouseEvent;

use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, ElementType, Events, GenerateTranslationScript, UiElement}, utils::color::Color, DEBUG};

#[derive(Default)]
pub struct Checkbox {
    id: String,

    transform: Transform,
    raw_transform: Transform,

    fill: Interpolatable<Color>,
    accent: Color,
    box_stroke: (f32, Color),
    dimensions: Interpolatable<Vector2D<f32>>,
    z_index: i32,

    value: bool,

    children: Vec<Box<dyn UiElement>>,
    events: Events,

    ticks: u64,
}

impl UiElement for Checkbox {
    fn get_identity(&self) -> crate::core::ElementType {
        ElementType::Checkbox    
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

    fn set_hovering(&mut self, val: bool, _: &MouseEvent) -> bool {
        self.events.is_hovering = val;
        val
    }

    fn set_clicked(&mut self, val: bool, _: &MouseEvent) {
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

    fn set_children(&mut self, _: Vec<Box<dyn UiElement>>) {}

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

        if self.events.is_hovering {
            self.on_hover();
        } else {
            self.fill.target = self.fill.original;
            self.dimensions.target = self.dimensions.original;
        }

        if self.events.is_clicked {
            self.on_click();
        }

        if let Some(t) = (self.transform.generate_translation)(dimensions) {
            self.transform.set_translation(t);
        }

        self.dimensions.value.lerp_towards(self.dimensions.target, 0.2);
        self.fill.value = *self.fill.value.blend_with(0.2, self.fill.target);

        context.save();
        context.set_transform(&self.transform);
        self.raw_transform = context.get_transform();

        context.fill_style(self.fill.value);
        context.set_stroke_size(self.box_stroke.0);
        context.stroke_style(self.box_stroke.1);

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

        if self.value {
            context.fill_style(self.accent);
            context.set_stroke_size(0.0);
        
            // todo: interpolate this
            let checkmark_offset = 5.0;
            let inner_position_x = position.x + checkmark_offset;
            let inner_position_y = position.y + checkmark_offset;
            let inner_width = self.dimensions.value.x - checkmark_offset * 2.0;
            let inner_height = self.dimensions.value.y - checkmark_offset * 2.0;
        
            context.begin_rect(inner_position_x, inner_position_y, inner_width, inner_height);
            context.fill();
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

impl Checkbox {
    pub fn on_hover(&mut self) {
        self.fill.target = Color::blend_colors(self.fill.original, Color::BLACK, 0.3);
        self.dimensions.target = self.dimensions.original * 1.1;
    }

    pub fn on_click(&mut self) {
        self.value = !self.value;
        self.events.is_clicked = false;

        if let Some(script) = &self.events.on_click {
            (script)();
        }
    }
}

impl Checkbox {
    pub fn new() -> Checkbox {
        Checkbox::default()
    }

    pub fn with_id(mut self, id: &str) -> Checkbox {
        self.id = id.to_string();
        self
    }

    pub fn with_value(mut self, value: bool) -> Checkbox {
        self.value = value;
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Checkbox {
        self.transform = transform;
        self
    }

    pub fn with_translation(mut self, translation: Box<dyn GenerateTranslationScript>) -> Checkbox {
        self.transform.generate_translation = translation;
        self
    }

    pub fn with_fill(mut self, fill: Color) -> Checkbox {
        self.fill = Interpolatable::new(fill);
        self
    }

    pub fn with_accent(mut self, accent: Color) -> Checkbox {
        self.accent = accent;
        self
    }

    pub fn with_box_stroke(mut self, stroke: (f32, Color)) -> Checkbox {
        self.box_stroke = stroke;
        self
    }

    pub fn with_dimensions(mut self, dimensions: Vector2D<f32>) -> Checkbox {
        self.dimensions = Interpolatable::new(dimensions);
        self
    }

    pub fn with_events(mut self, events: Events) -> Checkbox {
        self.events = events;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Checkbox {
        self.z_index = z_index;
        self
    }
}