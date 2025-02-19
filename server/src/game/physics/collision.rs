use crate::game::entity::base::Entity;

pub fn detect_collision(entity1: &Entity, entity2: &Entity) -> bool
{
    let distance = entity1.physics.position.distance(entity2.physics.position);
    let overlap = (entity1.display.radius + entity2.display.radius) - distance;

    if overlap <= 0.0 {
        return false;
    }

    true
}