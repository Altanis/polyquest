use std::{collections::HashSet, num::NonZeroU32, time::Instant};
use derive_new::new as New;
use shared::{game::{body::{get_body_base_identity, BodyIdentity}, entity::{get_min_score_from_level, EntityType, InputFlags, Notification, Ownership, TankUpgrades, UpgradeStats, BASE_TANK_RADIUS}, turret::{get_turret_base_identity, TurretStructure}}, utils::{codec::BinaryCodec, color::Color, consts::MAX_LEVEL, vec2::Vector2D}};
use strum::EnumCount;

use crate::game::state::{EntityDataStructure, GameState};

use super::ai::AI;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AliveState {
    Uninitialized,
    Alive,
    Dead
}

#[derive(Clone)]
pub struct PhysicsComponent {
    pub position: Vector2D,
    pub velocity: Vector2D,
    pub additional_velocity: Vector2D,
    pub angle: f32,
    pub mouse: Vector2D,
    pub inputs: InputFlags,
    pub has_moved: bool,
    pub collidable: bool,
    pub absorption_factor: f32,
    pub push_factor: f32,
    pub collisions: HashSet<u32>,
    pub ai: Option<AI>
}

#[derive(Clone)]
pub struct DisplayComponent {
    pub name: String,
    pub score: usize,
    pub level: usize,

    pub invincible: bool,

    pub stat_investments: [usize; UpgradeStats::COUNT],
    pub available_stat_points: usize,
    pub upgrades: TankUpgrades,

    pub opacity: f32,
    pub fov: f32,
    pub surroundings: Vec<u32>,
    pub notifications: Vec<Notification>,
    pub killer: Option<NonZeroU32>,

    pub entity_type: EntityType,
    pub body_identity: BodyIdentity,
    pub turret_identity: TurretStructure,
    pub turret_idx: isize,
    pub owners: Ownership,
    pub owned_entities: Vec<u32>,
    pub radius: f32
}

#[derive(Default, Clone)]
pub struct TimeComponent {
    pub ticks: u64,
    pub spawn_tick: u64
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
        position: Vector2D,
        lifetime: isize,
        owners: Ownership,
        turret_idx: isize,
        kb_factors: (f32, f32),
        ai: Option<AI>,
        projectile_type: EntityType
    },
    TankConstruction
}

impl Entity {
    pub fn from_id(id: u32) -> Self {        
        Entity {
            id,
            physics: PhysicsComponent {
                position: Vector2D::ZERO,
                velocity: Vector2D::ZERO,
                additional_velocity: Vector2D::ZERO,
                angle: 0.0,
                mouse: Vector2D::ZERO,
                inputs: InputFlags::new(0),
                has_moved: false,
                collidable: true,
                absorption_factor: get_body_base_identity().absorption_factor,
                push_factor: 8.0,
                collisions: HashSet::new(),
                ai: None
            },
            display: DisplayComponent {
                name: "".to_string(),
                level: 1,
                score: 0,
                invincible: false,
                stat_investments: [0; _],
                available_stat_points: 0,
                upgrades: TankUpgrades::default(),
                opacity: 1.0,
                fov: 0.35,
                surroundings: vec![],
                notifications: vec![],
                killer: NonZeroU32::new(0),
                entity_type: EntityType::Player,
                body_identity: get_body_base_identity(),
                turret_identity: get_turret_base_identity(),
                turret_idx: -1,
                owners: Ownership::from_single_owner(0),
                owned_entities: vec![],
                radius: BASE_TANK_RADIUS
            },
            stats: StatsComponent {
                health: 0.0, max_health: 0.0, alive: AliveState::Uninitialized, 
                last_damage_tick: 0, damage_reduction: 1.0,
                regen_per_tick: 0.0,
                damage_per_tick: 0.0,
                reload: 0.0,
                speed: 0.0,
                lifetime: -1,
                energy: 0.0, max_energy: 0.0,
            },
            time: TimeComponent {
                ticks: 0,
                spawn_tick: 0
            },
            connection: ConnectionComponent {
                outgoing_packets: vec![]
            }
        }
    }

    pub fn take_census(&self, codec: &mut BinaryCodec, is_self: bool) {
        match self.display.entity_type {
            EntityType::Player => self.take_tank_census(codec, is_self),
            EntityType::Bullet | EntityType::Drone | EntityType::Trap => self.take_projectile_census(codec, is_self),
        }
    }

