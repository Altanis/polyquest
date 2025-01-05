use std::collections::HashMap;
use derive_new::new as New;
use gloo::console::console;
use shared::{connection::packets::CensusProperties, fuzzy_compare, game::{body::{BodyIdentity, BodyIdentityIds, BodyRenderingHints}, entity::{EntityType, InputFlags, TankUpgrades, UpgradeStats, BASE_TANK_RADIUS}, turret::{TurretIdentity, TurretIdentityIds, TurretRenderingHints, TurretStructure}}, lerp, lerp_angle, utils::{codec::BinaryCodec, color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use strum::EnumCount;
use ui::{canvas2d::Canvas2d, core::UiElement, elements::tank::Tank};

use crate::{rendering::phases::GamePhase, world::World};

use shared::game::theme::{ENEMY_FILL, ENEMY_STROKE, PLAYER_FILL, PLAYER_STROKE, SMASHER_GUARD_FILL, SMASHER_GUARD_STROKE, STROKE_SIZE, TURRET_FILL, TURRET_STROKE};

use super::base::{Entity, HealthState};

impl Entity {
    pub fn parse_tank_census(&mut self, codec: &mut BinaryCodec, is_self: bool) {
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

                    self.stats.health.target = health;

                    if health > 0.0 && old_state != HealthState::Alive {
                        self.stats.health_state = HealthState::Alive;
                    } else if health <= 0.0 && old_state == HealthState::Alive {
                        self.stats.health_state = HealthState::Dying;
                    }
                },
                CensusProperties::MaxHealth => self.stats.max_health.target = codec.decode_f32().unwrap(),
                CensusProperties::Energy => self.stats.energy.target = codec.decode_f32().unwrap(),
                CensusProperties::MaxEnergy => self.stats.max_energy.target = codec.decode_f32().unwrap(),
                CensusProperties::Stats => {
                    self.display.available_stat_points = codec.decode_varuint().unwrap() as usize;
                    for i in 0..UpgradeStats::COUNT {
                        self.display.stat_investments[i] = codec.decode_varuint().unwrap() as usize;
                    }
                },
                CensusProperties::Upgrades => {
                    self.display.upgrades.body.clear();
                    self.display.upgrades.turret.clear();

                    let body_length = codec.decode_varuint().unwrap() as usize;
                    for _ in 0..body_length {
                        self.display.upgrades.body.push((codec.decode_varuint().unwrap() as usize).try_into().unwrap());
                    }

                    let turret_length = codec.decode_varuint().unwrap() as usize;
                    for _ in 0..turret_length {
                        self.display.upgrades.turret.push((codec.decode_varuint().unwrap() as usize).try_into().unwrap());
                    }
                },
                CensusProperties::Opacity => self.display.opacity.target = codec.decode_f32().unwrap(),
                CensusProperties::Fov => self.display.fov.target = codec.decode_f32().unwrap(),
                CensusProperties::Radius => self.display.radius.target = codec.decode_f32().unwrap(),
                CensusProperties::Identity => {
                    let body_identity_id: BodyIdentityIds = (codec.decode_varuint().unwrap() as usize).try_into().unwrap();
                    self.display.body_identity = body_identity_id.try_into().unwrap();

                    let turret_identity_id: TurretIdentityIds = (codec.decode_varuint().unwrap() as usize).try_into().unwrap();
                    self.display.turret_identity = turret_identity_id.try_into().unwrap();
                },
                _ => {}
            }
        }
    }

    fn render_tank_turrets(&self, context: &mut Canvas2d, is_self: bool) {
        Tank::render_turrets(context, self.display.radius.value, &self.display.turret_identity);
    }

    fn render_tank_body(&self, context: &mut Canvas2d, is_self: bool) {
        let (fill, stroke) = self.compute_body_fill(is_self);
        Tank::render_body(context, &self.display.body_identity, self.display.radius.value, fill, stroke);
    }

    pub fn render_tank(&mut self, context: &mut Canvas2d, is_friendly: bool, dt: f32) {
        self.time.ticks += 1;
        if matches!(self.stats.health_state, HealthState::Dying | HealthState::Dead) {
            self.destroy_tank(context, is_friendly, dt);
        }

        context.save();
        
        context.translate(
            self.physics.position.value.x + self.physics.velocity.value.x, 
            self.physics.position.value.y + self.physics.velocity.value.y
        );
        context.rotate(self.physics.angle.value);
        context.global_alpha(self.display.opacity.value as f64);
        context.set_stroke_size(STROKE_SIZE);

        self.render_tank_turrets(context, is_friendly);
        self.render_tank_body(context, is_friendly);

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