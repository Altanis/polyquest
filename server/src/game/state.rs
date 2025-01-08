use std::{cell::{RefCell, RefMut}, collections::HashMap};
use super::{collision::{collision::detect_collision, shg::SpatialHashGrid}, entity::base::Entity};

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
    pub counter: u32
}

impl GameState {
    pub fn get_next_id(&mut self) -> u32 {
        self.counter += 1;
        self.counter
    }

    pub fn insert_entity(&mut self, entity: Entity) {
        self.shg.insert(entity.id, entity.physics.position, entity.display.radius);
        self.entities.insert(entity.id, RefCell::new(entity));
    }

    pub fn get_entity(&mut self, id: u32) -> Option<RefMut<'_, Entity>> {
        self.entities.get(&id).map(|entity_ref| entity_ref.borrow_mut())
    }

    pub fn delete_entity(&mut self, id: u32) {
        self.shg.delete(id);
        self.entities.remove(&id);
    }

    pub fn tick(&mut self) {
        let ids: Vec<_> = self.entities.keys().copied().collect();
        for id in ids {
            // let dt = {
            //     let entity = self.get_entity(id).unwrap();
                
            //     let time = Instant::now();
            //     let delta_time = time.duration_since(entity.time.last_tick).as_millis_f32();
            //     (delta_time / MSPT as f32).min(1.5)
            // };

            Entity::tick(self, id);
        }

        let ids: Vec<_> = self.entities.keys().copied().collect();
        for id in ids {
            let mut this = self.entities[&id].borrow_mut();
            self.shg.reinsert(id, this.physics.position, this.display.radius);

            let collisions = self.shg.query_radius(id, this.physics.position, this.display.radius);

            for collision in collisions {
                let mut other = self.entities[&collision].borrow_mut();
                let resolve_collision = this.should_collide(&other);

                if resolve_collision && detect_collision(&this, &other) {
                    this.collide(&mut other);
                }
            }
        }
    }
}