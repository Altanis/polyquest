use shared::{connection::packets::CensusProperties, game::orb::OrbIdentityIds, normalize_angle, rand, utils::{codec::BinaryCodec, consts::ARENA_SIZE, vec2::Vector2D}};
use rand::Rng;
use strum::IntoEnumIterator;

use crate::game::state::EntityDataStructure;
use super::base::{AliveState, Entity, EntityConstruction};

impl Entity {
    pub fn tick_orb(&mut self, _: &EntityDataStructure) -> Vec<EntityConstruction> {
        let constructions = vec![];

        self.base_tick();

        if self.display.orb_identity.id == OrbIdentityIds::Flickering {
            self.display.opacity = rand!(0.0, 0.7) + 0.3;
        }

        let (soft_border_left, soft_border_right) = (ARENA_SIZE / 7.0, 6.0 * ARENA_SIZE / 7.0);
        if self.physics.position.x < soft_border_left
            || self.physics.position.x > soft_border_right
        {
            self.physics.velocity.x *= -1.0;
        } else if self.physics.position.y < soft_border_left
            || self.physics.position.y > soft_border_right
        {
            self.physics.velocity.y *= -1.0;
        }

        self.physics.angle = normalize_angle!(self.physics.angle + 0.01 * self.display.orb_identity.angular_speed);
        self.physics.velocity += Vector2D::from_polar(
            0.03 * self.display.orb_identity.linear_speed,
            self.physics.angle,
        );

        self.stats.regen_per_tick = self.stats.max_health / 25000.0;

        constructions
    }

    pub fn take_orb_census(&self, codec: &mut BinaryCodec) {
        codec.encode_varuint(self.id as u64);
        codec.encode_varuint(self.display.entity_type as u64);        

        if self.stats.alive != AliveState::Alive {
            codec.encode_varuint(0);
            return;
        }

        codec.encode_varuint(9);
        for property in CensusProperties::iter() {
            codec.encode_varuint(property.clone() as u64);

            match property {
                CensusProperties::Position => {
                    codec.encode_f32(self.physics.position.x);
                    codec.encode_f32(self.physics.position.y);
                },
                CensusProperties::Velocity => {
                    codec.encode_f32(self.physics.velocity.x);
                    codec.encode_f32(self.physics.velocity.y);
                },
                CensusProperties::Angle => codec.encode_f32(self.physics.angle),
                CensusProperties::Health => codec.encode_f32(self.stats.health),
                CensusProperties::MaxHealth => codec.encode_f32(self.stats.max_health),
                CensusProperties::Opacity => codec.encode_f32(self.display.opacity),
                CensusProperties::Radius => codec.encode_f32(self.display.radius),
                CensusProperties::Ticks => codec.encode_varuint(self.time.ticks),
                CensusProperties::Identity => {
                    codec.encode_varuint(self.display.orb_identity.id as u64);
                },
                _ => codec.backspace(),
            }
        }
    }
}