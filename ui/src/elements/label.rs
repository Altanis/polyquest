use gloo::utils::window;
use shared::{fuzzy_compare, lerp, lerp_angle, utils::{color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use web_sys::MouseEvent;
use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, DeletionEffects, ElementType, Events, GenerateTranslationScript, HoverEffects, UiElement}, utils::sound::Sound, DEBUG};

pub enum TextEffects {
    Typewriter(usize, u64, Option<Sound>) // Typewriter(char_index, tick_interval)
}

pub struct Label {
    id: String,
    transform: Transform,
    font: Interpolatable<f32>,
    angle: Interpolatable<f32>,
    fill: Color,
    stroke: Option<Color>,
    text: String,
    dimensions: Vector2D,
    z_index: i32,
    effects: Option<TextEffects>,
    children: Vec<Box<dyn UiElement>>,
    events: Events,
    is_animating: bool,
    opacity: Interpolatable<f32>,
    destroyed: bool,
    align: &'static str,

    ticks: u64,
}

impl Default for Label {
    fn default() -> Self {
        Self {
            id: String::default(),
            transform: Default::default(),
            font: Interpolatable::default(),
            angle: Interpolatable::default(), 
            fill: Color::default(),
            stroke: None,
            text: String::default(),
            dimensions: Default::default(),
            z_index: Default::default(),
            effects: None,
            children: Default::default(),
            events: Default::default(),
            is_animating: Default::default(),
            opacity: Interpolatable::new(1.0),
            destroyed: Default::default(),
            align: "center",
            ticks: Default::default(),
        }
    }
}

impl UiElement for Label {
    fn get_identity(&self) -> crate::core::ElementType {
        ElementType::Label    
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
        self.transform = transform;
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn set_opacity(&mut self, opacity: f32) {
        self.opacity.target = opacity;
        self.opacity.value = opacity;
    }

    fn get_z_index(&self) -> i32 {
        self.z_index
    }

    fn set_hovering(&mut self, val: bool, _: &MouseEvent) -> bool {
        self.events.is_hovering = val;
        self.is_animating = true;

        val
    }

    fn set_clicked(&mut self, val: bool, _: &MouseEvent) {
        self.events.is_clicked = val;
    }

    // Text labels are meant to not have children.
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

    fn set_children(&mut self, _: Vec<Box<dyn UiElement>>) {}

    fn get_bounding_rect(&self) -> BoundingRect {
        let mut position = -self.dimensions * (1.0 / 2.0);
        let mut dimensions = self.dimensions;

        self.transform.transform_point(&mut position);

        let scale = self.transform.get_scale();
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

        let mut char_index: isize = isize::MAX;
        if let Some(TextEffects::Typewriter(idx, interval, sound)) = &mut self.effects {
            char_index = *idx as isize;
            
            if window().navigator().user_activation().has_been_active() && let Some(sound) = sound {
                if !sound.is_playing() && (char_index as usize) < self.text.len() {
                    sound.play();
                } else if sound.is_playing() && (char_index as usize) >= self.text.len() {
                    sound.stop();
                }
            }
            
            if self.ticks % *interval == 0 && (char_index as usize) < self.text.len() {
                *idx += 1;
            }
        }

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
            self.font.target = self.font.original;
            self.angle.target = self.angle.original;
            self.opacity.target = self.opacity.original;

            if !fuzzy_compare!(self.font.target, self.font.value, 1e-1)
                || !fuzzy_compare!(self.angle.value, self.angle.target, 1e-1)
                || !fuzzy_compare!(self.opacity.value, self.opacity.target, 1e-1) {
                self.is_animating = true;
            }
        }

        if let Some(t) = (self.transform.generate_translation)(dimensions) {
            self.transform.set_translation(t);
        }

        self.font.value = lerp!(self.font.value, self.font.target, 0.2);
        self.angle.value = lerp_angle!(self.angle.value, self.angle.target, shake_lerp_factor);
        self.opacity.value = lerp!(self.opacity.value, self.opacity.target, 0.2);

        let text: Vec<_> = self.text.split("\n").collect();
        let stroke_size = self.stroke.map_or(0.0, |_| if self.font.value < 24.0 {
            self.font.value / 4.5
        } else {
            (self.font.value.log(10.0)) * 5.0
        });

        let margin = self.font.value + stroke_size;

        let text_len = text.len();
        let half_up = text_len as f32 * margin / 2.0;
        let (mut width, mut height) = (0.0, 0.0);

        for (i, mut partial) in text.into_iter().enumerate() {
            char_index -= partial.len() as isize;
            if char_index < 0 {
                partial = &partial[0..(partial.len() - char_index.unsigned_abs())];
            }

            context.save();
            context.transform(&self.transform);
            context.global_alpha(self.opacity.value);

            if text_len != 1 {
                context.translate(
                    0.0,
                    ((i + 1) as f32 * margin) - half_up
                );
            }

            let (font, prefix) = match partial {
                _ if partial.starts_with("{icon}") => ("'Font Awesome 6 Free'", "{icon}"),
                _ if partial.starts_with("{brand}") => ("'Font Awesome 6 Brands'", "{brand}"),
                _ => ("Ubuntu", ""),
            };
            
            if !prefix.is_empty() {
                partial = &partial[prefix.len()..];
            }

            context.rotate(self.angle.value);
    
            context.set_miter_limit(2.0);
            context.fill_style(self.fill);
            context.set_font(&format!("bold {}px {}", self.font.value as u32, font));
            context.set_text_align(self.align);
    
            if stroke_size != 0.0 {
                context.stroke_style(self.stroke.unwrap());
                context.set_stroke_size(stroke_size);
                context.stroke_text(partial);
            }
    
            let metrics = context.measure_text(partial);
            let text_width = metrics.width() as f32;
            let text_height = margin;
    
            if text_width > width {
                width = text_width;
            }

            height += text_height;
    
            context.fill_text(partial);
            context.restore();
    
            if DEBUG {
                self.get_bounding_rect().render(context);
            }

            if char_index < 0 {
                break;
            }
        }

        self.dimensions = Vector2D::new(width, height);

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
    }

