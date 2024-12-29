use std::collections::HashMap;
use derive_new::new as New;
use gloo::console::console;
use shared::{connection::packets::CensusProperties, game::{body::{BodyIdentity, BodyIdentityIds}, entity::{EntityType, InputFlags, UpgradeStats}, turret::{TurretIdentity, TurretIdentityIds, TurretStructure}}, lerp, lerp_angle, utils::{codec::BinaryCodec, interpolatable::Interpolatable, vec2::Vector2D}};
use strum::EnumCount;
use ui::{canvas2d::Canvas2d, core::UiElement, utils::color::Color};

use crate::{rendering::phases::GamePhase, world::World};

use super::theme::{ENEMY_FILL, ENEMY_STROKE, PLAYER_FILL, PLAYER_STROKE, STROKE_SIZE};

#[derive(Debug, Default)]
pub struct Game {
    pub surroundings: HashMap<u32, Entity>,
    pub self_entity: Entity,
    pub arena_size: f32
}

impl Game {
    pub fn get_mut_entity(&mut self, id: u32) -> &mut Entity {
        if self.self_entity.id == id {
            &mut self.self_entity
        } else {
            self.surroundings.get_mut(&id).unwrap()
        }
    }
}

#[derive(Debug, Default, Clone, New)]
pub struct PhysicsComponent {
    pub position: Interpolatable<Vector2D<f32>>,
    pub velocity: Interpolatable<Vector2D<f32>>,
    pub angle: Interpolatable<f32>,
    pub mouse: Vector2D<f32>,

    pub inputs: InputFlags
}

#[derive(Default, Debug, Clone)]
pub struct DisplayComponent {
    pub name: String,
    pub score: Interpolatable<f32>,

    pub stat_investments: [usize; UpgradeStats::COUNT],
    pub available_stat_points: usize,
    pub should_display_stats: bool,

    pub opacity: Interpolatable<f32>,
    pub fov: Interpolatable<f32>,

    pub entity_type: EntityType,
    pub body_identity: BodyIdentity,
    pub turret_identity: TurretStructure,
    pub radius: Interpolatable<f32>,

