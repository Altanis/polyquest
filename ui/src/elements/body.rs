use gloo::utils::window;
use shared::utils::{color::Color, vec2::Vector2D};
use web_sys::MouseEvent;

use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, ElementType, Events, UiElement}};

#[derive(Default)]
pub struct Body {
    id: String,
    transform: Transform,
    fill: Color,
    events: Events,
    dimensions: Vector2D,
    children: Vec<Box<dyn UiElement>>
}

impl UiElement for Body {
    fn get_identity(&self) -> crate::core::ElementType {
        ElementType::Body    
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

    fn set_opacity(&mut self, _: f32) {}

    fn get_z_index(&self) -> i32 {
        1
    }

    fn set_hovering(&mut self, _: bool, _: &MouseEvent) -> bool {
        false
    }

    fn set_clicked(&mut self, _: bool, _: &MouseEvent) {}

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
        BoundingRect::new(
            Vector2D::ZERO,
            self.dimensions
        )
    }

    fn render(&mut self, context: &mut Canvas2d, _: Vector2D) -> bool {
        context.save();
        context.fill_style(self.fill);
        context.fill_rect(0.0, 0.0, context.get_width(), context.get_height());
        context.restore();

        context.save();

        self.dimensions = context.get_dimensions();
        context.translate(self.dimensions.x / 2.0, self.dimensions.y / 2.0);

        let factor = window().device_pixel_ratio() as f32;
        self.dimensions *= 1.0 / factor;

        context.scale(factor, factor);

        if let Some(t) = (self.transform.generate_translation)(self.dimensions) {
            self.transform.set_translation(t);
        }

        false
    }

    fn destroy(&mut self) {}

    fn has_animation_state(&self) -> bool {
        false
    }
}

impl Body {
    pub fn render_children(&mut self, context: &mut Canvas2d) {
        let mut deletions = vec![];
        let dimensions = self.dimensions;
        let children = self.get_mut_children();

        for (i, child) in children.iter_mut().enumerate() {
            if child.render(context, dimensions) {
                deletions.push(i);
            }
        }

        deletions.sort_by_key(|&e| std::cmp::Reverse(e));

        for deletion in deletions {
            children.remove(deletion);
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