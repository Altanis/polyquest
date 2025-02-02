use std::collections::HashMap;
use derive_new::new as New;
use gloo::console::console;
use gloo_utils::window;
use shared::{connection::packets::CensusProperties, fuzzy_compare, game::{body::{BodyIdentity, BodyIdentityIds, BodyRenderingHints}, entity::{EntityType, InputFlags, Notification, Ownership, UpgradeStats, BASE_TANK_RADIUS}, orb::OrbIdentity, theme::{BAR_BACKGROUND, HIGH_HEALTH_BAR, LOW_HEALTH_BAR, MEDIUM_HEALTH_BAR}, turret::{TurretIdentity, TurretIdentityIds, TurretRenderingHints, TurretStructure}}, lerp, lerp_angle, prettify_score, utils::{codec::BinaryCodec, color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use strum::EnumCount;
use ui::{canvas2d::Canvas2d, core::UiElement, elements::tank::Tank};

use crate::{rendering::phases::GamePhase, world::World};

use shared::game::theme::{ENEMY_FILL, ENEMY_STROKE, PLAYER_FILL, PLAYER_STROKE, SMASHER_GUARD_FILL, SMASHER_GUARD_STROKE, STROKE_SIZE, TURRET_FILL, TURRET_STROKE};

#[derive(Debug, Default)]
pub struct LeaderboardState {
    pub entries: Vec<(usize, String, BodyIdentityIds, TurretIdentityIds)>,
    pub angle: Interpolatable<f32>,
    pub intersection: Interpolatable<Vector2D>,
    pub arrow_opacity: Interpolatable<f32>
}

#[derive(Debug, Default)]
pub struct Game {
    pub surroundings: HashMap<u32, Entity>,
    pub self_entity: Entity,
    pub leaderboard: LeaderboardState,

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
    pub position: Interpolatable<Vector2D>,
    pub velocity: Interpolatable<Vector2D>,
    pub angle: Interpolatable<f32>,
    pub mouse: Vector2D,

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
    pub upgrades: Vec<i32>,
    pub notifications: Vec<Notification>,
    pub kills: usize,
    
    pub opacity: Interpolatable<f32>,
    pub fov: Interpolatable<f32>,

    pub entity_type: EntityType,
    pub body_identity: BodyIdentity,
    pub turret_identity: TurretStructure,
    pub orb_identity: OrbIdentity,
    pub turret_lengths: Vec<Interpolatable<f32>>,
    pub turret_index: usize,
    pub owners: Ownership,
    pub radius: Interpolatable<f32>,
    pub damage_blend: Interpolatable<f32>,
    pub invincible: bool,

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
    pub server_ticks: u64,
    pub last_tick: f64
}

impl Default for TimeComponent {
    fn default() -> TimeComponent {
        TimeComponent { ticks: 0, server_ticks: 0, last_tick: 0.0 }
    }
}

#[derive(Debug, Default, Clone, New)]
pub struct ConnectionComponent {
    pub outgoing_packets: Vec<BinaryCodec>
}

#[derive(Debug, Default, Clone)]
pub struct StatsComponent {
    pub has_spawned: bool,
    pub life_timestamps: (f64, f64),
    pub health: Interpolatable<f32>,
    pub max_health: Interpolatable<f32>,
    pub health_bar_opacity: Interpolatable<f32>,
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
            EntityType::Bullet | EntityType::Drone | EntityType::Trap => entity.parse_projectile_census(codec),
            EntityType::Orb => entity.parse_orb_census(codec)
        }

        if is_self {
            if entity.stats.health.target > 0.0 {
                entity.display.score.direction = 1.0;
                world.renderer.change_phase(GamePhase::Game);
            } else if (entity.stats.health.target < 0.0 || fuzzy_compare!(entity.stats.health.target, 0.0, 1e-1))
                && entity.stats.has_spawned
                && !matches!(world.renderer.phase, GamePhase::Home(_))
            {
                if entity.display.score.direction == 1.0 {
                    entity.display.score.direction = -1.0;
                    entity.display.score.value = 0.0;
                    entity.stats.life_timestamps.1 = window().performance().unwrap().now();
                }

                world.renderer.change_phase(GamePhase::Death);
            }
        }

        entity.id
    }

    pub fn render(world: &mut World, id: u32, dt: f32) {
        let self_id = world.game.self_entity.id;
        if id == self_id {
            world.game.self_entity.render_tank(&mut world.renderer.canvas2d, true, dt);
        } else {
            let (shooter, turret_idx) = {
                let mut shooter = None;
                let mut turret_idx = None;

                let entity = world.game.surroundings.get_mut(&id).unwrap();
                let is_friendly = id == self_id || entity.display.owners.has_owner(self_id);
                match entity.display.entity_type {
                    EntityType::Player => entity.render_tank(&mut world.renderer.canvas2d, is_friendly, dt),
                    EntityType::Bullet | EntityType::Drone | EntityType::Trap => {
                        if entity.time.server_ticks == 0 || entity.time.server_ticks == 1 {
                            turret_idx = Some(entity.display.turret_index);
    
                            if is_friendly {
                                world.game.self_entity.display.turret_lengths[turret_idx.unwrap()].target = 0.75;
                            } else {
                                shooter = Some(entity.display.owners.deep);
                            }
                        }
        
                        entity.render_projectile(&mut world.renderer.canvas2d, is_friendly, dt);
                    },
                    EntityType::Orb => entity.render_orb(&mut world.renderer.canvas2d, dt)
                }

                (shooter, turret_idx)
            };

            if let Some(shooter) = shooter && let Some(entity) = world.game.surroundings.get_mut(&shooter) {
                entity.display.turret_lengths[turret_idx.unwrap()].target = 0.75;
            }
        }
    }

    pub fn compute_body_fill(&self, is_friendly: bool) -> (Color, Color) {
        let mut fill = Color::blend_colors(
            if is_friendly { PLAYER_FILL } else { ENEMY_FILL }, 
            Color::RED, 
            self.display.damage_blend.value
        );

        let mut stroke = Color::blend_colors(
            if is_friendly { PLAYER_STROKE } else { ENEMY_STROKE }, 
            Color::RED, 
            self.display.damage_blend.value
        );

        if self.display.invincible && self.time.ticks % 20 > 10 {
            fill.blend_with(0.3, Color::WHITE);
            stroke.blend_with(0.3, Color::WHITE);
        }

        (fill, stroke)
    }

    pub fn render_health_bar(world: &mut World, id: u32, dt: f32) {
        let mut entity = if id == world.game.self_entity.id {
            &mut world.game.self_entity
        } else {
            world.game.surroundings.get_mut(&id).unwrap()
        };

        if entity.display.entity_type.is_projectile() { return; }
        if entity.stats.health_state != HealthState::Alive { return; }

        let ratio = entity.stats.health.value / entity.stats.max_health.value;
        entity.stats.health_bar_opacity.target = if ratio > 0.99 { 0.0 } else { 1.0 };
        entity.stats.health_bar_opacity.value = lerp!(entity.stats.health_bar_opacity.value, entity.stats.health_bar_opacity.target, 0.2 * dt);

        let color = if ratio > 0.6 {
            Color::blend_colors(MEDIUM_HEALTH_BAR, HIGH_HEALTH_BAR, (ratio - 0.6) / 0.4)
        } else if ratio > 0.2 {
            Color::blend_colors(LOW_HEALTH_BAR, MEDIUM_HEALTH_BAR, (ratio - 0.2) / 0.4)
        } else {
            Color::blend_colors(Color::RED, LOW_HEALTH_BAR, ratio / 0.2)
        };

        let (width, height) = (entity.display.radius.value + 60.0, 14.0);
        let context = &mut world.renderer.canvas2d;

        context.save();
        context.global_alpha(entity.stats.health_bar_opacity.value);
        context.translate(
            entity.physics.position.value.x + entity.physics.velocity.value.x, 
            entity.physics.position.value.y + entity.physics.velocity.value.y + entity.display.radius.value + 20.0
        );

        let true_width = (width - height).max(1.0);
        let offset = -true_width / 2.0;

        context.set_line_cap("round");
        context.set_stroke_size(height);
        context.stroke_style(BAR_BACKGROUND);

        context.begin_path();
        context.move_to(offset + 0.5, 0.5);
        context.line_to(offset + 0.5 + true_width, 0.5);
        context.stroke();

        context.set_stroke_size(height * 0.75);
        context.stroke_style(color);
        context.begin_path();
        context.move_to(offset + 0.5, 0.5);
        context.line_to(offset + 0.5 + true_width * ratio.clamp(0.0, 1.0), 0.5);
        context.stroke();

        context.restore();
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

        if !is_self {
            self.physics.angle.value = lerp_angle!(
                self.physics.angle.value, 
                self.physics.angle.target, 
                factor
            );
        }

        self.physics.position.value.lerp_towards(
            self.physics.position.target, 
            factor
        );

        self.physics.velocity.value.lerp_towards(
            self.physics.velocity.target, 
            factor
        );

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
            factor / 2.0
        );

        self.display.radius.value = lerp!(
            self.display.radius.value, 
            self.display.radius.target, 
            factor
        );

        self.display.damage_blend.value = lerp!(
            self.display.damage_blend.value,
            self.display.damage_blend.target,
            factor
        );

        if fuzzy_compare!(self.display.damage_blend.value, self.display.damage_blend.target, 1e-1) {
            self.display.damage_blend.target = 0.0;
        }

        for length in self.display.turret_lengths.iter_mut() {
            length.value = lerp!(length.value, length.target, factor);
            if fuzzy_compare!(length.value, length.target, 1e-1) {
                length.target = 1.0;
            }
        }
    }
}