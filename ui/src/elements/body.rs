use shared::utils::vec2::Vector2D;

use crate::{canvas2d::{Canvas2d, Transform}, utils::color::Color, core::{BoundingRect, Events, RenderingScript, UiElement}};

#[derive(Default)]
pub struct Body {
    transform: Transform,
    fill: Color,
    events: Events,
    dimensions: Vector2D<f32>,
    children: Vec<Box<dyn UiElement>>
}

impl UiElement for Body {
    fn get_mut_events(&mut self) -> &mut Events {
        &mut self.events
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform.clone();
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn set_hovering(&mut self, _: bool) {}
    fn set_clicked(&mut self, _: bool) {}

    fn get_mut_children(&mut self) -> &mut Vec<Box<dyn UiElement>> {
        &mut self.children
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

    fn render(&mut self, context: &mut crate::canvas2d::Canvas2d) {
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
    }
}

impl Body {
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