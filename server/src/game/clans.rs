use shared::game::entity::ClanInformation;

use super::state::EntityDataStructure;

/// State that maintains clan operations.
#[derive(Default)]
pub struct ClanState {
    pub clans: Vec<ClanInformation>,
    counter: u32
}

impl ClanState {
    fn get_next_clan_id(&mut self) -> u32 {
        self.counter += 1;
        self.counter
    }

    pub fn create_clan(&mut self, name: String, description: String, max_members: usize, owner: u32) -> u32 {
        let id = self.get_next_clan_id();

        self.clans.push(ClanInformation {
            id,
            owner,
            name,
            description,
            members: vec![owner],
            pending_members: vec![],
            max_members
        });

        id
    }

    pub fn tick(&mut self, entities: &EntityDataStructure) {
        for clan in self.clans.iter_mut() {
            let mut i = 0;
            while i < clan.members.len() {
                let member = &clan.members[i];
                if let Some(entity) = entities.get(member) {
                    entity.borrow_mut().display.clan_id = Some(clan.id);
                    i += 1;
                } else {
                    clan.members.remove(i);
                }
            }
        }
    }
}