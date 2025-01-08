use shared::{connection::packets::{CensusProperties, Inputs}, game::{body::BodyIdentity, entity::{get_min_score_from_level, Ownership, UpgradeStats, BASE_TANK_RADIUS, FICTITIOUS_TANK_RADIUS}, turret::TurretStructure}, rand, utils::{codec::BinaryCodec, consts::{ARENA_SIZE, FRICTION, MAX_LEVEL}, vec2::Vector2D}};
use strum::{EnumCount, IntoEnumIterator};
use rand::Rng;
use crate::{connection::packets, game::state::EntityDataStructure};

use super::base::{AliveState, Entity, EntityConstruction};

impl Entity {
    pub fn tick_tank(&mut self, entities: &EntityDataStructure) -> Vec<EntityConstruction> {
        let mut constructions = vec![];

        if self.stats.alive == AliveState::Alive && self.stats.health <= 0.0 {
            self.stats.alive = AliveState::Dead;
        } else if self.stats.health <= self.stats.max_health {
            // regeneration maybe
        }

        if self.stats.alive == AliveState::Alive {
            let (mut movement, mut shooting) = (Vector2D::ZERO, false);

            for flag in Inputs::iter() {
                if self.physics.inputs.is_set(flag) {
                    match flag {
                        Inputs::Up => movement.y -= 1.0,
                        Inputs::Down => movement.y += 1.0,
                        Inputs::Left => movement.x -= 1.0,
                        Inputs::Right => movement.x += 1.0,
                        Inputs::Shoot => shooting = true,
                        Inputs::LevelUp => {
                            let new_level = self.display.level + 1;
                            self.update_level(new_level);
                            
                            self.display.score = get_min_score_from_level(self.display.level).max(self.display.score);
                        }
                    }
                }
            }

            constructions.append(&mut self.handle_shooting(shooting));
    
            movement.set_magnitude(self.stats.speed);
            self.physics.velocity += movement;
    
            self.physics.velocity *= 1.0 - FRICTION;
            self.physics.position += self.physics.velocity;
    
            self.physics.position.constrain(0.0, ARENA_SIZE);
    
            self.update_display();
        }

        let update_packet = packets::form_update_packet(self, entities);
        self.connection.outgoing_packets.push(update_packet);

        constructions
    }

    fn handle_shooting(&mut self, shooting: bool) -> Vec<EntityConstruction> {
        let mut constructions = vec![];

        for (i, turret) in self.display.turret_identity.turrets.iter_mut().enumerate() {
            if !turret.can_fire(self.stats.reload, shooting) { continue; }

            let base_speed = (20.0 
                + (3.0 * self.display.stat_investments[UpgradeStats::ProjectileSpeed as usize] as f32))
                * turret.projectile_identity.speed;
            let initial_speed = base_speed + 30.0 - rand!(0.0, 1.0) * turret.projectile_identity.scatter_rate;

            let penetration = (1.5 * self.display.stat_investments[UpgradeStats::ProjectilePenetration as usize] as f32 + 2.0)
                * turret.projectile_identity.health;
            let damage = (7.0 + self.display.stat_investments[UpgradeStats::ProjectileDamage as usize] as f32 * 3.0)
                * turret.projectile_identity.damage;

            let radius = (turret.width / 2.0) * (self.display.radius / FICTITIOUS_TANK_RADIUS);

            let projectile_angle = self.physics.angle + turret.angle + (std::f32::consts::PI / 180.0)
                * turret.projectile_identity.scatter_rate
                * (rand!(0.0, 1.0) - 0.5)
                * 10.0;

            let push_factor = ((7.0 / 3.0) + self.display.stat_investments[UpgradeStats::ProjectileDamage as usize] as f32) 
                * turret.projectile_identity.damage 
                * turret.projectile_identity.absorption_factor;

            let mut position = self.physics.position;
            position += Vector2D::from_polar(turret.length * (FICTITIOUS_TANK_RADIUS / BASE_TANK_RADIUS) * (self.display.radius / BASE_TANK_RADIUS), projectile_angle);
            // position -= *Vector2D::from_polar(turret.x_offset * (self.display.radius / BASE_TANK_RADIUS), projectile_angle).swap();
            position += Vector2D::from_polar(turret.y_offset, projectile_angle);

            self.physics.velocity -= Vector2D::from_polar(turret.recoil, projectile_angle);

            constructions.push(EntityConstruction::ProjectileConstruction {
                angle: projectile_angle,
                speed: (base_speed, initial_speed),
                penetration,
                damage,
                radius,
                position,
                lifetime: (turret.projectile_identity.lifetime * 72) as isize,
                owners: Ownership::from_single_owner(self.id),
                kb_factors: (turret.projectile_identity.absorption_factor, push_factor)
            });
        }

        constructions
    }

    fn update_display(&mut self) {
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

    pub fn take_tank_census(&self, codec: &mut BinaryCodec, is_self: bool) {
        codec.encode_varuint(self.id as u64);
        codec.encode_varuint(self.display.entity_type as u64);        

        if !is_self && self.stats.alive != AliveState::Alive {
            codec.encode_varuint(0);
            return;
        }

        if is_self {
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
        } else {
            codec.encode_varuint(12);
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
                    CensusProperties::Opacity => codec.encode_f32(self.display.opacity),
                    CensusProperties::Radius => codec.encode_f32(self.display.radius),
                    CensusProperties::Identity => {
                        codec.encode_varuint(self.display.body_identity.id as u64);
                        codec.encode_varuint(self.display.turret_identity.id as u64);
                    }
                    _ => codec.backspace(),
                }
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
}