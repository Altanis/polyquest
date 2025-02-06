use core::f32;
use shared::{game::entity::{EntityType, Ownership}, utils::vec2::Vector2D};

use crate::game::state::EntityDataStructure;

use super::base::AliveState;

/// The state of the AI.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum AIState {
    /// The AI has no target.
    #[default]
    Idle,
    /// The AI is locked onto a target.
    Active(u32),
    /// The AI is being possessed by an actor (i.e., a mouse).
    Possessed(Vector2D)
}

#[derive(Default, Clone)]
pub struct AI {
    /// The aim of the AI.
    pub aim: Vector2D,
    /// The speed of the entity the AI is a part of.
    pub speed: f32,
    /// The final movement vector after a timestep.
    pub movement: Vector2D,
    /// The state of the AI.
    pub state: AIState,
    /// The entity which owns the AI.
    pub ownership: Ownership,
    /// Whether or not the entity predicts its target's movements.
    pub prediction: bool,
    /// Whether or not this AI is possessable.
    pub controllable: bool
}

impl AI {
    pub fn new(ownership: Ownership, prediction: bool, controllable: bool) -> AI {
        AI {
            ownership,
            prediction,
            controllable,
            ..Default::default()
        }
    }

    fn get_target(&mut self, entities: &EntityDataStructure, position: Vector2D, surroundings: Vec<u32>) -> Option<u32> {
        if let AIState::Active(id) = self.state {
            if !surroundings.contains(&id) || entities.get(&id).is_none() {
                self.state = AIState::Idle;
            } else {
                return Some(id);
            }
        }

        let mut surroundings = surroundings
            .iter()
            .filter(|&&id| !self.ownership.has_owner(id) && entities.get(&id).is_some())
            .map(|id| entities.get(id).unwrap().borrow_mut())
            .filter(|entity| {
                if entity.stats.alive != AliveState::Alive || !matches!(entity.display.entity_type, EntityType::Player | EntityType::Orb) {
                    return false;
                } else if let Some(owners) = entity.display.owners {                    
                    if self.ownership.has_owner(owners.shallow) || self.ownership.has_owner(owners.deep)
                        || owners.has_owner(self.ownership.shallow) || owners.has_owner(self.ownership.deep)
                    {
                        return false;
                    }
                }

                true
            });

        if self.state == AIState::Idle {
            let (mut min_distance, mut id) = {
                let entity = surroundings.next();
                if let Some(entity) = entity {
                    (entity.physics.position.distance(position), entity.id)
                } else {
                    return None;
                }
            };

            for entity in surroundings {
                let distance = entity.physics.position.distance(position);
                if min_distance > distance {
                    min_distance = distance;
                    id = entity.id;
                }
            }

            self.state = AIState::Active(id);
            return Some(id);
        }

        None
    }

    pub fn tick(&mut self, entities: &EntityDataStructure, self_position: Vector2D, owner_position: Vector2D, surroundings: Vec<u32>) {
        if let AIState::Possessed(mouse) = self.state {
            self.aim = mouse;
            self.movement = Vector2D::from_polar(1.0, (self.aim - self_position).angle());
        } else {
            let Some(target) = self.get_target(entities, owner_position, surroundings) else { return; };
            let entity = entities.get(&target).unwrap().borrow_mut();

            if self.prediction {
                
            } else {
                self.aim = entity.physics.position;
            }

            self.movement = Vector2D::from_polar(1.0, (self.aim - self_position).angle());
        }
    }
}