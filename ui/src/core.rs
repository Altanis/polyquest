use std::any::Any;

use shared::utils::vec2::Vector2D;
use web_sys::MouseEvent;
use crate::{canvas2d::{Canvas2d, Transform}, utils::color::Color};

#[derive(Clone, Copy, PartialEq)]
pub enum ElementType {
    Body,
    Button,
    Checkbox,
    Label,
    Modal,
    ProgressBar,
    Rect,
    Tank
}

pub trait UiElement {
    fn get_identity(&self) -> ElementType;
    fn get_id(&self) -> String;

    fn get_mut_events(&mut self) -> &mut Events;

    fn set_transform(&mut self, transform: Transform);
    fn get_transform(&self) -> &Transform;

    fn get_z_index(&self) -> i32;

    fn get_mut_children(&mut self) -> &mut Vec<Box<dyn UiElement>>;
    fn get_element_by_id(&mut self, id: &str) -> Option<(usize, &mut Box<dyn UiElement>)>;
    fn delete_element_by_id(&mut self, id: &str, destroy: bool);
    fn set_children(&mut self, children: Vec<Box<dyn UiElement>>);

    fn set_hovering(&mut self, val: bool, event: &MouseEvent) -> bool;
    fn set_clicked(&mut self, val: bool, event: &MouseEvent);

    fn get_bounding_rect(&self) -> BoundingRect;
    fn render(&mut self, context: &mut Canvas2d, dimensions: Vector2D<f32>) -> bool;
    fn destroy(&mut self);

    fn has_animation_state(&self) -> bool;
}

#[derive(Default, Debug, Clone)]
pub struct BoundingRect {
    pub position: Vector2D<f32>,
    pub dimensions: Vector2D<f32>
}

impl BoundingRect {
    pub fn new(position: Vector2D<f32>, dimensions: Vector2D<f32>) -> BoundingRect {
        BoundingRect {
            position,
            dimensions
        }
    }

    pub fn intersects(&self, point: Vector2D<f32>) -> bool {
        let within_x = point.x >= self.position.x && point.x <= self.position.x + self.dimensions.x;
        let within_y = point.y >= self.position.y && point.y <= self.position.y + self.dimensions.y;
        within_x && within_y
    }

    pub fn render(&self, context: &mut Canvas2d) {
        context.save();
        context.translate(self.position.x, self.position.y);
        context.fill_style(Color(255, 0, 0));
        context.stroke_style(Color(255, 0, 0));
        context.set_stroke_size(5.0);

        context.begin_arc(0.0, 0.0, 5.0, std::f64::consts::TAU);
        context.fill();

        context.stroke_rect(0.0, 0.0, self.dimensions.x, self.dimensions.y);

        context.restore();
    }
}

pub struct Events {
    pub hoverable: bool,
    pub is_hovering: bool,
    pub is_clicked: bool,
    pub hover_effects: Vec<HoverEffects>,
    pub deletion_effects: Vec<DeletionEffects>,
    pub on_click: Option<Box<OnClickScript>>
}

impl Default for Events {
    fn default() -> Events {
        Events {
            hoverable: true,
            is_hovering: false,
            is_clicked: false,
            hover_effects: vec![],
            deletion_effects: vec![],
            on_click: None
        }
    }
}

impl Events {
    pub fn with_hoverable(mut self, hoverable: bool) -> Events {
        self.hoverable = hoverable;
        self
    }

    pub fn with_hover_effects(mut self, hover_effects: Vec<HoverEffects>) -> Events {
        self.hover_effects = hover_effects;
        self
    }

    pub fn with_deletion_effects(mut self, deletion_effects: Vec<DeletionEffects>) -> Events {
        self.deletion_effects = deletion_effects;
        self
    }

    pub fn with_on_click(mut self, click_fn: Box<OnClickScript>) -> Events {
        self.on_click = Some(click_fn);
        self
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HoverEffects {
    Inflation(f32), // Inflation(blowup_factor)
    AdjustBrightness(f32), // AdjustBrightness(brightness)
    Shake(f32, bool, f32), // Shake(+/- angle, infinite, factor)
    Opacity(f32), // Opacity(hovering_opacity)
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeletionEffects {
    Disappear
}

pub type RenderingScript = dyn Fn(&Canvas2d);
pub type OnClickScript = dyn Fn(Box<&dyn UiElement>);

pub trait GenerateTranslationScript: Fn(Vector2D<f32>) -> Option<Vector2D<f32>> + Send + Sync + 'static {
    fn clone_box(&self) -> Box<dyn GenerateTranslationScript>;
}

impl<T> GenerateTranslationScript for T
where
    T: Fn(Vector2D<f32>) -> Option<Vector2D<f32>> + Clone + Send + Sync + 'static,
{
    fn clone_box(&self) -> Box<dyn GenerateTranslationScript> {
        Box::new(self.clone())
    }
}

impl Default for Box<dyn GenerateTranslationScript> {
    fn default() -> Self {
        Box::new(|_| None)
    }
}