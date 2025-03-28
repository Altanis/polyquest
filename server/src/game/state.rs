use std::{cell::{RefCell, RefMut}, collections::HashMap};
use shared::{game::{entity::{ClanInformation, EntityType}, orb::*}, rand, utils::{consts::ARENA_SIZE, vec2::Vector2D}};
use rand::Rng;
use crate::game::entity::base::AliveState;

use super::{clans::ClanState, entity::base::{DisplayComponent, Entity, PhysicsComponent, StatsComponent}, physics::{collision::detect_collision, shg::SpatialHashGrid}};

pub type EntityDataStructure = HashMap<u32, RefCell<Entity>>;

pub struct GameServer {
    states: Vec<GameState>,
}

impl GameServer {
    pub fn new(states: Vec<GameState>) -> GameServer {
        GameServer {
            states
        }
    }

    /// Gets a server given its id.
    pub fn get_server(&mut self) -> &mut GameState {
        &mut self.states[0]
    }

    pub fn tick(&mut self) {
        for state in self.states.iter_mut() {
            state.tick();
        }
    }
}

#[derive(Default)]
pub struct GameState {
    pub entities: EntityDataStructure,
    pub shg: SpatialHashGrid,
    pub clan_state: ClanState,
    pub counter: u32,
    pub mspt: f32,
    pub desired_orb_count: usize
}

impl GameState {
    pub fn get_next_id(&mut self) -> u32 {
        self.counter += 1;
        self.counter
    }

    pub fn get_random_position(&self) -> Vector2D {
        let (mut position, mut iterations) = (Vector2D::new(rand!(0.0, ARENA_SIZE), rand!(0.0, ARENA_SIZE)), -1);
        let (collision_detection, collision_radius) = (300.0, 50.0);

        while iterations >= 20 {
            iterations += 1;

            let nearby_entities = self.shg.query_radius(self.counter + 1, position, collision_detection);
            let mut is_position_valid = true;

            for nearby_entity in nearby_entities {
                let entity = self.entities[&nearby_entity].borrow();
                if (collision_radius + entity.display.radius) - position.distance(entity.physics.position) > 5.0 {
                    position = Vector2D::new(rand!(0.0, ARENA_SIZE), rand!(0.0, ARENA_SIZE));
                    is_position_valid = false;
                    break;
                }
            }

            if is_position_valid {
                break;
            }
        }

        position
    }

    pub fn insert_entity(&mut self, entity: Entity) {
        self.shg.insert(entity.id, entity.physics.position, entity.display.radius);
        self.entities.insert(entity.id, RefCell::new(entity));
    }

