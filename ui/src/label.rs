use shared::utils::vec2::Vector2D;
use crate::{canvas2d::Canvas2d, color::Color, UiElement};

#[derive(Default, Clone)]
pub enum Alignment {
    #[default]
    Left,
    Center,
    Right
}

impl Alignment {
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        (match *self {
            Alignment::Left => "left",
            Alignment::Center => "center",
            Alignment::Right => "right"
        }).to_string()
    }
}

#[derive(Clone)]
pub enum Effect {
    Typewriter
}

#[derive(Default)]
pub struct Label {
    position: Vector2D<f32>,
    align: Alignment,
    fill: (u32, Color),
    stroke: Option<Color>,
    effects: Vec<Effect>,
    text: String
}

impl_builder!(Label {
    position: Vector2D<f32>,
    align: Alignment,
    fill: (u32, Color),
    stroke: Option<Color>,
    effects: Vec<Effect>,
    text: String
});

impl UiElement for Label {
    fn setup(&mut self) {}
    
    fn on_hover(&self) {

    }

    fn on_click(&self) {

    }

    fn render(&self, context: &mut Canvas2d) {
        context.save();
        context.translate(self.position.x, self.position.y);

        context.set_miter_limit(2.0);
        context.fill_style(self.fill.1);
        context.set_font(&format!("bold {}px BankGothic", self.fill.0));
        context.set_text_align(&self.align.to_string());

        if let Some(stroke) = self.stroke {
            context.stroke_style(stroke);
            context.set_stroke_size((self.fill.0 / 5) as f64);
            context.stroke_text(&self.text);
        }

        context.fill_text(&self.text);

        context.restore();
    }
}

impl Label {}