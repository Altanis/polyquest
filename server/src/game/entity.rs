use std::{collections::HashMap, time::Instant};

use derive_new::new as New;
use shared::{connection::packets::{CensusProperties, ClientboundPackets, Inputs}, utils::{codec::BinaryCodec, vec2::Vector2D}};
use strum::IntoEnumIterator;

use crate::{connection::packets, server::FRICTION};

use super::state::EntityDataStructure;

#[derive(Default, Clone, New)]
pub struct InputFlags(u32);
impl InputFlags {
    pub fn is_set(&self, flag: Inputs) -> bool {
        self.0 & flag as u32 == flag as u32
    }

    pub fn set_flag(&mut self, flag: Inputs) {
        self.0 |= flag as u32;
    }

    pub fn clear_flag(&mut self, flag: Inputs) {
        self.0 &= !(flag as u32);
    }
}

#[derive(Default, Clone, New)]
pub struct PhysicsComponent {
    pub position: Vector2D,
    pub velocity: Vector2D,
    pub inputs: InputFlags,
    pub mouse: Vector2D
}

#[derive(Default, Clone, New)]
pub struct NametagComponent {
    pub name: String
}

#[derive(Default, Clone, New)]
pub struct StatsComponent {
    pub score: u32,
    pub health: f32,
    pub max_health: f32,
    pub energy: f32,
    pub max_energy: f32,
    pub alive: bool
}

#[derive(Clone, New)]
pub struct TimeComponent {
    pub ticks: u64,
    pub last_tick: Instant
}

impl Default for TimeComponent {
    fn default() -> Self {
        TimeComponent { ticks: 0, last_tick: Instant::now() }
    }
}

#[derive(Default, Clone, New)]
pub struct ConnectionComponent {
    pub outgoing_packets: Vec<BinaryCodec>
}

/// An entity which stores all these components, along with its id.
#[derive(Default, Clone)]
pub struct Entity {
    pub id: u32,
    pub physics: PhysicsComponent,
    pub nametag: NametagComponent,
    pub stats: StatsComponent,
    pub time: TimeComponent,
    pub connection: ConnectionComponent
}

impl Entity {
    pub fn take_census(&self, codec: &mut BinaryCodec, is_self: bool) {
        if !is_self && !self.stats.alive {
            codec.encode_varuint(0);
            return;
        }

        codec.encode_varuint(5);
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
                _ => codec.backspace(),
            }
        }
    }

    pub fn tick(dt: f32, entities: &EntityDataStructure, id: u32) {
        let mut self_entity = entities.get(&id).unwrap().borrow_mut();

        self_entity.time.ticks += 1;

        if self_entity.stats.health <= 0.0 {
            self_entity.stats.alive = false;
        } else if self_entity.stats.health <= self_entity.stats.max_health {
            // regeneration maybe
        }

        if !self_entity.physics.velocity.is_zero(1e-1) {
            let velocity = self_entity.physics.velocity;
            self_entity.physics.position += velocity * dt;
            self_entity.physics.velocity *= FRICTION;
        }

        let update_packet = packets::form_update_packet(&self_entity, entities);
        self_entity.connection.outgoing_packets.push(update_packet);
    }
}