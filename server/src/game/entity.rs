use std::time::Instant;

use derive_new::new as New;
use shared::{connection::packets::{CensusProperties, Inputs}, game::{body::{BodyIdentity, BodyIdentityIds}, entity::{get_min_score_from_level, EntityType, InputFlags, TankUpgrades, UpgradeStats, BASE_TANK_RADIUS, LEVEL_TO_SCORE_TABLE}, turret::{TurretIdentity, TurretIdentityIds, TurretStructure}}, utils::{codec::BinaryCodec, consts::{ARENA_SIZE, FRICTION, MAX_LEVEL}, vec2::Vector2D}};
use strum::{EnumCount, IntoEnumIterator};

use crate::connection::packets;

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
    pub score: usize,
    pub level: usize,

    pub stat_investments: [usize; UpgradeStats::COUNT],
    pub available_stat_points: usize,
    pub upgrades: TankUpgrades,

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

#[derive(Clone)]
pub struct StatsComponent {
    pub health: f32,
    pub max_health: f32,

    pub regen_per_tick: f32,
    pub damage_per_tick: f32,
    pub reload: f32,
    pub speed: f32,

    pub alive: bool,

    pub energy: f32,
    pub max_energy: f32,
}

/// An entity which stores all these components, along with its id.
#[derive(Clone)]
pub struct Entity {
    pub id: u32,
    pub physics: PhysicsComponent,
    pub display: DisplayComponent,
    pub stats: StatsComponent,
    pub time: TimeComponent,
    pub connection: ConnectionComponent
}

