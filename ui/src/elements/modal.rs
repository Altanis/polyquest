use gloo::{console::console, utils::window};
use shared::{lerp, utils::{color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use web_sys::MouseEvent;

use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, ElementType, Events, GenerateTranslationScript, HoverEffects, OnClickScript, UiElement}, translate};

use super::{button::Button, label::Label};

#[derive(Default)]
pub struct Modal {
    id: String,
    transform: Transform,
    raw_transform: Transform,
    fill: Color,
    events: Events,
    dimensions: Interpolatable<Vector2D>,
    children: Vec<Box<dyn UiElement>>,
    deletion: bool,
    opacity: Interpolatable<f32>,
    is_animating: bool
}

impl UiElement for Modal {
    fn get_identity(&self) -> crate::core::ElementType {
        ElementType::Modal    
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
        999
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
    
        if is_hovering {
            self.is_animating = true;
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
        let mut position = -self.dimensions.value * (1.0 / 4.0);
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
        self.is_animating = false;
        let mut to_delete = false;

        context.save();
        context.reset_transform();
        context.fill_style(Color(0, 0, 0));
        context.global_alpha(0.6);
        context.fill_rect(0.0, 0.0, context.get_width(), context.get_height());
        context.restore();

        if !self.deletion && let Some(t) = (self.transform.generate_translation)(dimensions) {
            self.transform.set_translation(t);
        }

        self.dimensions.value.lerp_towards(self.dimensions.target, 0.2);
        if self.deletion && self.dimensions.value.is_zero(10.0) {
            to_delete = true;
        }
        
        context.save();
        context.set_transform(&self.transform);

        let position = -self.dimensions.value * (1.0 / 2.0);
        context.translate(position.x, position.y);

        self.raw_transform = context.get_transform();

        context.fill_style(self.fill);

        let stroke = 10.0;
        if stroke != 0.0 {
            let color = Color::blend_colors(
                self.fill, 
                Color::BLACK, 
                0.25
            );

            context.set_stroke_size(stroke);
            context.stroke_style(color);
        }

        context.begin_round_rect(
            0.0,
            0.0,
            self.dimensions.value.x,
            self.dimensions.value.y,
            5.0
        );

        context.fill();
        context.stroke();

        if self.dimensions.value.partial_eq(self.dimensions.target, 100.0) {
            self.opacity.value = lerp!(self.opacity.value, self.opacity.target, 0.2);
            let opacity = self.opacity.value;

            for child in self.get_mut_children().iter_mut() {
                context.save();
                context.global_alpha(opacity);
                child.render(context, dimensions);
                context.restore();
            }
        }
        
        if !self.dimensions.value.partial_eq(self.dimensions.target, 10.0) {
            self.is_animating = true;
        }

        context.restore();

        to_delete
    }
    
    fn destroy(&mut self) {
        self.deletion = true;
        self.dimensions.target = Vector2D::ZERO;
        self.opacity.target = 0.0;

        self.children.clear();
    }

    fn has_animation_state(&self) -> bool {
        self.is_animating
    }
}

impl Modal {
    pub fn new() -> Modal {
        let mut modal = Modal {
            opacity: Interpolatable::new(1.0),
            ..Default::default()
        };

        modal.opacity.value = 0.0;

        modal
    }

    pub fn with_id(mut self, id: &str) -> Modal {
        self.id = id.to_string();
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Modal {
        self.transform = transform;
        self
    }

    pub fn with_translation(mut self, translation: Box<dyn GenerateTranslationScript>) -> Modal {
        self.transform.generate_translation = translation;
        self
    }

    pub fn with_fill(mut self, fill: Color) -> Modal {
        self.fill = fill;
        self
    }

    pub fn with_events(mut self, events: Events) -> Modal {
        self.events = events;
        self
    }

    pub fn with_dimensions(mut self, dimensions: Vector2D) -> Modal {
        self.dimensions = Interpolatable::new(dimensions);
        self.dimensions.value = Vector2D::ZERO;

        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn UiElement>>) -> Modal {
        self.children = children;
        self
    }

    pub fn with_close_button(mut self, cb: Box<OnClickScript>) -> Modal {
        let text = Label::new()
            .with_id(&format!("close-button-{}-text", self.id))
            .with_text("X".to_string())
            .with_fill(Color::WHITE)
            .with_font(32.0)
            .with_stroke(Color::BLACK)
            .with_transform(translate!(0.0, 10.0))
            .with_events(Events::default()
                .with_hover_effects(vec![HoverEffects::Inflation(1.1)])
            );

        let close = Button::new()
            .with_id(&format!("close-button-{}", self.id))
            .with_fill(Color::RED)
            .with_dimensions(Vector2D::new(50.0, 50.0))
            .with_transform(translate!(self.dimensions.target.x - 35.0, 35.0))
            .with_events(Events::default()
                .with_hover_effects(vec![
                    HoverEffects::Inflation(1.1),
                    HoverEffects::AdjustBrightness(0.0)
                ])
                .with_on_click(cb)
            )
            .with_children(vec![Box::new(text)]);

        self.children.push(Box::new(close));
        self
    }
}