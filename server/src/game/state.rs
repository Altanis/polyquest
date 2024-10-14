use std::collections::HashMap;

use super::entity::Entity;

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
    entities: HashMap<u32, Entity>,
    counter: u32
}

impl GameState {
    pub fn get_next_id(&mut self) -> u32 {
        self.counter += 1;
        self.counter - 1
    }

    pub fn insert_entity(&mut self, entity: Entity) {
        self.entities.insert(entity.id, entity);
    }

    pub fn get_entity(&mut self, id: u32) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn get_mut_entity(&mut self, id: u32) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    pub fn tick(&mut self) {
        
    }
}