use derive_new::new as New;
use shared::{connection::packets::{CensusProperties, ClientboundPackets, Inputs}, utils::{codec::BinaryCodec, vec2::Vector2D}};
use strum::IntoEnumIterator;

use crate::server::FRICTION;

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

#[derive(Default, Clone, New)]
pub struct TimeComponent {
    pub ticks: u64
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
    pub fn take_census(&self, codec: &mut BinaryCodec) {
        if !self.stats.alive {
            codec.encode_varuint(0);
            return;
        }

        codec.encode_varuint(18);
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

    pub fn tick(&mut self, dt: f32) {
        self.time.ticks += 1;

        if self.stats.health <= 0.0 {
            self.stats.alive = false;
        } else if self.stats.health <= self.stats.max_health {
            // regeneration maybe
        }

        if !self.physics.velocity.is_zero(1e-1) {
            self.physics.position += self.physics.velocity * dt;
            self.physics.velocity *= FRICTION;
        }

        let mut codec = BinaryCodec::new();
        self.take_census(&mut codec);
        self.connection.outgoing_packets.push(codec)
    }
}