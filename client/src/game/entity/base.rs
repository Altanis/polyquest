use std::collections::HashMap;
use derive_new::new as New;
use gloo::console::console;
use shared::{connection::packets::CensusProperties, fuzzy_compare, game::{body::{BodyIdentity, BodyIdentityIds, BodyRenderingHints}, entity::{EntityType, InputFlags, Notification, Ownership, TankUpgrades, UpgradeStats, BASE_TANK_RADIUS}, turret::{TurretIdentity, TurretIdentityIds, TurretRenderingHints, TurretStructure}}, lerp, lerp_angle, prettify_score, utils::{codec::BinaryCodec, color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use strum::EnumCount;
use ui::{canvas2d::Canvas2d, core::UiElement, elements::tank::Tank};

use crate::{rendering::phases::GamePhase, world::World};

use shared::game::theme::{ENEMY_FILL, ENEMY_STROKE, PLAYER_FILL, PLAYER_STROKE, SMASHER_GUARD_FILL, SMASHER_GUARD_STROKE, STROKE_SIZE, TURRET_FILL, TURRET_STROKE};

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

    pub inputs: InputFlags,
    pub auto_fire: bool
}

#[derive(Default, Debug, Clone)]
pub struct DisplayComponent {
    pub name: String,
    pub score: Interpolatable<f32>,

    pub stat_investments: [usize; UpgradeStats::COUNT],
    pub available_stat_points: usize,
    pub should_display_stats: bool,
    pub upgrades: TankUpgrades,
    pub notifications: Vec<Notification>,

    pub opacity: Interpolatable<f32>,
    pub fov: Interpolatable<f32>,

    pub entity_type: EntityType,
    pub body_identity: BodyIdentity,
    pub turret_identity: TurretStructure,
    pub owners: Ownership,
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
    pub fn parse_census(world: &mut World, codec: &mut BinaryCodec, is_self: bool) -> u32 {
        let entity = if is_self {
            let entity = &mut world.game.self_entity;
            entity.id = codec.decode_varuint().unwrap() as u32;

            entity
        } else {
            let id = codec.decode_varuint().unwrap() as u32;
            world.game.surroundings.entry(id)
                .or_insert_with(|| Entity { id, ..Default::default() })
        };

        let old_state = entity.stats.health_state;

        entity.display.entity_type = (codec.decode_varuint().unwrap() as u8).try_into().unwrap();
        match entity.display.entity_type {
            EntityType::Player => entity.parse_tank_census(codec, is_self),
            EntityType::Projectile => entity.parse_projectile_census(codec, is_self),
        }

        if is_self {
            if entity.stats.health.target > 0.0 && old_state != HealthState::Alive {
                world.renderer.phase = GamePhase::Game;
            } else if entity.stats.health.value <= 0.0 && old_state == HealthState::Alive {
                world.renderer.phase = GamePhase::Death;
            }
        }

        entity.id
    }

    pub fn render(world: &mut World, id: u32, dt: f32) {
        let self_id = world.game.self_entity.id;
        let mut entity = if id == self_id {
            &mut world.game.self_entity
        } else {
            world.game.surroundings.get_mut(&id).unwrap()
        };

        let is_friendly = id == self_id || entity.display.owners.has_owner(self_id);
        match entity.display.entity_type {
            EntityType::Player => entity.render_tank(&mut world.renderer.canvas2d, is_friendly, dt),
            EntityType::Projectile => entity.render_projectile(&mut world.renderer.canvas2d, is_friendly, dt),
        }
    }

    pub fn compute_body_fill(&self, is_friendly: bool) -> (Color, Color) {
        let fill = if is_friendly { PLAYER_FILL } else { ENEMY_FILL };
        let stroke = if is_friendly { PLAYER_STROKE } else { ENEMY_STROKE };

        (fill, stroke)
    }


    pub fn render_health_bar(world: &mut World, id: u32, dt: f32) {


    }

    pub fn render_nametag(world: &mut World, id: u32, dt: f32) {
        let entity = world.game.surroundings.get_mut(&id).unwrap();
        let context = &world.renderer.canvas2d;

        if entity.display.entity_type != EntityType::Player || entity.stats.health_state != HealthState::Alive {
            return;
        }

        context.save();
        context.translate(
            entity.physics.position.value.x + entity.physics.velocity.value.x, 
            entity.physics.position.value.y + entity.physics.velocity.value.y
        );


        context.set_miter_limit(2.0);
        context.fill_style(Color::WHITE);
        context.stroke_style(Color::BLACK);
        context.set_text_align("center");

        context.save();

        let font = entity.display.radius.value / 1.3;
        context.set_font(&format!("bold {}px Ubuntu", font));
        context.set_stroke_size((font / 5.0).ceil());

        context.translate(0.0, -entity.display.radius.value - 60.0);
        context.stroke_text(&entity.display.name);
        context.fill_text(&entity.display.name);
        context.restore();

        if !fuzzy_compare!(entity.display.score.value, 0.0, 1e-3) {
            context.save();

            let font = entity.display.radius.value / 2.0;
            context.set_font(&format!("bold {}px Ubuntu", font));
            context.set_stroke_size((font / 5.0).ceil());
    
            context.translate(0.0, -entity.display.radius.value - 20.0);
            context.stroke_text(&prettify_score!(entity.display.score.value));
            context.fill_text(&prettify_score!(entity.display.score.value));
            context.restore();
        }

        context.restore();
    }

    pub fn lerp_all(&mut self, dt: f32, is_self: bool) {
        let factor = if self.time.ticks <= 1 { 1.0 } else { 0.2 * dt };

        self.physics.position.value.lerp_towards(
            self.physics.position.target, 
            factor
        );

        self.physics.velocity.value.lerp_towards(
            self.physics.velocity.target, 
            factor
        );

        if !is_self {
            self.physics.angle.value = lerp_angle!(
                self.physics.angle.value, 
                self.physics.angle.target, 
                factor
            );
        }

        self.display.score.value = lerp!(
            self.display.score.value, 
            self.display.score.target, 
            factor
        );
        
        self.stats.health.value = lerp!(
            self.stats.health.value, 
            self.stats.health.target, 
            factor
        );
        
        self.stats.max_health.value = lerp!(
            self.stats.max_health.value, 
            self.stats.max_health.target, 
            factor
        );
        
        self.stats.energy.value = lerp!(
            self.stats.energy.value, 
            self.stats.energy.target, 
            factor
        );
        
        self.stats.max_energy.value = lerp!(
            self.stats.max_energy.value, 
            self.stats.max_energy.target, 
            factor
        );
        
        self.display.opacity.value = lerp!(
            self.display.opacity.value, 
            self.display.opacity.target, 
            factor
        );
        
        self.display.fov.value = lerp!(
            self.display.fov.value, 
            self.display.fov.target, 
            factor
        );

        self.display.radius.value = lerp!(
            self.display.radius.value, 
            self.display.radius.target, 
            factor
        );
    }
}