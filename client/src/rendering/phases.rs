use gloo::console::console;
use gloo_utils::{document, window};
use shared::{rand, utils::vec2::Vector2D};
use ui::{canvas2d::{Canvas2d, ShapeType, Transform}, core::{Events, HoverEffects, UiElement}, elements::{button::Button, label::{Label, TextEffects}}, translate, utils::{color::Color, sound::Sound}};
use rand::Rng;
use web_sys::{wasm_bindgen::JsCast, HtmlInputElement};
use crate::world::{get_world, World};

pub enum GamePhase {
    Lore,
    Home(Box<HomescreenElements>),
    Game,
    Death
}

impl Default for GamePhase {
    fn default() -> Self {
        GamePhase::Home(Box::default())
    }
}

#[derive(Clone)]
struct Shape {
    position: Vector2D<f32>,
    color: Color,
    radius: f32,
    angle: f32,
    speed: f32,
    shape: ShapeType
}

pub struct HomescreenElements {
    shapes: [Shape; 50]
}

impl Default for HomescreenElements {
    fn default() -> Self {
        let mut elements = HomescreenElements {
            shapes: std::array::from_fn(|_| Shape {
                position: Vector2D::from_random(-1920.0 / 2.0, 1920.0 / 2.0),
                color: Color::random(),
                radius: 20.0,
                angle: rand!(0.0, std::f32::consts::TAU),
                speed: 1.0,
                shape: ShapeType::random()
            })
        };

        let shapes_clone = elements.shapes.clone();

        for first_shape in elements.shapes.iter_mut() {            
            if first_shape.shape == ShapeType::Pentagon {
                first_shape.radius *= 1.5;
            }

            for second_shape in shapes_clone.iter() {
                while first_shape.position.distance(second_shape.position) 
                    <= (first_shape.radius + second_shape.radius - 50.0) 
                {
                    first_shape.position = Vector2D::from_random(-1920.0 / 2.0, 1920.0 / 2.0);
                }
            }

            // let height = window().inner_height().unwrap().as_f64().unwrap();
            // first_shape.position.y = (height / 1.5) as f32;
        }

        elements
    }
}

impl GamePhase {
    pub fn generate_lore_elements() -> Vec<Box<dyn UiElement>> {
        vec![]
    }

    pub fn generate_homescreen_elements() -> Vec<Box<dyn UiElement>> {
        let title = Label::new()
            .with_text("PolyQuest".to_string())
            .with_fill(Color::WHITE)
            .with_font(72.0)
            .with_stroke(Color::BLACK)
            .with_transform(translate!(0.0, -80.0))
            .with_events(Events::default().with_hoverable(false))
            .with_effects(TextEffects::Typewriter(0, 2));

        let text = Label::new()
            .with_text("Start".to_string())
            .with_fill(Color::WHITE)
            .with_font(32.0)
            .with_stroke(Color::BLACK)
            .with_transform(translate!(0.0, 10.0))
            .with_events(Events::default()
                .with_hover_effects(vec![HoverEffects::Inflation(1.1)])
            );

        let start = Button::new()
            .with_fill(Color::GREEN)
            .with_stroke(7.0)
            .with_roundness(5.0)
            .with_dimensions(Vector2D::new(200.0, 75.0))
            .with_transform(translate!(0.0, 100.0))
            .with_events(Events::default()
                .with_hover_effects(vec![
                    HoverEffects::Inflation(1.1),
                    HoverEffects::AdjustBrightness(0.0)
                ])
                .with_on_click(Box::new(|| {
                    let name = document().get_element_by_id("text_input").unwrap()
                        .dyn_into::<HtmlInputElement>().unwrap()
                        .value();
                    
                    if !name.is_empty() {
                        let mut world = get_world();
                        world.soundtrack.stop();

                        Sound::new("button_click", false).play();
                    }
                }))
            )
            .with_children(vec![Box::new(text)]);
            
        vec![Box::new(title), Box::new(start)]
    }

    pub fn render_homescreen(context: &mut Canvas2d) {
        let world = get_world();
        let GamePhase::Home(ref mut elements) = world.renderer.phase else { return; };

        for shape in elements.shapes.iter_mut() {
            shape.position.y -= shape.speed;
            shape.angle += 0.005;

            if shape.position.y <= -1920.0 / 2.0 {
                shape.position.y = 1920.0 / 2.0;
            }

            context.save();
            context.translate(shape.position.x, shape.position.y);
            context.rotate(shape.angle);

            context.stroke_style(shape.color);
            context.set_stroke_size(5.0);
            // context.shadow_blur(2.0);
            // context.shadow_color(shape.color);

            shape.shape.render(context, shape.radius, false, true);
            context.fill_style(shape.color);
            context.global_alpha(0.2);
            context.fill();

            context.restore();
        }
    }
}