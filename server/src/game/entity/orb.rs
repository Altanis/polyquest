use shared::{connection::packets::CensusProperties, game::orb::OrbIdentityIds, normalize_angle, rand, utils::{codec::BinaryCodec, consts::{ARENA_SIZE, FRICTION}}};
use rand::Rng;
use strum::IntoEnumIterator;

use crate::game::state::EntityDataStructure;

use super::base::{AliveState, Entity, EntityConstruction};

impl Entity {
    pub fn tick_orb(&mut self, entities: &EntityDataStructure) -> Vec<EntityConstruction> {
        let constructions = vec![];

        if self.stats.health <= 0.0 {
            self.stats.alive = AliveState::Dead;
        }

        if self.display.orb_identity.id == OrbIdentityIds::Flickering {
            self.display.opacity = rand!(0.0, 0.7) + 0.3;
        }

        self.physics.velocity *= 1.0 - FRICTION;
        self.physics.position += self.physics.velocity + self.physics.additional_velocity;
        
        if self.physics.bound_to_walls {
            self.physics.position.constrain(0.0, ARENA_SIZE);
        }

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