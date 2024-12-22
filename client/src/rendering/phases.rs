use std::{collections::BTreeSet, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use gloo::console::console;
use gloo_utils::{document, window};
use shared::{bool, rand, utils::vec2::Vector2D};
use ui::{canvas2d::{Canvas2d, ShapeType, Transform}, core::{ElementType, Events, HoverEffects, OnClickScript, UiElement}, elements::{button::Button, checkbox::Checkbox, label::{Label, TextEffects}, modal::Modal}, get_debug_window_props, translate, utils::{color::Color, sound::Sound}};
use rand::Rng;
use wasm_bindgen_futures::spawn_local;
use web_sys::{wasm_bindgen::JsCast, HtmlDivElement, HtmlInputElement};
use crate::{connection::socket::ConnectionState, storage_get, storage_set, world::{get_world, World}};

#[derive(Debug, Clone)]
pub enum GamePhase {
    Lore(u8),
    Home(Box<HomescreenElements>),
    Game,
    Death
}

impl Default for GamePhase {
    fn default() -> Self {
        let lore_played = bool!(storage_get!("lore_done").unwrap_or("0".to_string()).as_str());

        if !lore_played {
            GamePhase::Lore(0)
        } else {
            GamePhase::Home(Box::default())
        }
    }
}

#[derive(Debug, Clone)]
struct Shape {
    position: Vector2D<f32>,
    color: Color,
    radius: f32,
    angle: f32,
    speed: f32,
    shape: ShapeType
}

#[derive(Debug, Clone)]
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
    pub fn generate_lore_elements(world: &mut World) -> Vec<Box<dyn UiElement>> {
        let GamePhase::Lore(phase) = world.renderer.phase else { return vec![]; };

        let text = match phase {
            0 => "Are you there?",
            1 => "Long ago, peace thrived\nacross the universe.",
            2 => "Civilizations from distant worlds shared\nknowledge, power, and resources in harmony.",
            3 => "Health, Energy, Experience.\nEvery known being relies on these essentials.",
            4 => "Health flows from cometary waters.\nEnergy from the stars.\nExperience from the resources of planets.",
            5 => "As populations grew,\ndemand for these resources surged.",
            6 => "Supplies dwindled, and once-peaceful\n societies turned to conflict.",
            7 => "Alliances formed, each racing to\n amass resources, while tensions flared.",
            8 => "Some civilizations rose up for peace.",
            9 => "They were all eventually killed.",
            10 => "Survive.\nHarvest Health, Energy, and Experience.",
            11 => "Trade with allies for weapons and gear.",
            12 => "Good luck.",
            _ => {
                world.sounds.get_mut_sound("soundtrack_home").play();
                world.renderer.phase = GamePhase::Home(Box::default());

                storage_set!("lore_done", "1");

                return vec![];
            }
        };

        if phase == 9 {
            world.sounds.get_mut_sound("soundtrack_lore").stop();
        }

        let sound_name = if phase == 9 { "dialogue_unsettling" } else { "dialogue_normal" };

        let dialogue = Label::new()
            .with_text(text.to_string())
            .with_fill(Color::WHITE)
            .with_font(36.0)
            .with_stroke(Color::BLACK)
            .with_transform(translate!(0.0, -80.0))
            .with_events(Events::default().with_hoverable(false))
            .with_effects(TextEffects::Typewriter(
                0, 
                2,
                Some(Sound::new(sound_name, false))
            ));

        let continue_text = Label::new()
            .with_text("Continue".to_string())
            .with_fill(Color::WHITE)
            .with_font(32.0)
            .with_stroke(Color::BLACK)
            .with_transform(translate!(0.0, 10.0))
            .with_events(Events::default()
                .with_hover_effects(vec![HoverEffects::Inflation(1.1)])
            );

        let start = Button::new()
            .with_fill(Color::GREEN)
            .with_dimensions(Vector2D::new(200.0, 75.0))
            .with_transform(translate!(0.0, 100.0))
            .with_events(Events::default()
                .with_hover_effects(vec![
                    HoverEffects::Inflation(1.1),
                    HoverEffects::AdjustBrightness(0.0)
                ])
                .with_on_click(Box::new(|| {
                    spawn_local(async {
                        Sound::new(sound_name, false).restart();
                        Sound::new(sound_name, false).stop();

                        let mut world = get_world();
                        let GamePhase::Lore(phase) = &mut world.renderer.phase else { return; };

                        *phase += 1;
                        world.renderer.body.set_children(vec![]);
                    });
                }))
            )
            .with_children(vec![Box::new(continue_text)]);

        vec![Box::new(dialogue), Box::new(start)]
    }

    pub fn generate_homescreen_elements(world: &World) -> Vec<Box<dyn UiElement>> {
        let mut elements: Vec<Box<dyn UiElement>> = vec![];

        let title = Label::new()
            .with_text("PolyQuest".to_string())
            .with_fill(Color::WHITE)
            .with_font(72.0)
            .with_stroke(Color::BLACK)
            .with_transform(translate!(0.0, -80.0))
            .with_events(Events::default().with_hoverable(false));
            // .with_effects(TextEffects::Typewriter(0, 2, Some(Sound::new("dialogue_normal", false))));

        let start = Button::new()
            .with_fill(Color::GREEN)
            .with_dimensions(Vector2D::new(200.0, 75.0))
            .with_transform(translate!(0.0, 100.0))
            .with_events(Events::default()
                .with_hover_effects(vec![
                    HoverEffects::Inflation(1.1),
                    HoverEffects::AdjustBrightness(0.0)
                ])
                .with_on_click(Box::new(|| {
                    spawn_local(async {
                        let name = document().get_element_by_id("text_input").unwrap()
                            .dyn_into::<HtmlInputElement>().unwrap()
                            .value();
                    
                        if !name.is_empty() {
                            let mut world = get_world();
                            world.sounds.get_mut_sound("soundtrack_home").stop();
                            world.sounds.get_mut_sound("button_click").play();
                        }
                    });
                }))
            )
            .with_children(vec![Box::new(
                Label::new()
                    .with_text("Start".to_string())
                    .with_fill(Color::WHITE)
                    .with_font(32.0)
                    .with_stroke(Color::BLACK)
                    .with_transform(translate!(0.0, 10.0))
                    .with_events(Events::default()
                        .with_hover_effects(vec![HoverEffects::Inflation(1.1)])
                    )
            )]);

        let buttons: [(Vector2D<f32>, Color, &str, Box<OnClickScript>); 2] = [
            (
                Vector2D::ZERO,
                Color::GRAY, "{icon}\u{f013}",
                Box::new(|| {
                    spawn_local(async {
                        let mut children: Vec<Box<dyn UiElement>> = vec![
                            Box::new(Label::new()
                                .with_text("Settings".to_string())
                                .with_fill(Color::WHITE)
                                .with_font(48.0)
                                .with_stroke(Color::BLACK)
                                .with_transform(translate!(125.0, 75.0))
                                .with_events(Events::default().with_hoverable(false)))
                        ];

                        let settings: [(&str, &str, bool); 2] = [
                            (
                                "Soundtrack", "soundtrack", true
                            ),
                            (
                                "Sound Effects", "sfx", true
                            )
                        ];

                        for (i, (display_name, storage_name, default)) in settings.into_iter().enumerate() {
                            let checked = storage_get!(storage_name)
                                .map(|value| bool!(value.as_str()))
                                .unwrap_or(default);
                            let value = Arc::new(AtomicBool::new(checked));

                            let checkbox = Checkbox::new()
                                .with_accent(Color::GREEN)
                                .with_box_stroke((7.0, Color::BLACK))
                                .with_dimensions(Vector2D::new(50.0, 50.0))
                                .with_fill(Color::WHITE)
                                .with_transform(translate!(350.0, 180.0 + (i as f64 * 75.0)))
                                .with_value(checked)
                                .with_events(Events::default()
                                    .with_on_click(Box::new(move || {
                                        let value = value.clone();
                                        spawn_local(async move {
                                            let new = !value.load(Ordering::Relaxed);
                                            value.store(new, Ordering::Relaxed);
                                            
                                            storage_set!(storage_name, &(new as u8).to_string());
                                        });
                                    }))
                                );

                            let text = Label::new()
                                .with_text(display_name.to_string())
                                .with_fill(Color::WHITE)
                                .with_font(36.0)
                                .with_stroke(Color::BLACK)
                                .with_transform(translate!(550.0, 191.0 + (i as f64 * 75.0)))
                                .with_events(Events::default().with_hoverable(false));

                            children.push(Box::new(checkbox));
                            children.push(Box::new(text));
                        }

                        let mut modal = Modal::new()
                            .with_fill(Color::ORANGE)
                            .with_dimensions(Vector2D::new(1000.0, 350.0))
                            .with_children(children)
                            .with_close_button(Box::new(|| {
                                spawn_local(async {
                                    let mut world = get_world();
                
                                    for child in world.renderer.body.get_mut_children().iter_mut() {
                                        if child.get_identity() == ElementType::Modal {
                                            child.destroy();
                                            break;
                                        }
                                    }
                                });
                            }));
                        
                        get_world().renderer.body.get_mut_children().push(Box::new(modal));
                    });
                })
            ),
            (
                Vector2D::new(-100.0, 0.0), 
                Color::BLUE, "{brand}\u{f392}",
                Box::new(|| {
                    spawn_local(async {
                        let _ = window().open_with_url("https://discord.gg/UTvaAAgku3");
                    });
                })
            )
        ];

        for (translation, color, text, cb) in buttons {
            let button = Button::new()
                .with_fill(color)
                .with_dimensions(Vector2D::new(60.0, 60.0))
                .with_translation(Box::new(move |dimensions| {
                    Some(dimensions * (1.0 / 1.75) + translation)
                }))
                .with_events(Events::default()
                    .with_hover_effects(vec![
                        HoverEffects::Inflation(1.1),
                        HoverEffects::AdjustBrightness(0.0)
                    ])
                    .with_on_click(cb)
                )
                .with_children(vec![Box::new(
                    Label::new()
                        .with_text(text.to_string())
                        .with_fill(Color::WHITE)
                        .with_font(32.0)
                        .with_transform(translate!(0.0, 10.0))
                        .with_events(Events::default()
                            .with_hover_effects(vec![HoverEffects::Inflation(1.1)])
                        )
                )]);
            
            elements.push(Box::new(button));
        }

        elements.push(Box::new(title));

        if world.connection.state == ConnectionState::Connected {
            elements.push(Box::new(start));
        }

        elements
    }

    pub fn render_homescreen(world: &mut World) {
        let GamePhase::Home(ref mut elements) = world.renderer.phase else { return; };

        for shape in elements.shapes.iter_mut() {
            shape.position.y -= shape.speed;
            shape.angle += 0.005;

            if shape.position.y <= -1920.0 / 2.0 {
                shape.position.y = 1920.0 / 2.0;
            }

            world.renderer.canvas2d.save();
            world.renderer.canvas2d.translate(shape.position.x, shape.position.y);
            world.renderer.canvas2d.rotate(shape.angle);

            world.renderer.canvas2d.stroke_style(shape.color);
            world.renderer.canvas2d.set_stroke_size(5.0);
            // world.renderer.canvas2d.shadow_blur(2.0);
            // world.renderer.canvas2d.shadow_color(shape.color);

            shape.shape.render(&world.renderer.canvas2d, shape.radius, false, true);
            world.renderer.canvas2d.fill_style(shape.color);
            world.renderer.canvas2d.global_alpha(0.2);
            world.renderer.canvas2d.fill();

            world.renderer.canvas2d.restore();
        }
    }

    pub fn generate_game_elements(world: &World) -> Vec<Box<dyn UiElement>> {
        vec![]
    }

    pub fn render_game(world: &mut World) {

    }
}