use std::time::Instant;

use derive_new::new as New;
use shared::{connection::packets::{CensusProperties, Inputs}, constrain, game::{body::{BodyIdentity, BodyIdentityIds}, entity::{EntityType, InputFlags, UpgradeStats}, turret::{TurretIdentityIds, TurretStructure}}, utils::{codec::BinaryCodec, vec2::Vector2D}};
use strum::{EnumCount, IntoEnumIterator};

use crate::{connection::packets, server::FRICTION};

use super::state::EntityDataStructure;

#[derive(Clone)]
pub struct PhysicsComponent {
    pub position: Vector2D<f32>,
    pub velocity: Vector2D<f32>,
    pub mouse: Vector2D<f32>,

    pub inputs: InputFlags
}

#[derive(Clone)]
pub struct DisplayComponent {
    pub name: String,
    pub score: u32,
    pub level: u32,

    pub health: f32,
    pub max_health: f32,
    pub regen_per_tick: f32,
    pub alive: bool,
    pub energy: f32,
    pub max_energy: f32,

    pub stat_investments: [usize; UpgradeStats::COUNT],
    pub available_stat_points: usize,

    pub opacity: f32,
    pub fov: f32,
    pub surroundings: Vec<u32>,

    pub entity_type: EntityType,
    pub body_identity: BodyIdentity,
    pub turret_identity: TurretStructure,
    pub radius: f32
}

#[derive(Clone)]
pub struct TimeComponent {
    pub ticks: u64,
    pub last_tick: Instant
}

impl Default for TimeComponent {
    fn default() -> TimeComponent {
        TimeComponent { ticks: 0, last_tick: Instant::now() }
    }
}

#[derive(Default, Clone, New)]
pub struct ConnectionComponent {
    pub outgoing_packets: Vec<BinaryCodec>
}

/// An entity which stores all these components, along with its id.
#[derive(Clone)]
pub struct Entity {
    pub id: u32,
    pub physics: PhysicsComponent,
    pub display: DisplayComponent,
    pub time: TimeComponent,
    pub connection: ConnectionComponent
}

impl Entity {
    pub fn take_census(&self, codec: &mut BinaryCodec, is_self: bool) {
        codec.encode_varuint(self.id as u64);
        codec.encode_varuint(self.display.entity_type as u64);        

        if !is_self && !self.display.alive {
            codec.encode_varuint(0);
            return;
        }

        codec.encode_varuint(7);
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
                CensusProperties::Health => codec.encode_f32(self.display.health),
                CensusProperties::MaxHealth => codec.encode_f32(self.display.max_health),
                CensusProperties::Name => codec.encode_string(self.display.name.clone()),
                CensusProperties::Identity => {
                    codec.encode_varuint(self.display.body_identity.id as u64);
                    codec.encode_varuint(self.display.turret_identity.id as u64);
                }
                _ => codec.backspace(),
            }
        }
    }

    pub fn tick(dt: f32, entities: &EntityDataStructure, id: u32) {
        let mut self_entity = entities.get(&id).unwrap().borrow_mut();

        self_entity.time.ticks += 1;

        if self_entity.display.health <= 0.0 {
            self_entity.display.alive = false;
        } else if self_entity.display.health <= self_entity.display.max_health {
            // regeneration maybe
        }

        if !self_entity.physics.velocity.is_zero(1e-1) {
            let velocity = self_entity.physics.velocity;
            self_entity.physics.position += velocity * dt;
            self_entity.physics.velocity *= FRICTION;
        }

        Entity::update_display(&mut self_entity, dt);

        let update_packet = packets::form_update_packet(&self_entity, entities);
        self_entity.connection.outgoing_packets.push(update_packet);
    }

    fn update_display(&mut self, dt: f32) {
        let is_shooting = self.physics.inputs.is_set(Inputs::Shoot);

        // Invisibility
        if self.physics.velocity.is_zero(3.0) && !is_shooting {
            if self.display.body_identity.invisibility_rate != -1.0 && self.display.opacity > 0.0 {
                self.display.opacity -= self.display.body_identity.invisibility_rate * dt;
                self.display.opacity = constrain!(0.0, self.display.opacity, 1.0);
            }
        } else if self.display.body_identity.invisibility_rate != -1.0 && self.display.opacity < 1.0 {
            self.display.opacity += self.display.body_identity.invisibility_rate * dt;
            self.display.opacity = constrain!(0.0, self.display.opacity, 1.0);
        }

        // Upgrade Level

        // Health Regen
        self.display.regen_per_tick = (self.display.max_health 
            * 4.0 
            * (self.display.stat_investments[UpgradeStats::HealthRegen as usize] as f32)
            + self.display.max_health
        ) / 25000.0;

        // Max Health
        let prev_health_ratio = self.display.health / self.display.max_health;
        self.display.max_health = self.display.body_identity.max_health 
            + (2.0 * (self.display.level - 1) as f32)
            + (20.0 * self.display.stat_investments[UpgradeStats::MaxHealth as usize] as f32);
        self.display.health = self.display.max_health * prev_health_ratio;

        // Body Damage

        // Reload

        // Movement Speed

        // FoV
    }
}