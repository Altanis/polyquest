use gloo::console::console;
use gloo_utils::window;
use shared::{rand, utils::vec2::Vector2D};
use ui::{elements::{button::Button, label::{Label, TextEffects}}, canvas2d::{Canvas2d, ShapeType, Transform}, color::Color, core::{Events, HoverEffects, UiElement}};
use rand::Rng;
use crate::world::{get_world, World};

pub enum GamePhase {
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
        }

        elements
    }
}

impl GamePhase {
    pub fn render_homescreen(context: &mut Canvas2d) {
        let world = get_world();
        let GamePhase::Home(ref mut elements) = world.renderer.phase else { return; };

        for shape in elements.shapes.iter_mut() {
            shape.position.y -= 0.5;
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