    pub fn get_entity(&self, id: u32) -> Option<RefMut<'_, Entity>> {
        self.entities.get(&id).map(|entity_ref| entity_ref.borrow_mut())
    }

    pub fn delete_entity(&mut self, id: u32) {
        let owned_entities = {
            let entity = self.entities.get(&id).unwrap().borrow_mut();
            entity.display.owned_entities.clone()
        };

        for id in owned_entities {
            self.delete_entity(id);
        }

        {
            let entity = self.entities.get(&id).unwrap().borrow_mut();

            if let Some(owners) = entity.display.owners {
                if owners.shallow != entity.id {
                    let mut shallow_owner = self.entities.get(&owners.shallow).unwrap().borrow_mut();
                    shallow_owner.display.owned_entities.retain(|&oid| id != oid);
                    
                    let turret_idx = entity.display.turret_idx;
                    if turret_idx != -1 
                        && let Some(turret) = shallow_owner.display.turret_identity.turrets.get_mut(turret_idx as usize) 
                        && turret.projectile_identity.projectile_type == entity.display.entity_type
                    {
                        turret.projectiles_spawned -= 1;
                    }
                }

                if owners.deep != entity.id && owners.deep != owners.shallow {
                    let mut deep_owner = self.entities.get(&owners.deep).unwrap().borrow_mut();
                    deep_owner.display.owned_entities.retain(|&oid| id != oid);
                    
                    let turret_idx = entity.display.turret_idx;
                    if turret_idx != -1 
                        && let Some(turret) = deep_owner.display.turret_identity.turrets.get_mut(turret_idx as usize)
                        && turret.projectile_identity.projectile_type == entity.display.entity_type
                    {
                        turret.projectiles_spawned -= 1;
                    }
                }
            }
        }

        let mut entity = self.entities.get(&id).unwrap().borrow_mut();

        if entity.display.entity_type == EntityType::Player {
            entity.stats.alive = AliveState::Uninitialized;
        } else {
            drop(entity);
            
            self.shg.delete(id);
            self.entities.remove(&id);
        }
    }

    fn spawn_random_shape(&mut self) {
        let position = self.get_random_position();
        let center_size = ARENA_SIZE * 0.15;
        let (center_min, center_max) = ((ARENA_SIZE - center_size) / 2.0, (ARENA_SIZE + center_size) / 2.0);
        
        let is_center = position.x >= center_min && position.x <= center_max && position.y >= center_min && position.y <= center_max;
        let identity = if is_center {
            OrbIdentityIds::Radiant
        } else {
            match rand!(0, 100) {
                0..=49 => OrbIdentityIds::Flickering,  // 50%
                50..=74 => OrbIdentityIds::Basic,      // 25%
                75..=84 => OrbIdentityIds::Stable,     // 10%
                85..=94 => OrbIdentityIds::Heavy,      // 10%
                _ => OrbIdentityIds::Radiant,          // 5%
            }
        };

        let identity: OrbIdentity = identity.try_into().unwrap();

        let entity = Entity {
            id: self.get_next_id(),
            physics: PhysicsComponent {
                position,
                collidable: true,
                absorption_factor: identity.absorption_factor,
                push_factor: identity.push_factor,
                bound_to_walls: true,
                angle: rand!(-3.13, 3.13),
                ..Default::default()
            },
            stats: StatsComponent {
                health: identity.max_health, max_health: identity.max_health, alive: AliveState::Alive, 
                last_damage_tick: 0, damage_reduction: 0.25,
                regen_per_tick: 0.0,
                damage_per_tick: identity.body_damage,
                reload: 0.0,
                speed: 0.0,
                lifetime: -1
            },
            display: DisplayComponent {
                entity_type: EntityType::Orb,
                opacity: 1.0,
                radius: identity.radius,
                orb_identity: identity,
                ..Default::default()
            },
            ..Default::default()
        };

        self.insert_entity(entity);
    }

    pub fn tick(&mut self) {
        let mspt = std::time::Instant::now();
        let mut current_orb_count = 0;

        self.clan_state.tick(&self.entities);

        let ids: Vec<_> = self.entities.keys().copied().collect();

        for id in ids {
            Entity::tick(self, id);
            if let Some(entity) = self.get_entity(id) && entity.display.entity_type == EntityType::Orb {
                current_orb_count += 1;
            }
        }

        let ids: Vec<_> = self.entities.keys().copied().collect();
        for id in ids {
            let mut this = self.entities[&id].borrow_mut();
            self.shg.reinsert(id, this.physics.position, this.display.radius);

            let collisions = self.shg.query_radius(id, this.physics.position, this.display.radius);

            for collision in collisions {
                let mut other = self.entities[&collision].borrow_mut();
                let resolve_collision = this.should_collide(&other);
                let is_colliding = detect_collision(&this, &other);

                if resolve_collision && is_colliding {
                    this.collide(&self.entities, &mut other);
                } else if this.display.entity_type.is_drone() && other.display.entity_type.is_drone() && is_colliding {
                    let angle = (this.physics.position - other.physics.position).angle();

                    // let (this_absorption_factor, other_absorption_factor) = (this.physics.absorption_factor, other.physics.absorption_factor);
                    // let (this_push_factor, other_push_factor) = (this.physics.push_factor, other.physics.push_factor);

                    this.physics.velocity += Vector2D::from_polar(4.0, angle);
                    other.physics.velocity -= Vector2D::from_polar(4.0, angle);

                    // this.physics.velocity += Vector2D::from_polar(this_absorption_factor * other_push_factor, angle);
                    // other.physics.velocity -= Vector2D::from_polar(other_absorption_factor * this_push_factor, angle);
                }
            }
        }

        let displacement = self.desired_orb_count - current_orb_count;
        for _ in 0..displacement {
            self.spawn_random_shape();
        }

        self.mspt = mspt.elapsed().as_millis_f32();
    }
}