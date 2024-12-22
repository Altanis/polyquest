use shared::utils::vec2::Vector2D;
use web_sys::MouseEvent;

use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, ElementType, Events, UiElement}, utils::color::Color};

#[derive(Default)]
pub struct Body {
    id: String,
    transform: Transform,
    fill: Color,
    events: Events,
    dimensions: Vector2D<f32>,
    children: Vec<Box<dyn UiElement>>
}

impl UiElement for Body {
    fn get_identity(&self) -> crate::core::ElementType {
        ElementType::Body    
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
        1
    }

    fn set_hovering(&mut self, _: bool, _: &MouseEvent) -> bool {
        false
    }

    fn set_clicked(&mut self, _: bool, _: &MouseEvent) {}

    fn get_mut_children(&mut self) -> &mut Vec<Box<dyn UiElement>> {
        self.children.sort_by_key(|child| child.get_z_index());
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
        BoundingRect::new(
            Vector2D::ZERO,
            self.dimensions
        )
    }

    fn render(&mut self, context: &mut Canvas2d, _: Vector2D<f32>) -> bool {
        context.save();
        context.fill_style(self.fill);
        context.fill_rect(0, 0, context.get_width(), context.get_height());
        context.restore();

        context.save();

        self.dimensions = context.get_dimensions();
        context.translate(self.dimensions.x / 2.0, self.dimensions.y / 2.0);

        let factor = (self.dimensions.x / 1920.0).max(self.dimensions.y / 1080.0);
        self.dimensions *= 1.0 / factor;

        context.scale(factor, factor);

        if let Some(t) = (self.transform.generate_translation)(self.dimensions) {
            self.transform.set_translation(t);
        }

        false
    }

    fn destroy(&mut self) {}
}

impl Body {
    pub fn render_children(&mut self, context: &mut Canvas2d) {
        let mut deletions = vec![];
        for (i, child) in self.children.iter_mut().enumerate() {
            if child.render(context, self.dimensions) {
                deletions.push(i);
            }
        }

        for deletion in deletions {
            self.children.remove(deletion);
        }
    }

    pub fn with_id(mut self, id: &str) -> Body {
        self.id = id.to_string();
        self
    }

    pub fn with_fill(mut self, fill: Color) -> Body {
        self.fill = fill;
        self
    }

    pub fn with_events(mut self, events: Events) -> Body {
        self.events = events;
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn UiElement>>) -> Body {
        self.children = children;
        self
    }
}