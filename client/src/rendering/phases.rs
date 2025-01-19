use std::{collections::BTreeSet, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use gloo::console::console;
use gloo_utils::{document, window};
use shared::{bool, connection::packets::Inputs, fuzzy_compare, game::{body::{BodyIdentity, BodyIdentityIds}, entity::{generate_identity, get_level_from_score, get_min_score_from_level, UpgradeStats, FICTITIOUS_TANK_RADIUS, MAX_STAT_INVESTMENT}, theme::{STROKE_INTENSITY, STROKE_SIZE}, turret::{TurretIdentityIds, TurretStructure}}, lerp, prettify_ms, prettify_score, rand, to_locale, utils::{color::Color, vec2::Vector2D}};
use strum::{EnumCount, IntoEnumIterator};
use ui::{canvas2d::{Canvas2d, ShapeType, Transform}, core::{DeletionEffects, ElementType, Events, HoverEffects, OnClickScript, UiElement}, elements::{button::Button, checkbox::Checkbox, label::{Label, TextEffects}, modal::Modal, progress_bar::ProgressBar, rect::Rect, tank::Tank}, get_debug_window_props, get_element_by_id_and_cast, translate, utils::sound::Sound};
use rand::Rng;
use wasm_bindgen_futures::spawn_local;
use web_sys::{wasm_bindgen::JsCast, HtmlDivElement, HtmlInputElement};
use crate::{connection::{packets, socket::ConnectionState}, game::entity::base::{Entity, HealthState}, storage_get, storage_set, world::{get_world, World}};
use shared::game::theme::{BAR_BACKGROUND, GRID_ALPHA, GRID_COLOR, GRID_SIZE, INBOUNDS_FILL, LEVEL_BAR_FOREGROUND, OUTBOUNDS_FILL, SCORE_BAR_FOREGROUND, UPGRADE_STAT_COLORS};

use self::packets::{form_input_packet, form_stats_packet, form_upgrade_packet};

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
struct Shape {
    position: Vector2D,
    color: Color,
    radius: f32,
    angle: f32,
    speed: f32,
    shape: ShapeType
}

#[derive(Debug, Clone, PartialEq)]
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
    pub fn same_phase(&self, other: &GamePhase) -> bool {
        matches!((self, other), 
            (GamePhase::Lore(_), GamePhase::Lore(_)) |
            (GamePhase::Home(_), GamePhase::Home(_)) |
            (GamePhase::Game, GamePhase::Game) |
            (GamePhase::Death, GamePhase::Death)
        )    
    }

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
                world.renderer.change_phase(GamePhase::Home(Box::default()));

                storage_set!("lore_done", "1");

                return vec![];
            }
        };

        if phase == 9 {
            world.sounds.get_mut_sound("soundtrack_lore").stop();
        }

        let sound_name = if phase == 9 { "dialogue_unsettling" } else { "dialogue_normal" };

        let dialogue = Label::new()
            .with_id("dialogue_entry")
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
            .with_id("continue_text")
            .with_text("Continue".to_string())
            .with_fill(Color::WHITE)
            .with_font(32.0)
            .with_stroke(Color::BLACK)
            .with_transform(translate!(0.0, 10.0))
            .with_events(Events::default()
                .with_hover_effects(vec![HoverEffects::Inflation(1.1)])
            );

        let continue_button = Button::new()
            .with_id(&format!("continue_button_{}", phase))
            .with_fill(Color::GREEN)
            .with_dimensions(Vector2D::new(200.0, 75.0))
            .with_transform(translate!(0.0, 100.0))
            .with_events(Events::default()
                .with_hover_effects(vec![
                    HoverEffects::Inflation(1.1),
                    HoverEffects::AdjustBrightness(0.0)
                ])
                .with_on_click(Box::new(|_| {
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

        vec![Box::new(dialogue), Box::new(continue_button)]
    }

    pub fn generate_homescreen_elements(world: &World) -> Vec<Box<dyn UiElement>> {
        let mut elements: Vec<Box<dyn UiElement>> = vec![];

        let title = Label::new()
            .with_id("title")
            .with_text("PolyQuest".to_string())
            .with_fill(Color::WHITE)
            .with_font(72.0)
            .with_stroke(Color::BLACK)
            .with_transform(translate!(0.0, -80.0))
            .with_events(Events::default().with_hoverable(false));

        let start = Button::new()
            .with_id("play_button")
            .with_fill(Color::GREEN)
            .with_dimensions(Vector2D::new(200.0, 75.0))
            .with_transform(translate!(0.0, 100.0))
            .with_events(Events::default()
                .with_hover_effects(vec![
                    HoverEffects::Inflation(1.1),
                    HoverEffects::AdjustBrightness(0.0)
                ])
                .with_on_click(Box::new(|_| {
                    spawn_local(async {
                        let name = get_element_by_id_and_cast!("text_input", HtmlInputElement)
                            .value();
                    
                        if !name.is_empty() {
                            let mut world = get_world();

                            world.sounds.get_mut_sound("button_click").play();
                            world.connection.send_message(packets::form_spawn_packet(name));
                        }
                    });
                }))
            )
            .with_children(vec![Box::new(
                Label::new()
                    .with_id("start_text")
                    .with_text("Start".to_string())
                    .with_fill(Color::WHITE)
                    .with_font(32.0)
                    .with_stroke(Color::BLACK)
                    .with_transform(translate!(0.0, 10.0))
                    .with_events(Events::default()
                        .with_hover_effects(vec![HoverEffects::Inflation(1.1)])
                    )
            )]);

        let buttons: [(Vector2D, Color, &str, Box<OnClickScript>); 2] = [
            (
                Vector2D::ZERO,
                Color::GRAY, "{icon}\u{f013}",
                Box::new(|_| {
                    spawn_local(async {
                        let mut children: Vec<Box<dyn UiElement>> = vec![
                            Box::new(Label::new()
                                .with_id("settings")
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
                                .with_id(&format!("checkbox-{}", i))
                                .with_accent(Color::GREEN)
                                .with_box_stroke((7.0, Color::BLACK))
                                .with_dimensions(Vector2D::new(50.0, 50.0))
                                .with_fill(Color::WHITE)
                                .with_transform(translate!(350.0, 180.0 + (i as f32 * 75.0)))
                                .with_value(checked)
                                .with_events(Events::default()
                                    .with_on_click(Box::new(move |_| {
                                        let value = value.clone();
                                        spawn_local(async move {
                                            let new = !value.load(Ordering::Relaxed);
                                            value.store(new, Ordering::Relaxed);
                                            
                                            storage_set!(storage_name, &(new as u8).to_string());
                                        });
                                    }))
                                );

                            let text = Label::new()
                                .with_id(&format!("label-{}", i))
                                .with_text(display_name.to_string())
                                .with_fill(Color::WHITE)
                                .with_font(36.0)
                                .with_stroke(Color::BLACK)
                                .with_transform(translate!(550.0, 191.0 + (i as f32 * 75.0)))
                                .with_events(Events::default().with_hoverable(false));

                            children.push(Box::new(checkbox));
                            children.push(Box::new(text));
                        }

                        let mut modal = Modal::new()
                            .with_id("modal-settings")
                            .with_fill(Color::ORANGE)
                            .with_dimensions(Vector2D::new(1000.0, 350.0))
                            .with_children(children)
                            .with_close_button(Box::new(|_| {
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
                Box::new(|_| {
                    spawn_local(async {
                        let _ = window().open_with_url("https://discord.gg/UTvaAAgku3");
                    });
                })
            )
        ];

        for (i, (translation, color, text, cb)) in buttons.into_iter().enumerate() {
            let button = Button::new()
                .with_id(&format!("menu-item-{}", i))
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
                        .with_id(&format!("menu-item-symbol-{}", i))
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

        let connection_text = match world.connection.state {
            ConnectionState::Connected => "",
            ConnectionState::Connecting => "Connecting...",
            ConnectionState::Failed => "Could not connect."
        };

        let state = Label::new()
            .with_id("connecting_text")
            .with_text(connection_text.to_string())
            .with_fill(Color::WHITE)
            .with_font(40.0)
            .with_stroke(Color::BLACK)
            .with_events(Events::default().with_hoverable(false));

        elements.push(Box::new(state));

        if world.connection.state == ConnectionState::Connected {
            elements.push(Box::new(start));
        }

        elements
    }

    pub fn render_homescreen(world: &mut World) {
        let GamePhase::Home(ref mut elements) = world.renderer.phase else { return; };
        let context = &mut world.renderer.canvas2d;

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

    pub fn generate_game_elements(world: &mut World) -> Vec<Box<dyn UiElement>> {
        let mut elements: Vec<Box<dyn UiElement>> = vec![];
        let dimensions = world.renderer.canvas2d.get_dimensions() * (1.0 / window().device_pixel_ratio() as f32);

        'nametag: {
            let score = world.game.self_entity.display.score.value as usize;
            let level = get_level_from_score(world.game.self_entity.display.score.target as usize);
    
            elements.push(Box::new(
                Label::new()
                    .with_id("nametag")
                    .with_text(world.game.self_entity.display.name.clone())
                    .with_fill(Color::WHITE)
                    .with_font(36.0)
                    .with_stroke(Color::BLACK)
                    .with_transform(translate!(dimensions.x / 2.0, dimensions.y - 72.5))
                    .with_events(Events::default().with_hoverable(false))
            ));
    
            elements.push(Box::new(
                ProgressBar::new()
                    .with_id("score_bar")
                    .with_transform(translate!(dimensions.x / 2.0, dimensions.y - 50.0))
                    .with_fill(BAR_BACKGROUND)
                    .with_accent(SCORE_BAR_FOREGROUND)
                    .with_dimensions(Vector2D::new(300.0, 20.0))
                    .with_value(5.0)
                    .with_max(10.0)
                    .with_children(vec![Box::new(
                        Label::new()
                            .with_id("score_bar_text")
                            .with_text(format!("Score: {}", to_locale!(score)))
                            .with_fill(Color::WHITE)
                            .with_font(16.0)
                            .with_stroke(Color::BLACK)
                            .with_events(Events::default().with_hoverable(false))
                        )
                    ])
            ));
    
            let score_min = get_min_score_from_level(level);
            let score_max = get_min_score_from_level(level + 1);

            elements.push(Box::new(
                ProgressBar::new()
                    .with_id("level_bar")
                    .with_transform(translate!(dimensions.x / 2.0, dimensions.y - 25.0))
                    .with_fill(BAR_BACKGROUND)
                    .with_accent(LEVEL_BAR_FOREGROUND)
                    .with_dimensions(Vector2D::new(425.0, 25.0))
                    .with_value((score.max(score_min) - score_min) as f32)
                    .with_max((score_max - score_min) as f32)
                    .with_children(vec![Box::new(
                        Label::new()
                            .with_id("level_bar_text")
                            .with_text(format!("Level {} {}", 
                                get_level_from_score(world.game.self_entity.display.score.target as usize),
                                generate_identity(world.game.self_entity.display.body_identity.id, world.game.self_entity.display.turret_identity.id)
                            ))
                            .with_fill(Color::WHITE)
                            .with_font(19.0)
                            .with_stroke(Color::BLACK)
                            .with_events(Events::default().with_hoverable(false))
                        )
                    ])
            ));
        }

        'stats: {
            let (stat_width, stat_height) = (200.0, 20.0);
            let upgrades_center = Vector2D::new(
                133.3 + (stat_width / 2.0 + dimensions.x / 50.0) / 5.0,
                (dimensions.y - (UpgradeStats::COUNT as f32 * 22.5) - 20.0)
            );
    
            let available_stat_points = world.game.self_entity.display.available_stat_points;
            let should_display_stats = available_stat_points > 0 || world.game.self_entity.display.should_display_stats;
    
            if should_display_stats {
                elements.push(Box::new(
                    Rect::new()
                        .with_id("stats_div")
                        .with_transform(translate!(
                            upgrades_center.x - stat_width / 2.0 - 35.0,
                            upgrades_center.y - 25.0
                        ))
                        .with_fill(Color::BLACK)
                        .with_stroke(5.0)
                        .with_roundness(5.0)
                        .with_dimensions(Vector2D::new(stat_width + 40.0, 37.5 + (UpgradeStats::COUNT as f32 * 22.5)))
                        .with_opacity(0.2)
                        .with_events(Events::default()
                            .with_deletion_effects(vec![DeletionEffects::Disappear])
                        )
                ));
    
                elements.push(Box::new(
                    Label::new()
                        .with_id("upgrades_text")
                        .with_text(format!("x{}", available_stat_points))
                        .with_fill(Color::WHITE)
                        .with_font(18.0)
                        .with_stroke(Color::BLACK)
                        .with_transform(translate!(upgrades_center.x - 15.0, upgrades_center.y))
                        .with_events(Events::default()
                            .with_deletion_effects(vec![DeletionEffects::Disappear])
                        )
                ));
        
                for (i, stat) in UpgradeStats::iter().enumerate() {
                    let color = UPGRADE_STAT_COLORS[i];
                    let value = world.game.self_entity.display.stat_investments[i];
        
                    elements.push(Box::new(
                        ProgressBar::new()
                            .with_id(&format!("upgrade_stat-{}", i))
                            .with_transform(translate!(
                                133.3,
                                (upgrades_center.y + 20.0) + (i as f32 * 22.5)
                            ))
                            .with_fill(BAR_BACKGROUND)
                            .with_accent(color)
                            .with_dimensions(Vector2D::new(stat_width, stat_height))
                            .with_value(value as f32)
                            .with_max(MAX_STAT_INVESTMENT as f32)
                            .with_children(vec![Box::new(
                                Label::new()
                                    .with_id(&format!("upgrade_stat_text-{}", i))
                                    .with_text(format!("{}", stat))
                                    .with_fill(Color::WHITE)
                                    .with_font(12.0)
                                    .with_stroke(Color::BLACK)
                                    .with_events(Events::default()
                                        .with_hoverable(false)
                                        .with_deletion_effects(vec![DeletionEffects::Disappear])
                                    )
                                ), 
                                Box::new(
                                    Label::new()
                                        .with_id(&format!("upgrade_stat_number-{}", i))
                                        .with_text(format!("[{}]", i + 1))
                                        .with_fill(Color::WHITE)
                                        .with_font(9.0)
                                        .with_stroke(Color::BLACK)
                                        .with_transform(translate!(stat_width / 2.0 - 20.0, -1.0))
                                        .with_events(Events::default()
                                            .with_hoverable(false)
                                            .with_deletion_effects(vec![DeletionEffects::Disappear]))
                                )
                            ])
                            .with_events(Events::default()
                                .with_hoverable(false)
                                .with_deletion_effects(vec![DeletionEffects::Disappear])
                            )
                    ));
        
                    elements.push(Box::new(
                        Button::new()
                            .with_id(&format!("upgrade-button-{}", i))
                            .with_fill(if available_stat_points > 0 && value < MAX_STAT_INVESTMENT { color } else { Color::SOFT_GRAY })
                            .with_dimensions(Vector2D::new(20.0, 20.0))
                            .with_transform(translate!(
                                133.3 + stat_width / 2.0 + 16.25,
                                (upgrades_center.y + 20.0) + (i as f32 * 22.5)
                            ))
                            .with_roundness(100.0)
                            .with_stroke((2.5, None))
                            .with_events(Events::default()
                                .with_hoverable(available_stat_points > 0 && value < MAX_STAT_INVESTMENT)
                                .with_hover_effects(vec![
                                    HoverEffects::Inflation(1.1),
                                    HoverEffects::AdjustBrightness(0.0)
                                ])
                                .with_deletion_effects(vec![DeletionEffects::Disappear])
                                .with_on_click(Box::new(|button| {
                                    let i = button.get_id()
                                        .split('-')
                                        .last()
                                        .and_then(|s| s.parse::<usize>().ok())
                                        .unwrap();
        
                                    spawn_local(async move {
                                        let world = get_world();
                                        world.connection.send_message(form_stats_packet(i));
                                    });
                                }))
                            )
                            .with_children(vec![Box::new(
                                Label::new()
                                    .with_id(&format!("upgrade-button-text-{}", i))
                                    .with_text("+".to_string())
                                    .with_fill(Color::WHITE)
                                    .with_stroke(Color::BLACK)
                                    .with_font(16.0)
                                    .with_transform(translate!(0.0, 5.0))
                                    .with_events(Events::default()
                                        .with_hover_effects(vec![HoverEffects::Inflation(1.1)])
                                        .with_deletion_effects(vec![DeletionEffects::Disappear])
                                    )
                            )])
                    ));
                }    
            }
    
            elements.push(Box::new(
                Button::new()
                    .with_id("toggle-upgrade-stats")
                    .with_fill(Color::SOFT_GRAY)
                    .with_dimensions(Vector2D::new(25.0, 20.0 + (UpgradeStats::COUNT as f32 * 25.0)))
                    .with_transform(translate!(
                        -5.0,
                        upgrades_center.y + 85.0
                    ))
                    .with_events(Events::default()
                        .with_hover_effects(vec![
                            HoverEffects::Inflation(1.1),
                            HoverEffects::AdjustBrightness(0.0)
                        ])
                        .with_on_click(Box::new(|button| {
                            spawn_local(async {
                                let mut world = get_world();
                                world.game.self_entity.display.should_display_stats =
                                    !world.game.self_entity.display.should_display_stats;
                            });
                        }))
                    )
                    .with_roundness(10.0)
            ));
        }

        'upgrades: {
            let position: Vector2D = Vector2D::new(dimensions.x / 35.0, 50.0);
            let dimensions = Vector2D::new(350.0, 250.0);

            if !world.game.self_entity.display.upgrades.is_empty() {
                elements.push(Box::new(
                    Rect::new()
                        .with_id("upgrades_div")
                        .with_transform(translate!(position.x as f32 - 25.0, position.y as f32 - 15.0))
                        .with_fill(Color::BLACK)
                        .with_stroke(10.0)
                        .with_roundness(5.0)
                        .with_dimensions(dimensions)
                        .with_opacity(0.2)
                        .with_events(Events::default().with_deletion_effects(vec![DeletionEffects::Disappear]))
                ));

                let is_body_upgrades = world.game.self_entity.display.upgrades.contains(&-1);

                let turret_structure = world.game.self_entity.display.turret_identity.clone();
                let body_structure = world.game.self_entity.display.body_identity.clone();

                for (i, &upgrade) in world.game.self_entity.display.upgrades.iter().enumerate() {
                    if upgrade == -1 {
                        break;
                    }

                    let color = Color::blend_colors(
                        UPGRADE_STAT_COLORS[(i + 3) % UpgradeStats::COUNT], 
                        Color::BLACK, 
                        0.05
                    );

                    let upgrade_position = position + Vector2D::new(
                        37.5 + (i % 3) as f32 * 112.5,
                        50.0 + (120.0 * (i / 3) as f32)
                    );

                    elements.push(Box::new(
                        Button::new()
                            .with_id(&format!("{}-upgrade-button-{}-{}", if is_body_upgrades { "body" } else { "turret" }, upgrade, i))
                            .with_fill(color)
                            .with_dimensions(Vector2D::new(100.0, 100.0))
                            .with_transform(translate!(
                                upgrade_position.x,
                                upgrade_position.y
                            ))
                            .with_roundness(1.0)
                            .with_stroke((3.5, Some(Color(85, 85, 85))))
                            .with_events(Events::default()
                                .with_hoverable(true)
                                .with_hover_effects(vec![
                                    HoverEffects::Inflation(1.1),
                                    HoverEffects::AdjustBrightness(0.0)
                                ])
                                .with_deletion_effects(vec![DeletionEffects::Disappear])
                                .with_on_click(Box::new(|button| {
                                    let i = button.get_id()
                                        .split('-')
                                        .last()
                                        .and_then(|s| s.parse::<usize>().ok())
                                        .unwrap();

                                    let upgrade_type = if button.get_id().split('-').next().unwrap() == "body" {
                                        0
                                    } else {
                                        1
                                    };

                                    spawn_local(async move {
                                        let world = get_world();
                                        world.connection.send_message(form_upgrade_packet(upgrade_type, i));
                                    });
                                }))
                            )
                            .with_children(vec![
                                Box::new(
                                    Tank::new()
                                        .with_id(&format!("upgrade-tank-icon-{}-{}", upgrade, i))
                                        .with_transform(translate!(0.0, -2.5))
                                        .with_radius(18.0)
                                        .with_stroke(STROKE_SIZE * (18.0 / FICTITIOUS_TANK_RADIUS))
                                        .with_body_identity(if is_body_upgrades {
                                            std::convert::TryInto::<BodyIdentity>::try_into(
                                                std::convert::TryInto::<BodyIdentityIds>::try_into(upgrade as usize).unwrap()
                                            ).unwrap()
                                        } else {
                                            body_structure.clone()
                                        })
                                        .with_turret_structure(if is_body_upgrades {
                                            turret_structure.clone()
                                        } else {
                                            std::convert::TryInto::<TurretStructure>::try_into(
                                                std::convert::TryInto::<TurretIdentityIds>::try_into(upgrade as usize).unwrap()
                                            ).unwrap()
                                        })
                                        .with_events(Events::default()
                                            .with_hover_effects(vec![HoverEffects::Inflation(1.1)])
                                            .with_deletion_effects(vec![DeletionEffects::Disappear])
                                        )                                        
                                ),
                                Box::new(
                                    Label::new()
                                        .with_id(&format!("body-button-text-{}-{}", upgrade, i))
                                        .with_text(if is_body_upgrades {
                                            format!("{}", std::convert::TryInto::<BodyIdentityIds>::try_into(upgrade as usize).unwrap())
                                        } else {
                                            format!("{}", std::convert::TryInto::<TurretIdentityIds>::try_into(upgrade as usize).unwrap())
                                        })
                                        .with_fill(Color::WHITE)
                                        .with_stroke(Color::BLACK)
                                        .with_font(18.0)
                                        .with_transform(translate!(0.0, 100.0 / 2.0 - 5.0))
                                        .with_events(Events::default()
                                            .with_hover_effects(vec![HoverEffects::Inflation(1.1)])
                                            .with_deletion_effects(vec![DeletionEffects::Disappear])
                                        )
                                )
                            ])
                    ));
                }
            }
        }

        elements
    }

    pub fn render_game(world: &mut World, delta_average: f64, is_dead: bool, dt: f32) {
        world.renderer.canvas2d.fill_style(OUTBOUNDS_FILL);
        world.renderer.canvas2d.fill_rect(0.0, 0.0, world.renderer.canvas2d.get_width() as f32, world.renderer.canvas2d.get_height() as f32);

        world.renderer.canvas2d.save();

        world.game.self_entity.lerp_all(dt, true);

        GamePhase::render_minimap(&mut world.renderer.canvas2d);

        world.renderer.canvas2d.save();

        let factor = window().device_pixel_ratio() as f32;
        let (screen_width, screen_height) = (world.renderer.canvas2d.get_width() as f32 / factor, world.renderer.canvas2d.get_height() as f32 / factor);

        world.renderer.canvas2d.translate(world.renderer.canvas2d.get_width() as f32 / 2.0, world.renderer.canvas2d.get_height() as f32 / 2.0);
        world.renderer.canvas2d.scale(factor * world.game.self_entity.display.fov.value, factor * world.game.self_entity.display.fov.value);
        world.renderer.canvas2d.translate(
            -world.game.self_entity.physics.position.value.x, 
            -world.game.self_entity.physics.position.value.y
        );

        world.renderer.canvas2d.fill_style(INBOUNDS_FILL);
        world.renderer.canvas2d.fill_rect(0.0, 0.0, world.game.arena_size, world.game.arena_size);

        GamePhase::render_grid(&mut world.renderer.canvas2d, world.game.self_entity.display.fov.value, world.game.arena_size);

        let mut entities: Vec<u32> = world.game.surroundings.iter_mut().map(|(k, v)| *k).collect();
        entities.push(world.game.self_entity.id);

        entities.sort_by(|a, b| {
            let a_index = world.game.get_mut_entity(*a).display.z_index;
            a_index
                .cmp(&world.game.get_mut_entity(*b).display.z_index)
        });

        for id in entities.iter_mut() {
            let is_self = *id == world.game.self_entity.id;
            if !is_self {
                world.game.get_mut_entity(*id).lerp_all(dt, false);
            }

            Entity::render(world, *id, dt);
        }

        entities.iter().for_each(|&id| Entity::render_health_bar(world, id, dt));
        entities.iter().for_each(|&id| if id != world.game.self_entity.id { Entity::render_nametag(world, id, dt) });

        if world.game.self_entity.stats.health_state == HealthState::Alive {
            GamePhase::send_packets(world);
        }

        world.renderer.canvas2d.restore();
        world.renderer.canvas2d.restore();

        world.renderer.canvas2d.save();

        world.renderer.backdrop_opacity.target = if is_dead { 0.6 } else { 0.0 };
        world.renderer.backdrop_opacity.value = lerp!(world.renderer.backdrop_opacity.value, world.renderer.backdrop_opacity.target, 0.2 * dt);

        world.renderer.canvas2d.save();
        world.renderer.canvas2d.fill_style(Color::BLACK);
        world.renderer.canvas2d.global_alpha(world.renderer.backdrop_opacity.value);
        world.renderer.canvas2d.fill_rect(0.0, 0.0, world.renderer.canvas2d.get_width() as f32, world.renderer.canvas2d.get_height() as f32);
        world.renderer.canvas2d.restore();


        GamePhase::render_notifications(world, dt);

        world.renderer.canvas2d.restore();
    }

    fn send_packets(world: &mut World) {
        if world.game.self_entity.physics.auto_fire {
            world.game.self_entity.physics.inputs.set_flag(Inputs::Shoot);
        }

        let mut mouse = world.game.self_entity.physics.mouse + (world.renderer.canvas2d.get_dimensions() * (1.0 / 2.0));
        let inverse_transform = world.renderer.canvas2d.get_transform().get_inverse();
        inverse_transform.transform_point(&mut mouse);

        world.connection.send_message(form_input_packet(
            world.game.self_entity.physics.inputs, 
            mouse
        ));
    }

    fn render_minimap(context: &mut Canvas2d) {

    }

    fn render_grid(context: &mut Canvas2d, fov: f32, arena_size: f32) {
        context.save();

        context.global_alpha(GRID_ALPHA);
        context.stroke_style(GRID_COLOR);
        context.set_stroke_size(1.0 / fov);

        for x in (0..(arena_size as usize)).step_by(GRID_SIZE as usize) {
            context.begin_path();
            context.move_to(x as f32, 0.0);
            context.line_to(x as f32, arena_size);
            context.stroke();
        }

        for y in (0..(arena_size as usize)).step_by(GRID_SIZE as usize) {
            context.begin_path();
            context.move_to(0.0, y as f32);
            context.line_to(arena_size, y as f32);
            context.stroke();
        }
        context.restore();
    }

    fn render_notifications(world: &mut World, dt: f32) {
        let length = world.game.self_entity.display.notifications.len();
        let mut deletions = vec![];

        for (i, notif) in world.game.self_entity.display.notifications.iter_mut().rev().enumerate() {
            if notif.position.direction == 1.0 {
                notif.position.value = Vector2D::new(
                    world.renderer.canvas2d.get_width() as f32 / 2.0,
                    50.0 + (i as f32 * 75.0)
                );

                notif.position.direction = -1.0;
            }

            notif.position.target = Vector2D::new(
                world.renderer.canvas2d.get_width() as f32 / 2.0,
                50.0 + (i as f32 * 75.0)
            );

            notif.opacity.target = if notif.lifetime > 0 { 1.0 } else { 0.0 };
            if fuzzy_compare!(notif.opacity.value, notif.opacity.target, 1e-1) {
                if notif.opacity.target == 1.0 {
                    notif.lifetime -= 1;
                    if notif.lifetime == 0 {
                        notif.opacity.target = 0.0;
                    }
                } else {
                    deletions.push(length - i - 1);
                }
            }

            notif.opacity.value = lerp!(notif.opacity.value, notif.opacity.target, 0.2 * dt);
            notif.position.value.lerp_towards(notif.position.target, 0.2 * dt);

            let context = &mut world.renderer.canvas2d;
            
            context.save();
            
            let font = 40.0;
            context.set_miter_limit(2.0);
            context.fill_style(Color::WHITE);
            context.stroke_style(Color::BLACK);
            context.set_text_align("center");
            context.set_font(&format!("bold {}px Ubuntu", font));
            context.set_stroke_size(font / 5.0);

            let width = context.measure_text(&notif.message).width();
            let height = font + (font / 5.0);

            context.save();
            context.global_alpha(0.6 * notif.opacity.value);
            context.fill_style(notif.color);
            context.stroke_style(Color::blend_colors(notif.color, Color::BLACK, STROKE_INTENSITY));
            context.set_stroke_size(STROKE_SIZE);
            context.fill_rect(
                notif.position.value.x - width as f32 / 2.0 - 25.0, 
                notif.position.value.y - height / 2.0,
                width as f32 + 50.0,
                height
            );
            context.stroke_rect(
                notif.position.value.x - width as f32 / 2.0 - 25.0, 
                notif.position.value.y - height / 2.0,
                width as f32 + 50.0,
                height
            );
            context.restore();

            context.save();
            context.global_alpha(notif.opacity.value);
            context.translate(notif.position.value.x, notif.position.value.y + height / 4.0);
            context.stroke_text(&notif.message);
            context.fill_text(&notif.message);
            context.restore();

            context.restore();
        }

        deletions.sort_by_key(|&e| std::cmp::Reverse(e));
        for deletion in deletions {
            world.game.self_entity.display.notifications.remove(deletion);
        }
    }

    pub fn generate_death_elements(world: &mut World) -> Vec<Box<dyn UiElement>> {
        let dimensions = world.renderer.canvas2d.get_dimensions() * (1.0 / window().device_pixel_ratio() as f32);
        let mut elements: Vec<Box<dyn UiElement>> = vec![
            Box::new(
                Label::new()
                    .with_id("death_starter")
                    .with_text("You were killed by".to_string())
                    .with_fill(Color::WHITE)
                    .with_font(18.0)
                    .with_stroke(Color::BLACK)
                    .with_transform(translate!(dimensions.x / 2.0, dimensions.y / 2.0 - 100.0))
                    .with_events(Events::default().with_hoverable(false))
            ),
            Box::new(
                Label::new()
                    .with_id("killer_name")
                    .with_text("ALTANIS!".to_string())
                    .with_fill(Color::WHITE)
                    .with_font(24.0)
                    .with_stroke(Color::BLACK)
                    .with_transform(translate!(dimensions.x / 2.0, dimensions.y / 2.0 - 70.0))
                    .with_events(Events::default().with_hoverable(false))
            ),
            Box::new(
                Label::new()
                    .with_id("score_tag")
                    .with_text("Score:".to_string())
                    .with_fill(Color::WHITE)
                    .with_font(21.0)
                    .with_stroke(Color::BLACK)
                    .with_transform(translate!(dimensions.x / 2.0 - 75.0, dimensions.y / 2.0 - 30.0))
                    .with_events(Events::default().with_hoverable(false))
                    .with_align("right")
            ),
            Box::new(
                Label::new()
                    .with_id("score_value")
                    .with_text(to_locale!(world.game.self_entity.display.score.value as u32))
                    .with_fill(Color::WHITE)
                    .with_font(21.0)
                    .with_stroke(Color::BLACK)
                    .with_transform(translate!(dimensions.x / 2.0, dimensions.y / 2.0 - 30.0))
                    .with_events(Events::default().with_hoverable(false))
                    .with_align("center")
            ),
            Box::new(
                Label::new()
                    .with_id("kills_tag")
                    .with_text("Kills:".to_string())
                    .with_fill(Color::WHITE)
                    .with_font(21.0)
                    .with_stroke(Color::BLACK)
                    .with_transform(translate!(dimensions.x / 2.0 - 75.0, dimensions.y / 2.0))
                    .with_events(Events::default().with_hoverable(false))
                    .with_align("right")
            ),
            Box::new(
                Label::new()
                    .with_id("kills_value")
                    .with_text(world.game.self_entity.display.kills.to_string())
                    .with_fill(Color::WHITE)
                    .with_font(21.0)
                    .with_stroke(Color::BLACK)
                    .with_transform(translate!(dimensions.x / 2.0, dimensions.y / 2.0))
                    .with_events(Events::default().with_hoverable(false))
                    .with_align("center")
            ),
            Box::new(
                Label::new()
                    .with_id("time_alive_tag")
                    .with_text("Time Alive:".to_string())
                    .with_fill(Color::WHITE)
                    .with_font(21.0)
                    .with_stroke(Color::BLACK)
                    .with_transform(translate!(dimensions.x / 2.0 - 75.0, dimensions.y / 2.0 + 30.0))
                    .with_events(Events::default().with_hoverable(false))
                    .with_align("right")
            ),
            Box::new(
                Label::new()
                    .with_id("time_alive_value")
                    .with_text(prettify_ms!(world.game.self_entity.stats.life_timestamps.1 - world.game.self_entity.stats.life_timestamps.0))
                    .with_fill(Color::WHITE)
                    .with_font(21.0)
                    .with_stroke(Color::BLACK)
                    .with_transform(translate!(dimensions.x / 2.0, dimensions.y / 2.0 + 30.0))
                    .with_events(Events::default().with_hoverable(false))
                    .with_align("center")
            ),
            Box::new(
                Button::new()
                    .with_id("start_button")
                    .with_fill(Color::GREEN)
                    .with_dimensions(Vector2D::new(150.0, 50.0))
                    .with_transform(translate!(dimensions.x / 2.0, dimensions.y / 2.0 + 100.0))
                    .with_events(Events::default()
                        .with_hover_effects(vec![
                            HoverEffects::Inflation(1.1),
                            HoverEffects::AdjustBrightness(0.0)
                        ])
                        .with_on_click(Box::new(|_| {
                            spawn_local(async {
                                let mut world = get_world();
                                world.renderer.change_phase(GamePhase::Home(Box::default()));
                            });
                        }))
                    )
                    .with_children(vec![Box::new(
                        Label::new()
                            .with_id("cont_text")
                            .with_text("Continue".to_string())
                            .with_fill(Color::WHITE)
                            .with_font(24.0)
                            .with_stroke(Color::BLACK)
                            .with_transform(translate!(0.0, 7.5))
                            .with_events(Events::default()
                                .with_hover_effects(vec![HoverEffects::Inflation(1.1)])
                            )
                    )])
            ),
        ];

        elements
    }
}