    fn has_animation_state(&self) -> bool {
        self.is_animating || matches!(self.effects, Some(TextEffects::Typewriter(..)))
    }
}

impl Label {
    pub fn on_hover(&mut self) -> f32 {
        let mut slf = 0.25;

        if let Some(inflation) = self.events.hover_effects.iter().find_map(
        |item| match item {
            HoverEffects::Inflation(a) => Some(*a),
            _ => None,
        }) {
            self.font.target = self.font.original * inflation;
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

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

impl Label {
    pub fn new() -> Label {
        Label::default()
    }

    pub fn with_id(mut self, id: &str) -> Label {
        self.id = id.to_string();
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Label {
        self.transform = transform;
        self
    }

    pub fn with_translation(mut self, translation: Box<dyn GenerateTranslationScript>) -> Label {
        self.transform.generate_translation = translation;
        self
    }

    pub fn with_font(mut self, font: f32) -> Label {
        self.font = Interpolatable::new(font);
        self
    }

    pub fn with_angle(mut self, angle: f32) -> Label {
        self.angle = Interpolatable::new(angle);
        self
    }

    pub fn with_fill(mut self, fill: Color) -> Label {
        self.fill = fill;
        self
    }

    pub fn with_stroke(mut self, stroke: Color) -> Label {
        self.stroke = Some(stroke);
        self
    }
    
    pub fn with_text(mut self, text: String) -> Label {
        self.text = text;
        self
    }

    pub fn with_dimensions(mut self, dimensions: Vector2D) -> Label {
        self.dimensions = dimensions;
        self
    }

    pub fn with_effects(mut self, effects: TextEffects) -> Label {
        self.effects = Some(effects);
        self
    }

    pub fn with_events(mut self, events: Events) -> Label {
        self.events = events;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Label {
        self.z_index = z_index;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Label {
        self.opacity = Interpolatable::new(opacity);
        self
    }

    pub fn with_align(mut self, alignment: &'static str) -> Label {
        self.align = alignment;
        self
    }
}