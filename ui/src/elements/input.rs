use std::{cell::RefCell, rc::Rc};

use gloo::{console::console, utils::{body, document, window}};
use shared::utils::{color::Color, vec2::Vector2D};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{Event, HtmlInputElement, MouseEvent};
use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, ElementType, Events, UiElement}, DEBUG};

pub struct Input {
    id: String,
    transform: Transform,
    raw_transform: Transform,
    font: (i32, Color),
    fill: Color,
    stroke: (f32, Option<Color>),
    dimensions: Vector2D,
    roundness: f32,
    events: Events,
    children: Vec<Box<dyn UiElement>>,
    input: Option<HtmlInputElement>,
    #[allow(clippy::type_complexity)]
    validator: Rc<RefCell<Option<Box<dyn Fn(&str) -> bool + 'static>>>>,
    max_length: i32,

    ticks: u64
}

impl Default for Input {
    fn default() -> Self {
        Input {
            id: Default::default(),
            transform: Default::default(),
            raw_transform: Default::default(),
            font: (0, Color::WHITE),
            fill: Default::default(),
            stroke: (-1.0, None),
            dimensions: Default::default(),
            roundness: 5.0,
            events: Default::default(),
            children: vec![],
            input: None,
            max_length: 0,
            validator: Rc::new(RefCell::new(None)),
            ticks: Default::default()
        }
    }
}

impl UiElement for Input {
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
        self.transform = transform.clone();
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn set_opacity(&mut self, _: f32) {}
    
    fn get_z_index(&self) -> i32 {
        i32::MAX
    }

    fn set_hovering(&mut self, _: bool, _: &MouseEvent) -> bool {
        false
    }

    fn set_clicked(&mut self, _: bool, _: &MouseEvent) {}

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
        let mut position = -self.dimensions * (1.0 / 2.0);
        let mut dimensions = self.dimensions;

        self.raw_transform.transform_point(&mut position);

        let scale = self.raw_transform.get_scale();
        dimensions.x *= scale.x;
        dimensions.y *= scale.y;

        BoundingRect::new(
            position,
            dimensions
        )
    }

    fn render(&mut self, context: &mut Canvas2d, _: Vector2D) -> bool {
        self.ticks += 1;
    
        context.save();
        context.transform(&self.transform);
    
        let mut position = -self.dimensions * (1.0 / 2.0);
        let mut dimensions = self.dimensions;
        context.get_transform().transform_point(&mut position);
    
        let scale = context.get_transform().get_scale();
        dimensions.x *= scale.x;
        dimensions.y *= scale.y;
    
        let rect = BoundingRect::new(position, dimensions);
    
        if self.input.is_none() {
            if let Some(input) = document().get_element_by_id(&self.id) {
                self.input = Some(input.dyn_into().unwrap());
            } else {
                let input = document().create_element("input").unwrap().dyn_into::<HtmlInputElement>().unwrap();
                input.set_id(&self.id);
                let _ = body().append_child(&input);

                input.dataset().set("last_valid_value", "").unwrap();

                let validator_clone = self.validator.clone();
                let input_clone = input.clone();

                let closure = Closure::<dyn FnMut(_)>::new(move |_: Event| {
                    let value = input_clone.value();
                    let validator_guard = validator_clone.borrow();
                    if let Some(validator) = &*validator_guard {
                        if validator(&value) {
                            input_clone.dataset().set("last_valid_value", &value).unwrap();
                        } else {
                            let last_valid = input_clone.dataset().get("last_valid_value").unwrap_or_default();
                            input_clone.set_value(&last_valid);
                        }
                    }
                });

                input.add_event_listener_with_callback("input", closure.as_ref().unchecked_ref()).unwrap();
                closure.forget();

                self.input = Some(input);
            }
        }
    
        let input = self.input.as_ref().unwrap();
        let _ = input.style().set_property("display", "block");
        let _ = input.style().set_property("position", "absolute");
        let _ = input.style().set_property("background-color", &self.fill.css());
    
        let canvas_rect = context.get_bounding_client_rect();
        
        let buffer_width = context.get_width() as f64;
        let buffer_height = context.get_height() as f64;
        let css_width = context.get_offset_width() as f64;
        let css_height = context.get_offset_height() as f64;
        
        let scale_x = css_width / buffer_width;
        let scale_y = css_height / buffer_height;

        let screen_left = canvas_rect.left() + (rect.position.x as f64) * scale_x;
        let screen_top = canvas_rect.top() + (rect.position.y as f64) * scale_y;
        let screen_width = (rect.dimensions.x as f64) * scale_x;
        let screen_height = (rect.dimensions.y as f64) * scale_y;
    
        let _ = input.style().set_property("left", &format!("{}px", screen_left));
        let _ = input.style().set_property("top", &format!("{}px", screen_top));
        let _ = input.style().set_property("width", &format!("{}px", screen_width));
        let _ = input.style().set_property("height", &format!("{}px", screen_height));
    
        let _ = input.style().set_property("font-size", &format!("{}px", self.font.0));
        let _ = input.style().set_property("padding", "5px");
        let _ = input.style().set_property("color", &self.font.1.css());
        let _ = input.style().set_property("font-family", "Ubuntu");
        let _ = input.style().set_property("border-radius", &format!("{}px", self.roundness));
        let _ = input.style().set_property("outline", "none");

        if let (stroke_width, Some(stroke_color)) = self.stroke {
            let _ = input.style().set_property("border", &format!("{}px solid {}", stroke_width, stroke_color.css()));
        }

        if self.max_length != 0 {
            input.set_max_length(self.max_length);
        }
    
        if DEBUG {
            context.save();
            context.reset_transform();
            context.stroke_style(Color::RED);
            context.begin_rect(rect.position.x, rect.position.y, rect.dimensions.x, rect.dimensions.y);
            context.stroke();
            context.restore();
        }
    
        context.restore();
    
        false
    }

    fn destroy(&mut self) {
        if let Some(element) = document().get_element_by_id(&self.id) {
            element.remove();   
        }
    }

    fn has_animation_state(&self) -> bool {
        false
    }
}


impl Input {
    pub fn new() -> Input {
        Input::default()
    }

    pub fn with_id(mut self, id: &str) -> Input {
        self.id = id.to_string();
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Input {
        self.transform = transform;
        self
    }

    pub fn with_fill(mut self, fill: Color) -> Input {
        self.fill = fill;
        self
    }

    pub fn with_stroke(mut self, stroke: (f32, Color)) -> Input {
        self.stroke = (stroke.0, Some(stroke.1));
        self
    }

    pub fn with_dimensions(mut self, dimensions: Vector2D) -> Input {
        self.dimensions = dimensions;
        self
    }

    pub fn with_roundness(mut self, roundness: f32) -> Input {
        self.roundness = roundness;
        self
    }

    pub fn with_events(mut self, events: Events) -> Input {
        self.events = events;
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn UiElement>>) -> Input {
        self.children = children;
        self
    }

    pub fn with_font(mut self, font: (i32, Color)) -> Input {
        self.font = font;
        self
    }

    pub fn with_max_length(mut self, length: i32) -> Input {
        self.max_length = length;
        self
    }

    pub fn with_validator(self, validator: impl Fn(&str) -> bool + 'static) -> Self {
        *self.validator.borrow_mut() = Some(Box::new(validator));
        self
    }
}