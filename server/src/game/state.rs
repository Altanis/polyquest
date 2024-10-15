use std::{borrow::BorrowMut, cell::{RefCell, RefMut}, collections::HashMap, time::Instant};

use crate::server::MSPT;

use super::entity::Entity;

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
    entities: EntityDataStructure,
    counter: u32
}

impl GameState {
    pub fn get_next_id(&mut self) -> u32 {
        self.counter += 1;
        self.counter - 1
    }

    pub fn insert_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.id, RefCell::new(entity));
    }

    pub fn get_entity(&mut self, id: u32) -> Option<RefMut<'_, Entity>> {
        self.entities.get(&id).map(|entity_ref| entity_ref.borrow_mut())
    }

    pub fn tick(&mut self) {
        let ids: Vec<_> = self.entities.keys().copied().collect();
        for id in ids {
            let dt = {
                let entity = self.get_entity(id).unwrap();
                
                let time = Instant::now();
                let delta_time = time.duration_since(entity.time.last_tick).as_millis_f32();
                (delta_time / MSPT as f32).min(1.5)
            };

            Entity::tick(dt, &self.entities, id);
        }
    }
}