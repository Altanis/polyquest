use shared::{connection::packets::{CensusProperties, Inputs}, normalize_angle, utils::{codec::BinaryCodec, consts::{ARENA_SIZE, FRICTION}, vec2::Vector2D}};
use strum::IntoEnumIterator;

use crate::game::state::EntityDataStructure;

use super::{ai::AIState, base::{AliveState, Entity, EntityConstruction}};

impl Entity {
    pub fn tick_projectile(&mut self, entities: &EntityDataStructure) -> Vec<EntityConstruction> {
        let constructions = vec![];

        if self.stats.lifetime != -1 && self.time.ticks >= self.stats.lifetime as u64
            || self.stats.health <= 0.0
        {
            self.stats.alive = AliveState::Dead;   
        }

        if let Some(ai) = &mut self.physics.ai
            && let Some(owner) = entities.get(&self.display.owners.unwrap().deep)
        {
            let owner = owner.borrow_mut();
            let is_shooting = owner.is_shooting();
            let is_repelling = owner.physics.inputs.is_set(Inputs::Repel);

            self.display.surroundings = owner.display.surroundings.clone();

            ai.speed = self.stats.speed;
            if (is_shooting || is_repelling) && ai.controllable {
                ai.state = AIState::Possessed(owner.physics.mouse);
            } else if matches!(ai.state, AIState::Possessed(_)) {
                ai.state = AIState::Idle;
            }

            if ai.state != AIState::Idle {
                self.physics.angle = ai.movement.angle() + if is_repelling { std::f32::consts::PI } else { 0.0 };
                let push_vec = ai.movement * ai.speed * if is_repelling { -1.0 } else { 1.0 };

                self.physics.additional_velocity.lerp_towards(push_vec, 0.15);
            } else {
                let mut delta = self.physics.position - owner.physics.position;
                let delta_magnitude = delta.magnitude();

                let unit_dist = delta_magnitude / 400.0;
                let resting = delta_magnitude <= (4.0 * owner.display.radius);

                if resting {
                    self.physics.angle += 0.01 * unit_dist;
                    self.physics.additional_velocity.lerp_towards(
                        Vector2D::from_polar(ai.speed / 3.0, self.physics.angle),
                        0.15
                    );
                } else {
                    let offset = delta.angle() + std::f32::consts::FRAC_PI_2;
                    delta.x = owner.physics.position.x + offset.cos() * owner.display.radius * 2.0
                        - self.physics.position.x;
                    delta.y = owner.physics.position.y + offset.sin() * owner.display.radius * 2.0
                        - self.physics.position.y;

                    self.physics.angle = delta.angle();
                    self.physics.additional_velocity.lerp_towards(
                        Vector2D::from_polar(ai.speed, self.physics.angle), 
                        0.15
                    );
                }
            }
        }

        self.physics.velocity *= 1.0 - FRICTION;
        self.physics.position += self.physics.velocity + self.physics.additional_velocity;
        self.physics.angle = normalize_angle!(self.physics.angle);
        
        if self.physics.bound_to_walls {
            self.physics.position.constrain(0.0, ARENA_SIZE);
        }

        constructions
    }

    pub fn take_projectile_census(&self, codec: &mut BinaryCodec, is_self: bool) {
        codec.encode_varuint(self.id as u64);
        codec.encode_varuint(self.display.entity_type as u64);        

        if !is_self && self.stats.alive != AliveState::Alive {
            codec.encode_varuint(0);
            return;
        }

        codec.encode_varuint(9);
        for property in CensusProperties::iter() {
            codec.encode_varuint(property.clone() as u64);

            match property {
                CensusProperties::Position => {
                    codec.encode_f32(self.physics.position.x);
                    codec.encode_f32(self.physics.position.y);
                },
                CensusProperties::Velocity => {
                    codec.encode_f32(self.physics.velocity.x);
                    codec.encode_f32(self.physics.velocity.y);
                },
                CensusProperties::Angle => codec.encode_f32(self.physics.angle),
                CensusProperties::Health => codec.encode_f32(self.stats.health),
                CensusProperties::MaxHealth => codec.encode_f32(self.stats.max_health),
                CensusProperties::Opacity => codec.encode_f32(self.display.opacity),
                CensusProperties::Radius => codec.encode_f32(self.display.radius),
                CensusProperties::Owners => {
                    let (shallow, deep) = self.display.owners.unwrap().to_tuple();
                    codec.encode_varuint(shallow as u64);
                    codec.encode_varuint(deep as u64);
                    codec.encode_varuint(self.display.turret_idx as u64);
                },
                CensusProperties::Ticks => codec.encode_varuint(self.time.ticks),
                _ => codec.backspace(),
            }
        }
    }
}