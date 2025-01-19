use std::{collections::HashMap, num::NonZeroU32};
use derive_new::new as New;
use gloo::console::console;
use shared::{connection::packets::CensusProperties, fuzzy_compare, game::{body::{BodyIdentity, BodyIdentityIds, BodyRenderingHints}, entity::{EntityType, InputFlags, UpgradeStats, BASE_TANK_RADIUS}, turret::{TurretIdentity, TurretIdentityIds, TurretRenderingHints, TurretStructure}}, lerp, lerp_angle, utils::{codec::BinaryCodec, color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use strum::EnumCount;
use ui::{canvas2d::Canvas2d, core::UiElement, elements::tank::Tank};

use crate::{rendering::phases::GamePhase, world::World};

use shared::game::theme::{ENEMY_FILL, ENEMY_STROKE, PLAYER_FILL, PLAYER_STROKE, SMASHER_GUARD_FILL, SMASHER_GUARD_STROKE, STROKE_SIZE, TURRET_FILL, TURRET_STROKE};

use super::base::{Entity, HealthState};

impl Entity {
    pub fn parse_projectile_census(&mut self, codec: &mut BinaryCodec, is_self: bool) {
        self.display.z_index = 0;

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
                CensusProperties::Angle => self.physics.angle.target = codec.decode_f32().unwrap(),
                CensusProperties::Health => {
                    let health = codec.decode_f32().unwrap();
                    if health < self.stats.health.target {
                        self.display.damage_blend.target = 0.9;
                    }

                    self.stats.health.target = health;
                    self.stats.health_state = if self.stats.health.target >= 0.0 {
                        HealthState::Alive
                    } else {
                        HealthState::Dying
                    };
                },
                CensusProperties::MaxHealth => self.stats.max_health.target = codec.decode_f32().unwrap(),
                CensusProperties::Opacity => self.display.opacity.target = codec.decode_f32().unwrap(),
                CensusProperties::Radius => self.display.radius.target = codec.decode_f32().unwrap(),
                CensusProperties::Owners => {
                    self.display.owners.shallow = NonZeroU32::new(codec.decode_varuint().unwrap() as u32);
                    self.display.owners.deep = NonZeroU32::new(codec.decode_varuint().unwrap() as u32);
                    self.display.turret_index = codec.decode_varuint().unwrap() as usize;
                },
                CensusProperties::Ticks => self.time.server_ticks = codec.decode_varuint().unwrap(),
                _ => {}
            }
        }
    }

    fn render_projectile_body(&self, context: &mut Canvas2d, is_friendly: bool) {
        let (fill, stroke) = self.compute_body_fill(is_friendly);
        
        context.save();

        context.fill_style(fill);
        context.stroke_style(stroke);
        context.set_stroke_size(STROKE_SIZE);

        context.rotate(std::f32::consts::FRAC_PI_2);

        match self.display.entity_type {
            EntityType::Bullet => context.begin_arc(0.0, 0.0, self.display.radius.value, std::f32::consts::TAU),
            EntityType::Drone => context.begin_triangle(self.display.radius.value),
            EntityType::Trap => context.begin_star(3, self.display.radius.value / 1.5, self.display.radius.value * 1.75),
            _ => unreachable!("Non-projectile entity attempted rendering.")
        }
        
        context.fill();
        context.stroke();

        context.restore();
    }

    pub fn render_projectile(&mut self, context: &mut Canvas2d, is_friendly: bool, dt: f32) {
        self.time.ticks += 1;
        if matches!(self.stats.health_state, HealthState::Dying | HealthState::Dead) {
            self.destroy_projectile(context, is_friendly, dt);
        }

        context.save();
        
        context.translate(
            self.physics.position.value.x + self.physics.velocity.value.x, 
            self.physics.position.value.y + self.physics.velocity.value.y
        );
        context.rotate(self.physics.angle.value);
        context.global_alpha(self.display.opacity.value);
        context.set_stroke_size(STROKE_SIZE);

        self.render_projectile_body(context, is_friendly);

        context.restore();
    }

    fn destroy_projectile(&mut self, context: &mut Canvas2d, is_friendly: bool, dt: f32) {
        if fuzzy_compare!(self.display.opacity.value, 0.0, 1e-1) {
            self.stats.health_state = HealthState::Dead;
            return;
        }

        self.display.opacity.target = 0.0;
        self.display.radius.target *= 1.05;
    }
}