    pub z_index: isize
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
pub struct StatsComponent {
    pub health: Interpolatable<f32>,
    pub max_health: Interpolatable<f32>,
    pub health_state: HealthState,
    pub energy: Interpolatable<f32>,
    pub max_energy: Interpolatable<f32>
}

/// An entity which stores all these components, along with its id.
#[derive(Debug, Default, Clone)]
pub struct Entity {
    pub id: u32,
    pub physics: PhysicsComponent,
    pub display: DisplayComponent,
    pub stats: StatsComponent,
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
                    let angle = codec.decode_f32().unwrap();
                    if !is_self {
                        entity.physics.angle.target = angle;
                    }
                },
                CensusProperties::Name => entity.display.name = codec.decode_string().unwrap(),
                CensusProperties::Score => entity.display.score.target = codec.decode_varuint().unwrap() as f32,
                CensusProperties::Health => {
                    let old_state = entity.stats.health_state;
                    let health = codec.decode_f32().unwrap();

                    entity.stats.health.target = health;

                    if is_self {
                        if health > 0.0 && old_state != HealthState::Alive {
                            entity.stats.health_state = HealthState::Alive;
                            world.renderer.phase = GamePhase::Game;
                        } else if health <= 0.0 && old_state == HealthState::Alive {
                            entity.stats.health_state = HealthState::Dying;
                            world.renderer.phase = GamePhase::Death;
                        }
                    }
                },
                CensusProperties::MaxHealth => entity.stats.max_health.target = codec.decode_f32().unwrap(),
                CensusProperties::Energy => entity.stats.energy.target = codec.decode_f32().unwrap(),
                CensusProperties::MaxEnergy => entity.stats.max_energy.target = codec.decode_f32().unwrap(),
                CensusProperties::Stats => {
                    entity.display.available_stat_points = codec.decode_varuint().unwrap() as usize;
                    for i in 0..UpgradeStats::COUNT {
                        entity.display.stat_investments[i] = codec.decode_varuint().unwrap() as usize;
                    }
                },
                CensusProperties::Opacity => entity.display.opacity.target = codec.decode_f32().unwrap(),
                CensusProperties::Fov => entity.display.fov.target = codec.decode_f32().unwrap(),
                CensusProperties::Radius => entity.display.radius.target = codec.decode_f32().unwrap(),
                CensusProperties::Identity => {
                    let body_identity_id: BodyIdentityIds = (codec.decode_varuint().unwrap() as usize).try_into().unwrap();
                    entity.display.body_identity = body_identity_id.try_into().unwrap();

                    let turret_identity_id: TurretIdentityIds = (codec.decode_varuint().unwrap() as usize).try_into().unwrap();
                    entity.display.turret_identity = turret_identity_id.try_into().unwrap();
                },
            }
        }
    }

    /// Renders a taken given its identities.
    fn render_tank(&self, context: &mut Canvas2d, radius: f32, is_self: bool) {
        let fill = if is_self { PLAYER_FILL } else { ENEMY_FILL };
        let stroke = if is_self { PLAYER_STROKE } else { ENEMY_STROKE };

        context.fill_style(fill);
        context.stroke_style(stroke);
        
        context.begin_arc(0.0, 0.0, radius as f64, std::f64::consts::TAU);
        context.fill();
        context.stroke();

        context.stroke_style(Color::GREEN);
        context.begin_path();
        context.move_to(0.0, 0.0);
        context.line_to(radius, 0.0);
        context.stroke();
    }

    pub fn render(world: &mut World, id: u32, dt: f32) {
        let mut is_self = false;

        let mut entity = if id == world.game.self_entity.id {
            is_self = true;
            
            let entity = &mut world.game.self_entity;
            if entity.stats.health_state == HealthState::Dying {
                return Entity::destroy(world, id, dt);
            } else if entity.stats.health_state == HealthState::Dead {
                return world.renderer.phase = GamePhase::Death;
            }

            entity
        } else {
            if world.game.surroundings.get_mut(&id).unwrap().stats.health_state == HealthState::Dying {
                return Entity::destroy(world, id, dt);
            }

            world.game.surroundings.get_mut(&id).unwrap()
        };

        entity.time.ticks += 1; // todo move ticks in destroy fn as well
        let mut context = &mut world.renderer.canvas2d;

        context.save();
        
        context.translate(
            entity.physics.position.value.x + entity.physics.velocity.value.x, 
            entity.physics.position.value.y + entity.physics.velocity.value.y
        );
        context.rotate(entity.physics.angle.value);
        context.global_alpha(entity.display.opacity.value as f64);
        context.set_stroke_size(STROKE_SIZE);

        // entity.render_turrets(context);
        entity.render_tank(context, entity.display.radius.value, is_self);

        context.restore();
    }

    fn destroy(world: &mut World, id: u32, dt: f32) {
        let mut context = &mut world.renderer.canvas2d;
        let mut entity = if id == world.game.self_entity.id {
            &mut world.game.self_entity
        } else {
            &mut world.game.surroundings.get_mut(&id).unwrap()
        };

        entity.time.ticks += 1;
    }

    pub fn render_health_bar(world: &mut World, id: u32, dt: f32) {


    }

    pub fn render_nametag(world: &mut World, id: u32, dt: f32) {

    }

    pub fn lerp_all(&mut self, dt: f32, is_self: bool) {
        self.physics.position.value.lerp_towards(
            self.physics.position.target, 
            0.2 * dt
        );

        self.physics.velocity.value.lerp_towards(
            self.physics.velocity.target, 
            0.1 * dt
        );

        if !is_self {
            self.physics.angle.value = lerp_angle!(
                self.physics.angle.value, 
                self.physics.angle.target, 
                0.2 * dt
            );
        }

        self.display.score.value = lerp!(
            self.display.score.value as f32, 
            self.display.score.target as f32, 
            0.2 * dt
        ) as f32;
        
        self.stats.health.value = lerp!(
            self.stats.health.value, 
            self.stats.health.target, 
            0.2 * dt
        );
        
        self.stats.max_health.value = lerp!(
            self.stats.max_health.value, 
            self.stats.max_health.target, 
            0.2 * dt
        );
        
        self.stats.energy.value = lerp!(
            self.stats.energy.value, 
            self.stats.energy.target, 
            0.2 * dt
        );
        
        self.stats.max_energy.value = lerp!(
            self.stats.max_energy.value, 
            self.stats.max_energy.target, 
            0.2 * dt
        );
        
        self.display.opacity.value = lerp!(
            self.display.opacity.value, 
            self.display.opacity.target, 
            0.2 * dt
        );
        
        self.display.fov.value = lerp!(
            self.display.fov.value, 
            self.display.fov.target, 
            0.2 * dt
        );

        self.display.radius.value = lerp!(
            self.display.radius.value, 
            self.display.radius.target, 
            0.2 * dt
        );
    }
}