impl Entity {
    pub fn take_census(&self, codec: &mut BinaryCodec, is_self: bool) {
        codec.encode_varuint(self.id as u64);
        codec.encode_varuint(self.display.entity_type as u64);        

        if !is_self && !self.stats.alive {
            codec.encode_varuint(0);
            return;
        }

        // TODO(Altanis): This census leaks unwanted information
        // of other players. Give them privacy.
        codec.encode_varuint(15);
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
                CensusProperties::Name => codec.encode_string(self.display.name.clone()),
                CensusProperties::Score => codec.encode_varuint(self.display.score as u64),
                CensusProperties::Health => codec.encode_f32(self.stats.health),
                CensusProperties::MaxHealth => codec.encode_f32(self.stats.max_health),
                CensusProperties::Energy => codec.encode_f32(self.stats.energy),
                CensusProperties::MaxEnergy => codec.encode_f32(self.stats.max_energy),
                CensusProperties::Stats => {
                    codec.encode_varuint(self.display.available_stat_points as u64);
                    for i in 0..UpgradeStats::COUNT {
                        codec.encode_varuint(self.display.stat_investments[i] as u64);
                    }
                },
                CensusProperties::Upgrades => {
                    codec.encode_varuint(self.display.upgrades.body.len() as u64);
                    for &upgrade in self.display.upgrades.body.iter() {
                        codec.encode_varuint(upgrade as u64);
                    }

                    codec.encode_varuint(self.display.upgrades.turret.len() as u64);
                    for &upgrade in self.display.upgrades.turret.iter() {
                        codec.encode_varuint(upgrade as u64);
                    }
                },
                CensusProperties::Opacity => codec.encode_f32(self.display.opacity),
                CensusProperties::Fov => codec.encode_f32(self.display.fov),
                CensusProperties::Radius => codec.encode_f32(self.display.radius),
                CensusProperties::Identity => {
                    codec.encode_varuint(self.display.body_identity.id as u64);
                    codec.encode_varuint(self.display.turret_identity.id as u64);
                }
                _ => codec.backspace(),
            }
        }
    }

    fn check_for_upgrades(&mut self) {
        for &upgrade in self.display.body_identity.upgrades.iter() {
            let upgrade_identity: BodyIdentity = upgrade.try_into().unwrap();
            if self.display.level >= upgrade_identity.level_requirement
                && !self.display.upgrades.body.contains(&upgrade)
            {
                self.display.upgrades.body.push(upgrade);
            }
        }

        for &upgrade in self.display.turret_identity.upgrades.iter() {
            let upgrade_identity: TurretStructure = upgrade.try_into().unwrap();
            if self.display.level >= upgrade_identity.level_requirement
                && !self.display.upgrades.turret.contains(&upgrade)
            {
                self.display.upgrades.turret.push(upgrade);
            }
        }
    }

    pub fn update_level(&mut self, level: usize) {
        if self.display.level == level || level > MAX_LEVEL {
            return;
        }

        self.display.level = level;
        self.display.radius = BASE_TANK_RADIUS * 1.007_f32.powf((self.display.level - 1) as f32);

        if level < 29 || level % 3 == 0 {
            self.display.available_stat_points += 1;
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

        let (mut movement, mut shooting) = (Vector2D::ZERO, false);

        for flag in Inputs::iter() {
            if self_entity.physics.inputs.is_set(flag) {
                match flag {
                    Inputs::Up => movement.y -= 1.0,
                    Inputs::Down => movement.y += 1.0,
                    Inputs::Left => movement.x -= 1.0,
                    Inputs::Right => movement.x += 1.0,
                    Inputs::Shoot => shooting = true,
                    Inputs::LevelUp => {
                        let new_level = self_entity.display.level + 1;
                        self_entity.update_level(new_level);
                        
                        self_entity.display.score = get_min_score_from_level(self_entity.display.level).max(self_entity.display.score);
                    }
                }
            }
        }

        movement.set_magnitude(self_entity.stats.speed);
        self_entity.physics.velocity += movement;

        if !self_entity.physics.velocity.is_zero(1e-1) {
            let velocity = self_entity.physics.velocity;
            self_entity.physics.position += velocity;
            self_entity.physics.velocity *= 1.0 - FRICTION;
        }

        self_entity.physics.position.constrain(0.0, ARENA_SIZE);

        Entity::update_display(&mut self_entity, dt);

        let update_packet = packets::form_update_packet(&self_entity, entities);
        self_entity.connection.outgoing_packets.push(update_packet);
    }

    fn update_display(&mut self, dt: f32) {
        let is_shooting = self.physics.inputs.is_set(Inputs::Shoot);

        // Invisibility
        if self.physics.velocity.is_zero(3.0) && !is_shooting {
            if self.display.body_identity.invisibility_rate != -1.0 && self.display.opacity > 0.0 {
                self.display.opacity -= self.display.body_identity.invisibility_rate;
                self.display.opacity = self.display.opacity.clamp(0.0, 1.0);
            }
        } else if self.display.body_identity.invisibility_rate != -1.0 && self.display.opacity < 1.0 {
            self.display.opacity += self.display.body_identity.invisibility_rate;
            self.display.opacity = self.display.opacity.clamp(0.0, 1.0);
        }

        // Upgrade Level
        let mut new_level = self.display.level;
        while new_level < MAX_LEVEL && get_min_score_from_level(new_level + 1) <= self.display.score {
            new_level += 1;
        }
        self.update_level(new_level);
        self.check_for_upgrades();

        // Health Regen
        self.stats.regen_per_tick = (self.stats.max_health 
            * 4.0 
            * (self.display.stat_investments[UpgradeStats::HealthRegen as usize] as f32)
            + self.stats.max_health
        ) / 25000.0;

        // Max Health
        let prev_health_ratio = self.stats.health / self.stats.max_health;
        self.stats.max_health = self.display.body_identity.max_health 
            + (2.0 * (self.display.level - 1) as f32)
            + (20.0 * self.display.stat_investments[UpgradeStats::MaxHealth as usize] as f32);
        self.stats.health = self.stats.max_health * prev_health_ratio;

        // Body Damage
        self.stats.damage_per_tick = (self.display.stat_investments[UpgradeStats::BodyDamage as usize] as f32
            * 6.0 + 20.0) * self.display.body_identity.body_damage;

        // Reload
        self.stats.reload = 15.0 * 0.914_f32.powf(self.display.stat_investments[UpgradeStats::Reload as usize] as f32);

        // Movement Speed
        self.stats.speed = self.display.body_identity.speed * 2.55 *
            1.07_f32.powf(self.display.stat_investments[UpgradeStats::MovementSpeed as usize] as f32)
            / 1.015_f32.powf((self.display.level - 1) as f32);

        // FoV
        self.display.fov = (0.55 * self.display.body_identity.fov) / 1.01f32.powf((self.display.level as f32 - 1.0) / 2.0);
    }
}