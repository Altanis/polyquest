use std::collections::HashMap;
use derive_new::new as New;
use gloo::console::console;
use gloo_utils::{window, document};
use shared::{connection::packets::CensusProperties, fuzzy_compare, game::{body::{BodyIdentity, BodyIdentityIds, BodyRenderingHints}, entity::{EntityType, InputFlags, UpgradeStats, BASE_TANK_RADIUS}, turret::{TurretIdentity, TurretIdentityIds, TurretRenderingHints, TurretStructure}}, lerp, lerp_angle, utils::{codec::BinaryCodec, color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use strum::EnumCount;
use ui::{canvas2d::Canvas2d, core::UiElement, elements::tank::Tank, get_element_by_id_and_cast};
use web_sys::{wasm_bindgen::JsCast, HtmlInputElement};

use crate::{rendering::phases::GamePhase, world::World};

use shared::game::theme::{ENEMY_FILL, ENEMY_STROKE, PLAYER_FILL, PLAYER_STROKE, SMASHER_GUARD_FILL, SMASHER_GUARD_STROKE, STROKE_SIZE, TURRET_FILL, TURRET_STROKE};

use super::base::{Entity, HealthState};

impl Entity {
    pub fn parse_tank_census(&mut self, codec: &mut BinaryCodec, is_self: bool) {
        self.display.z_index = 1;

        let properties = codec.decode_varuint().unwrap();
        for _ in 0..properties {
            let property: CensusProperties = (codec.decode_varuint().unwrap() as u8).try_into().unwrap();

            match property {
                CensusProperties::Position => {
                    self.physics.position.target = Vector2D::new(
                        codec.decode_f32().unwrap(),
                        codec.decode_f32().unwrap()
                    );
                },
                CensusProperties::Velocity => {
                    self.physics.velocity.target = Vector2D::new(
                        codec.decode_f32().unwrap(),
                        codec.decode_f32().unwrap()
                    );
                },
                CensusProperties::Angle => {
                    let angle = codec.decode_f32().unwrap();
                    if !is_self {
                        self.physics.angle.target = angle;
                    }
                },
                CensusProperties::Name => self.display.name = codec.decode_string().unwrap(),
                CensusProperties::Score => self.display.score.target = codec.decode_varuint().unwrap() as f32,
                CensusProperties::Health => {
                    let old_state = self.stats.health_state;
                    let health = codec.decode_f32().unwrap();
                    if health < self.stats.health.target {
                        self.display.damage_blend.target = 0.9;
                    }

                    self.stats.health.target = health;

                    if health > 0.0 && old_state != HealthState::Alive {
                        if !self.stats.has_spawned {
                            self.stats.life_timestamps.0 = window().performance().unwrap().now();
                            self.display.kills += 1;
                        }

                        self.stats.health_state = HealthState::Alive;
                        self.stats.has_spawned = true;
                    } else if health <= 0.0 && old_state == HealthState::Alive {
                        self.stats.health_state = HealthState::Dying;
                    }
                },
                CensusProperties::MaxHealth => self.stats.max_health.target = codec.decode_f32().unwrap(),
                CensusProperties::Stats => {
                    self.display.available_stat_points = codec.decode_varuint().unwrap() as usize;
                    for i in 0..UpgradeStats::COUNT {
                        self.display.stat_investments[i] = codec.decode_varuint().unwrap() as usize;
                    }
                },
                CensusProperties::Upgrades => {
                    self.display.upgrades.clear();

                    let body_length = codec.decode_varuint().unwrap() as usize;
                    for _ in 0..body_length {
                        self.display.upgrades.push(codec.decode_varuint().unwrap() as i32);
                    }

                    if body_length != 0 {
                        self.display.upgrades.push(-1);
                    }

                    let turret_length = codec.decode_varuint().unwrap() as usize;
                    for _ in 0..turret_length {
                        self.display.upgrades.push(codec.decode_varuint().unwrap() as i32);
                    }
                },
                CensusProperties::Opacity => {
                    let opacity = codec.decode_f32().unwrap();
                    if self.stats.health_state == HealthState::Alive {
                        self.display.opacity.target = opacity;
                    }
                },
                CensusProperties::Fov => self.display.fov.target = codec.decode_f32().unwrap(),
                CensusProperties::Radius => self.display.radius.target = codec.decode_f32().unwrap(),
                CensusProperties::Identity => {
                    let body_identity_id: BodyIdentityIds = (codec.decode_varuint().unwrap() as usize).try_into().unwrap();
                    self.display.body_identity = body_identity_id.try_into().unwrap();

                    let turret_identity_id: TurretIdentityIds = (codec.decode_varuint().unwrap() as usize).try_into().unwrap();
                    self.display.turret_identity = turret_identity_id.try_into().unwrap();

                    self.display.turret_lengths.resize(self.display.turret_identity.turrets.len(), Interpolatable::new(1.0));
                },
                CensusProperties::Ticks => self.time.server_ticks = codec.decode_varuint().unwrap(),
                CensusProperties::Invincibility => self.display.invincible = codec.decode_bool().unwrap(),
                CensusProperties::Messages => {
                    let mut old_messages = std::mem::take(&mut self.display.messages);

                    self.display.typing = codec.decode_bool().unwrap();
                    let len = codec.decode_varuint().unwrap() as usize;

                    if old_messages.len() > len {
                        for i in 0..(old_messages.len() - len) {
                            let Some((message, position, mut opacity)) = old_messages.get(i).cloned() else { continue; };
                            if fuzzy_compare!(opacity.value, 0.0, 1e-1) {
                                old_messages.remove(0);
                                continue;
                            }

                            opacity.target = 0.0;
                            self.display.messages.push((message, position, opacity));
                        }

                        for i in (old_messages.len() - len)..((old_messages.len() - len) + len) {
                            let message = codec.decode_string().unwrap();
                            if let Some((_, position, opacity)) = old_messages.get(i).cloned() {
                                self.display.messages.push((message, position, opacity));
                            } else {
                                self.display.messages.push((message, Interpolatable::new(Vector2D::ZERO), Interpolatable::new(1.0)));
                            }
                        }
                    } else {
                        for i in 0..len {
                            let message = codec.decode_string().unwrap();
                            if let Some((_, position, opacity)) = old_messages.get(i).cloned() {
                                self.display.messages.push((message, position, opacity));
                            } else {
                                self.display.messages.push((message, Interpolatable::new(Vector2D::ZERO), Interpolatable::new(1.0)));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn render_tank_turrets(&self, context: &mut Canvas2d, is_self: bool) {
        Tank::render_turrets(context, STROKE_SIZE, self.display.radius.value, &self.display.turret_identity, &self.display.turret_lengths);
    }

    fn render_tank_body(&self, context: &mut Canvas2d, is_self: bool) {
        let (fill, stroke) = self.compute_body_fill(is_self);
        Tank::render_body(context, STROKE_SIZE, &self.display.body_identity, self.display.radius.value, fill, stroke);
    }

    pub fn render_tank(&mut self, context: &mut Canvas2d, is_self: bool, is_friendly: bool, dt: f32) {
        self.time.ticks += 1;
        if matches!(self.stats.health_state, HealthState::Dying | HealthState::Dead) {
            self.destroy_tank(context, is_friendly, dt);
        }

        context.save();
        
        context.translate(
            self.physics.position.value.x + self.physics.velocity.value.x, 
            self.physics.position.value.y + self.physics.velocity.value.y
        );
        context.global_alpha(self.display.opacity.value);

        for (i, (message, position, opacity)) in self.display.messages.iter_mut().enumerate() {
            let offset = -((3 - i) as f32) * 75.0 + if is_self { 200.0 } else { 250.0 };
            position.target = Vector2D::new(0.0, -self.display.radius.value - 120.0 - offset);

            position.value.lerp_towards(position.target, 0.15 * dt);
            opacity.value = lerp!(opacity.value, opacity.target, 0.15 * dt);

            context.save();
            context.set_font("42px Ubuntu");
            context.set_stroke_size(42.0 / 5.0);
            context.set_text_align("center");

            let rect_width = context.measure_text(message).width() as f32 + 35.0;

            context.fill_style(Color::BLACK);
            context.global_alpha(0.4 * opacity.value);
            context.begin_round_rect(position.value.x - rect_width / 2.0, position.value.y - 42.0, rect_width, 60.0, 5.0);
            context.fill();

            context.fill_style(Color::WHITE);
            context.stroke_style(Color::BLACK);
            context.global_alpha(1.0 * opacity.value);
            context.translate(position.value.x, position.value.y);
            context.stroke_text(message);
            context.fill_text(message);

            context.restore();
        }
        
        if is_self {
            let input = get_element_by_id_and_cast!("chat_input", HtmlInputElement);
            input.style().set_property("display", if self.display.typing { "block" } else { "none" });
            let _ = input.focus();
        }

        if self.display.typing {
            context.save();
            context.fill_style(Color::MATERIAL_ORANGE);
            context.stroke_style(Color::BLACK);
            context.set_font("28px Ubuntu");
            context.set_stroke_size(28.0 / 5.0);
            context.set_text_align("center");

            context.translate(0.0, self.display.radius.value + 75.0);
            context.stroke_text("Typing...");
            context.fill_text("Typing...");
            context.restore();
        }

        context.save();
        context.rotate(self.physics.angle.value);
        context.set_stroke_size(STROKE_SIZE);
        self.render_tank_turrets(context, is_friendly);
        self.render_tank_body(context, is_friendly);
        context.restore();

        context.restore();
    }

    fn destroy_tank(&mut self, context: &mut Canvas2d, is_friendly: bool, dt: f32) {
        if fuzzy_compare!(self.display.opacity.value, 0.0, 1e-1) {
            self.stats.health_state = HealthState::Dead;
            return;
        }

        self.display.opacity.target = 0.0;
        self.display.radius.target *= 1.05;
    }
}