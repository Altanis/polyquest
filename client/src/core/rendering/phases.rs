use shared::utils::vec2::Vector2D;
use ui::{button::Button, canvas2d::{Canvas2d, Transform}, color::Color, core::{Events, HoverEffects, UiElement}, label::{Label, TextEffects}};

use crate::world::World;

pub enum GamePhase {
    Home(HomescreenElements),
    Game,
    Death
}

impl Default for GamePhase {
    fn default() -> Self {
        GamePhase::Home(HomescreenElements::default())
    }
}


#[derive(Default)]
pub struct HomescreenElements {
    pub fps_counter: Label,
    pub title: Label,
    pub button: Button
}

impl HomescreenElements {
    pub fn iter(&self) -> Vec<&dyn UiElement> {
        vec![&self.fps_counter, &self.title, &self.button]
    }

    pub fn iter_mut(&mut self) -> Vec<&mut dyn UiElement> {
        vec![&mut self.fps_counter, &mut self.title, &mut self.button]
    }

    pub fn setup(world: &mut World) {
        let context = &world.renderer.canvas2d;
        let GamePhase::Home(elements) = &mut world.renderer.phase else { return; };

        elements.fps_counter = Label::new()
            .with_text("60.0 FPS".to_string())
            .with_fill(Color::WHITE)
            .with_font(28.0)
            .with_stroke(Color::BLACK)
            .with_transform(Transform::default().translate(75.0, 20.0))
            .with_events(Events::with_hoverable(false));

        elements.title = Label::new()
            .with_text("PolyFlux".to_string())
            .with_fill(Color::WHITE)
            .with_font(72.0)
            .with_stroke(Color::BLACK)
            .with_transform(context.get_transform().translate(0.0, -200.0))
            .with_events(Events::with_hoverable(false))
            .with_effects(TextEffects::Typewriter(0, 2));

        let text = Label::new()
            .with_text("im\ngonna\ndie".to_string())
            .with_fill(Color::WHITE)
            .with_font(26.0)
            .with_stroke(Color::BLACK)
            .with_transform(context.get_transform())
            .with_events(Events::with_hover_effects(vec![
                HoverEffects::Inflation(1.1)
            ]));

        elements.button = Button::new()
            .with_transform(context.get_transform())
            .with_fill(Color::RED)
            .with_stroke(10.0)
            .with_roundness(5.0)
            .with_dimensions(Vector2D::new(200.0, 200.0))
            .with_events(Events::with_hover_effects(vec![
                HoverEffects::Inflation(1.1),
                HoverEffects::AdjustBrightness(0.0),
                HoverEffects::Shake(45.0, false)
            ]))
            .with_label(text);
    }

    pub fn render(world: &mut World, delta_average: f64) {
        let context = &mut world.renderer.canvas2d;
        let GamePhase::Home(elements) = &mut world.renderer.phase else { return; };

        let fps = format!("{:.1} FPS", 1000.0 / delta_average);
        elements.fps_counter.set_text(fps);

        elements.fps_counter.render(context);
        elements.title.render(context);
        elements.button.render(context);
    }
}