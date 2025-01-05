use shared::{connection::packets::CensusProperties, utils::{codec::BinaryCodec, consts::FRICTION}};
use strum::IntoEnumIterator;

use crate::game::state::EntityDataStructure;

use super::base::{AliveState, Entity, EntityConstruction};

impl Entity {
    pub fn tick_projectile(&mut self, entities: &EntityDataStructure) -> Vec<EntityConstruction> {
        let constructions = vec![];

        if self.stats.lifetime != -1 && self.time.ticks >= self.stats.lifetime as u64 {
            self.stats.alive = AliveState::Dead;   
        }

        self.physics.velocity *= 1.0 - FRICTION;
        self.physics.position += self.physics.velocity + self.physics.additional_velocity;

        // self.physics.position.constrain(0.0, ARENA_SIZE);

        constructions
    }

    pub fn take_projectile_census(&self, codec: &mut BinaryCodec, is_self: bool) {
        codec.encode_varuint(self.id as u64);
        codec.encode_varuint(self.display.entity_type as u64);        

        if !is_self && self.stats.alive != AliveState::Alive {
            codec.encode_varuint(0);
            return;
        }

        codec.encode_varuint(8);
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
                CensusProperties::Angle => codec.encode_f32(self.physics.mouse.angle()),
                CensusProperties::Health => codec.encode_f32(self.stats.health),
                CensusProperties::MaxHealth => codec.encode_f32(self.stats.max_health),
                CensusProperties::Opacity => codec.encode_f32(self.display.opacity),
                CensusProperties::Radius => codec.encode_f32(self.display.radius),
                CensusProperties::Owners => {
                    let (shallow, deep) = self.display.owners.to_tuple();
                    codec.encode_varuint(shallow as u64);
                    codec.encode_varuint(deep as u64);
                },
                _ => codec.backspace(),
            }
        }
    }
}