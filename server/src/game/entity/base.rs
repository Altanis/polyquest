use std::{collections::HashSet, time::Instant};
use derive_new::new as New;
use shared::{game::{body::{get_body_base_identity, BodyIdentity}, entity::{EntityType, InputFlags, Ownership, TankUpgrades, UpgradeStats}, turret::{get_turret_base_identity, TurretStructure}}, utils::{codec::BinaryCodec, vec2::Vector2D}};
use strum::EnumCount;

use crate::game::state::GameState;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AliveState {
    Uninitialized,
    Alive,
    Dead
}

#[derive(Clone)]
pub struct PhysicsComponent {
    pub position: Vector2D<f32>,
    pub velocity: Vector2D<f32>,
    pub additional_velocity: Vector2D<f32>,
    pub angle: f32,
    pub mouse: Vector2D<f32>,
    pub inputs: InputFlags,
    pub collidable: bool,
    pub absorption_factor: f32,
    pub push_factor: f32,
    pub collisions: HashSet<u32>
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
    pub owners: Ownership,
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

    pub last_damage_tick: u64,
    pub damage_reduction: f32,

    pub regen_per_tick: f32,
    pub damage_per_tick: f32,
    pub reload: f32,
    pub speed: f32,
    pub lifetime: isize,

    pub alive: AliveState,

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

pub enum EntityConstruction {
    ProjectileConstruction {
        speed: (f32, f32),
        penetration: f32,
        damage: f32,
        radius: f32,
        angle: f32,
        position: Vector2D<f32>,
        lifetime: isize,
        owners: Ownership,
        kb_factors: (f32, f32)
    },
    TankConstruction
}

impl Entity {
    pub fn take_census(&self, codec: &mut BinaryCodec, is_self: bool) {
        match self.display.entity_type {
            EntityType::Player => self.take_tank_census(codec, is_self),
            EntityType::Projectile => self.take_projectile_census(codec, is_self),
        }
    }

    pub fn tick(state: &mut GameState, id: u32) {        
        let (constructions, alive_state, entity_type) = {
            let mut entity = state.entities.get(&id).unwrap().borrow_mut();
            entity.time.ticks += 1;
            entity.physics.collisions.clear();

            (match entity.display.entity_type {
                EntityType::Player => entity.tick_tank(&state.entities),
                EntityType::Projectile => entity.tick_projectile(&state.entities),
            }, entity.stats.alive, entity.display.entity_type)
        };

        if alive_state == AliveState::Dead {
            if entity_type == EntityType::Player {
                state.entities.get(&id).unwrap().borrow_mut()
                    .stats.alive = AliveState::Uninitialized;
            } else {
                state.delete_entity(id);
            }
        }

        for construction in constructions {
            let id = state.get_next_id();

            match construction {
                EntityConstruction::ProjectileConstruction { .. } => {
                    state.insert_entity(Entity::generate_projectile_entity(id, construction));
                },
                EntityConstruction::TankConstruction => {}
            }
        }
    }

    pub fn should_collide(&self, other: &Entity) -> bool {
        if !self.physics.collidable || !other.physics.collidable {
            return false;
        }

        if other.display.owners.has_owner(self.id) || self.display.owners.has_owner(self.id) {
            return false;
        }

        true
    }

    pub fn collide(&mut self, other: &mut Entity) {
        if (self.stats.health <= 0.0 || other.stats.health <= 0.0) ||
            (self.stats.alive != AliveState::Alive || other.stats.alive != AliveState::Alive) ||
            (self.physics.collisions.contains(&other.id) || other.physics.collisions.contains(&self.id)) ||
            (self.stats.damage_reduction == 0.0 && other.stats.damage_reduction == 0.0) ||
            (
                (self.stats.damage_per_tick == 0.0 && self.physics.push_factor == 0.0) ||
                (other.stats.damage_per_tick == 0.0 && other.physics.push_factor == 0.0)
            )
        {
            return;
        }

        let kb_magnitude = self.physics.absorption_factor * self.physics.push_factor;
        self.physics.velocity += Vector2D::from_polar(kb_magnitude, (self.physics.position - other.physics.position).angle());

        let mut df1 = self.stats.damage_per_tick * self.stats.damage_reduction 
            * if matches!(self.display.entity_type, EntityType::Player) { 1.5 } else { 1.0 };
        let mut df2 = other.stats.damage_per_tick * other.stats.damage_reduction 
            * if matches!(other.display.entity_type, EntityType::Player) { 1.5 } else { 1.0 };

        let ratio = (1.0 - self.stats.health / df2).max(1.0 - other.stats.health / df1);
        if ratio > 0.0 {
            df1 *= 1.0 - ratio;
            df2 *= 1.0 - ratio;
        }

        if df1 != 0.0 {
            other.stats.last_damage_tick = other.time.ticks;
            other.stats.health -= df1;
        }

        if df2 != 0.0 {
            self.stats.last_damage_tick = self.time.ticks;
            self.stats.health -= df2;
        }

        // check for kill

        self.physics.collisions.insert(other.id);
        other.physics.collisions.insert(self.id);
    }

    fn generate_projectile_entity(id: u32, construction: EntityConstruction) -> Entity {
        let EntityConstruction::ProjectileConstruction { 
            speed, 
            penetration, 
            damage, 
            radius, 
            angle, 
            position, 
            lifetime,
            owners,
            kb_factors
        } = construction else { panic!("impossibility"); };

        Entity {
            id,
            physics: PhysicsComponent {
                position,
                velocity: Vector2D::from_polar(speed.1, angle),
                additional_velocity: Vector2D::from_polar(speed.0, angle),
                angle,
                mouse: Vector2D::ZERO,
                inputs: InputFlags::new(0),
                collidable: true,
                absorption_factor: kb_factors.0,
                push_factor: kb_factors.1,
                collisions: HashSet::new()
            },
            display: DisplayComponent {
                name: "".to_string(),
                level: 1,
                score: 0,
                stat_investments: [0; _],
                available_stat_points: 0,
                upgrades: TankUpgrades::default(),
                opacity: 1.0,
                fov: 0.0,
                surroundings: vec![],
                entity_type: EntityType::Projectile,
                body_identity: get_body_base_identity(),
                turret_identity: get_turret_base_identity(),
                owners,
                radius
            },
            stats: StatsComponent {
                health: penetration, max_health: penetration, alive: AliveState::Alive, 
                last_damage_tick: 0, damage_reduction: 0.25,
                regen_per_tick: 0.0,
                damage_per_tick: damage,
                reload: 0.0,
                speed: speed.1,
                energy: 0.0, max_energy: 0.0,
                lifetime
            },
            time: TimeComponent {
                ticks: 0,
                last_tick: Instant::now()
            },
            connection: ConnectionComponent {
                outgoing_packets: vec![]
            }
        }
    }
}