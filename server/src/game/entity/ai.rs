use core::f32;
use shared::utils::vec2::Vector2D;

use crate::game::state::EntityDataStructure;

/// The state of the AI.
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum AIState {
    /// The AI has no target.
    #[default]
    Idle,
    /// The AI is locked onto a target.
    Active(u32),
    /// The AI is being possessed by an actor (i.e., a mouse).
    Possessed(Vector2D<f32>)
}

#[derive(Default, Clone)]
pub struct AI {
    /// The aim of the AI.
    pub aim: Vector2D<f32>,
    /// The speed of the entity the AI is a part of.
    pub speed: f32,
    /// The final movement vector after a timestep.
    pub movement: Vector2D<f32>,
    /// The state of the AI.
    pub state: AIState,
    /// The entity which owns the AI.
    pub owner: u32,
    /// Whether or not the entity predicts its target's movements.
    pub prediction: bool,

}

impl AI {
    pub fn new(owner: u32, prediction: bool) -> AI {
        AI {
            owner,
            prediction,
            ..Default::default()
        }
    }

    fn get_target(&mut self, entities: &EntityDataStructure, position: Vector2D<f32>, surroundings: Vec<u32>) -> Option<u32> {
        if let AIState::Active(id) = self.state {
            if !surroundings.contains(&id) {
                self.state = AIState::Idle;
            } else {
                return Some(id);
            }
        }

        let mut surroundings = surroundings
            .iter()
            .filter(|&&id| id == self.owner)
            .map(|id| entities.get(id).unwrap().borrow_mut());

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
                if entity.display.owners.has_owner(self.owner) {
                    continue;
                }

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

    pub fn tick(&mut self, entities: &EntityDataStructure, position: Vector2D<f32>, surroundings: Vec<u32>) {
        if let AIState::Possessed(mouse) = self.state {
            self.aim = mouse;
            self.movement = Vector2D::from_polar(1.0, (self.aim - position).angle());
        } else {
            let Some(target) = self.get_target(entities, position, surroundings) else { return; };
            let entity = entities.get(&target).unwrap().borrow_mut();

            if self.prediction {
                
            } else {
                self.aim = entity.physics.position;
            }

            self.movement = Vector2D::from_polar(1.0, (self.aim - position).angle());
        }
    }
}