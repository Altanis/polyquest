use std::collections::HashMap;
use derive_new::new as New;
use gloo::console::console;
use shared::{connection::packets::CensusProperties, game::{body::{BodyIdentity, BodyIdentityIds}, entity::{EntityType, InputFlags, UpgradeStats}, turret::{TurretIdentityIds, TurretStructure}}, utils::{codec::BinaryCodec, interpolatable::Interpolatable, vec2::Vector2D}};
use strum::EnumCount;
use ui::core::UiElement;

use crate::{rendering::phases::GamePhase, world::World};

#[derive(Debug, Default)]
pub struct Game {
    pub surroundings: HashMap<u32, Entity>,
    pub self_entity: Entity
}


#[derive(Debug, Default, Clone, New)]
pub struct PhysicsComponent {
    pub position: Interpolatable<Vector2D<f32>>,
    pub velocity: Interpolatable<Vector2D<f32>>,
    pub mouse: Interpolatable<Vector2D<f32>>,

    pub inputs: InputFlags
}

#[derive(Default, Debug, Clone)]
pub struct DisplayComponent {
    pub name: String,
    pub score: Interpolatable<u32>,
    pub level: Interpolatable<u32>,

    pub health: Interpolatable<f32>,
    pub max_health: Interpolatable<f32>,
    pub health_state: HealthState,
    pub energy: Interpolatable<f32>,
    pub max_energy: Interpolatable<f32>,

    pub stat_investments: [usize; UpgradeStats::COUNT],
    pub available_stat_points: usize,

    pub opacity: Interpolatable<f32>,
    pub fov: Interpolatable<f32>,

    pub entity_type: EntityType,
    pub body_identity: BodyIdentity,
    pub turret_identity: TurretStructure,
    pub radius: f32
}


#[derive(Debug, Default, Clone, Copy, New, PartialEq)]
pub enum HealthState {
    Alive,
    Dying,
    #[default]
    Dead
}

#[derive(Debug, Clone, New)]
pub struct TimeComponent {
    pub ticks: u64,
    pub last_tick: f64
}

impl Default for TimeComponent {
    fn default() -> TimeComponent {
        TimeComponent { ticks: 0, last_tick: 0.0 }
    }
}

#[derive(Debug, Default, Clone, New)]
pub struct ConnectionComponent {
    pub outgoing_packets: Vec<BinaryCodec>
}

#[derive(Debug, Default, Clone, New)]
pub struct ViewComponent {
    pub fov: f32,
    pub surroundings: Vec<u32>
}

/// An entity which stores all these components, along with its id.
#[derive(Debug, Default, Clone)]
pub struct Entity {
    pub id: u32,
    pub physics: PhysicsComponent,
    pub display: DisplayComponent,
    pub time: TimeComponent,
    pub connection: ConnectionComponent
}

impl Entity {
    pub fn parse_census(world: &mut World, codec: &mut BinaryCodec, is_self: bool) {
        let entity = if is_self {
            let entity = &mut world.game.self_entity;
            entity.id = codec.decode_varuint().unwrap() as u32;

            entity
        } else {
            let id = codec.decode_varuint().unwrap() as u32;
            let entity = world.game.surroundings.entry(id)
                .or_insert_with(|| Entity { id, ..Default::default() });

            entity
        };

        entity.display.entity_type = (codec.decode_varuint().unwrap() as u8).try_into().unwrap();
        let properties = codec.decode_varuint().unwrap();

        for _ in 0..properties {
            let property: CensusProperties = (codec.decode_varuint().unwrap() as u8).try_into().unwrap();

            match property {
                CensusProperties::Position => {
                    entity.physics.position.target = Vector2D::new(
                        codec.decode_f32().unwrap(),
                        codec.decode_f32().unwrap()
                    );
                },
                CensusProperties::Velocity => {
                    entity.physics.velocity.target = Vector2D::new(
                        codec.decode_f32().unwrap(),
                        codec.decode_f32().unwrap()
                    );
                },
                CensusProperties::Angle => {
                    entity.physics.mouse.target = Vector2D::from_polar(1.0, codec.decode_f32().unwrap());
                },
                CensusProperties::Health => {
                    let old_state = entity.display.health_state;
                    let health = codec.decode_f32().unwrap();

                    entity.display.health.target = health;

                    if is_self {
                        if health > 0.0 && old_state != HealthState::Alive {
                            entity.display.health_state = HealthState::Alive;
                            world.renderer.phase = GamePhase::Game;
                            world.renderer.body.set_children(vec![]);
                        } else if health <= 0.0 && old_state == HealthState::Alive {
                            entity.display.health_state = HealthState::Dying;
                            world.renderer.phase = GamePhase::Death;
                            world.renderer.body.set_children(vec![]);                            
                        }
                    }
                },
                CensusProperties::MaxHealth => entity.display.max_health.target = codec.decode_f32().unwrap(),
                CensusProperties::Name => entity.display.name = codec.decode_string().unwrap(),
                CensusProperties::Identity => {
                    let body_identity_id: BodyIdentityIds = (codec.decode_varuint().unwrap() as usize).try_into().unwrap();
                    entity.display.body_identity = body_identity_id.try_into().unwrap();

                    let turret_identity_id: TurretIdentityIds = (codec.decode_varuint().unwrap() as usize).try_into().unwrap();
                    entity.display.turret_identity = turret_identity_id.try_into().unwrap();
                }
            }
        }
    }
}