    pub fn tick(state: &mut GameState, id: u32) {
        if !state.entities.contains_key(&id) { return; }

        let (constructions, alive_state) = {
            let mut entity = state.entities.get(&id).unwrap().borrow_mut();
            entity.time.ticks += 1;
            entity.physics.collisions.clear();

            let constructions = match entity.display.entity_type {
                EntityType::Player => entity.tick_tank(&state.entities, &state.shg),
                EntityType::Bullet | EntityType::Drone | EntityType::Trap => entity.tick_projectile(&state.entities),
            };

            let position = entity.physics.position;
            let surroundings = entity.display.surroundings.clone();

            if let Some(ai) = &mut entity.physics.ai {
                ai.tick(&state.entities, position, surroundings);
            }

            (constructions, entity.stats.alive)
        };

        if alive_state == AliveState::Dead {
            state.delete_entity(id);
        }

        for construction in constructions {
            let id = state.get_next_id();

            match construction {
                EntityConstruction::ProjectileConstruction { owners, .. } => {
                    state.insert_entity(Entity::generate_projectile_entity(id, construction));
                    
                    if let Some(shallow_owner) = &owners.shallow {
                        state.entities.get(&shallow_owner.get()).unwrap().borrow_mut()
                            .display.owned_entities.push(id);
                    }

                    if !owners.has_singular_owner() && let Some(deep_owner) = &owners.deep {
                        state.entities.get(&deep_owner.get()).unwrap().borrow_mut()
                            .display.owned_entities.push(id);
                    }
                },
                EntityConstruction::TankConstruction => {}
            }
        }
    }

    pub fn should_collide(&self, other: &Entity) -> bool {
        if !self.physics.collidable || !other.physics.collidable {
            return false;
        }

        if self.display.owners.is_related(other.id, other.display.owners)
            || other.display.owners.is_related(self.id, self.display.owners)
        {
            return false;
        }

        true
    }

    pub fn collide(&mut self, entities: &EntityDataStructure, other: &mut Entity) {
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

        self.physics.velocity += Vector2D::from_polar(
            self.physics.absorption_factor * other.physics.push_factor, 
            (self.physics.position - other.physics.position).angle()
        );

        other.physics.velocity += Vector2D::from_polar(
            other.physics.absorption_factor * self.physics.push_factor,
            (other.physics.position - self.physics.position).angle()
        );

        let mut df1 = self.stats.damage_per_tick * other.stats.damage_reduction;
        let mut df2 = other.stats.damage_per_tick * self.stats.damage_reduction;

        if self.display.entity_type == EntityType::Player && other.display.entity_type == EntityType::Player {
            df1 *= 1.5;
            df2 *= 1.5;
        }

        if df1 != 0.0 {
            other.stats.last_damage_tick = other.time.ticks;
            other.stats.health -= df1;
        }

        if df2 != 0.0 {
            self.stats.last_damage_tick = self.time.ticks;
            self.stats.health -= df2;
        }

        self.physics.collisions.insert(other.id);
        other.physics.collisions.insert(self.id);

        if self.stats.health <= 0.0 {
            other.kill(self);

            if let Some(id) = other.display.owners.shallow
                && let Some(entity) = entities.get(&id.into())
            {
                entity.borrow_mut().kill(self);
            }

            if other.display.owners.shallow != other.display.owners.deep
                && let Some(id) = other.display.owners.deep
                && let Some(entity) = entities.get(&id.into())
            {
                entity.borrow_mut().kill(self);
            }
        }
        
        if other.stats.health <= 0.0 {
            self.kill(other);

            if let Some(id) = self.display.owners.shallow
                && let Some(entity) = entities.get(&id.into())
            {
                entity.borrow_mut().kill(other);
            }

            if self.display.owners.shallow != self.display.owners.deep
                && let Some(id) = self.display.owners.deep
                && let Some(entity) = entities.get(&id.into())
            {
                entity.borrow_mut().kill(other);
            }
        }
    }

    pub fn kill(&mut self, other: &mut Entity) {
        if self.display.entity_type == EntityType::Player && other.display.entity_type == EntityType::Player {
            self.display.notifications.push(Notification {
                message: format!("You killed {}", other.display.name),
                color: Color::BLACK,
                lifetime: 150,
                ..Default::default()
            });

            self.display.score += other.display.score.min(get_min_score_from_level(MAX_LEVEL));
        }

        other.display.killer = NonZeroU32::new(self.id);
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
            turret_idx,
            kb_factors,
            ai,
            projectile_type
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
                has_moved: false,
                collidable: true,
                absorption_factor: kb_factors.0,
                push_factor: kb_factors.1,
                collisions: HashSet::new(),
                ai
            },
            display: DisplayComponent {
                name: "".to_string(),
                level: 1,
                score: 0,
                invincible: false,
                stat_investments: [0; _],
                available_stat_points: 0,
                upgrades: TankUpgrades::default(),
                opacity: 1.0,
                fov: 0.0,
                surroundings: vec![],
                notifications: vec![],
                killer: NonZeroU32::new(0),
                entity_type: projectile_type,
                body_identity: get_body_base_identity(),
                turret_identity: get_turret_base_identity(),
                turret_idx,
                owners,
                owned_entities: vec![],
                radius
            },
            stats: StatsComponent {
                health: penetration, max_health: penetration, alive: AliveState::Alive, 
                last_damage_tick: 0, damage_reduction: 0.25,
                regen_per_tick: 0.0,
                damage_per_tick: damage,
                reload: 0.0,
                speed: speed.0,
                energy: 0.0, max_energy: 0.0,
                lifetime
            },
            time: TimeComponent {
                ticks: 0,
                spawn_tick: 0
            },
            connection: ConnectionComponent {
                outgoing_packets: vec![]
            }
        }
    }
}