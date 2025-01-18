use std::fmt::Display;

use super::entity::EntityType;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TurretRenderingHints {
    Trapezoidal(f32), // Trapezoidal(angle)
    Trapper
}

#[derive(Default, Debug, Clone)]
pub struct ProjectileIdentity {
    pub projectile_type: EntityType,
    /// Multiplier for the size of the projectile relative to the turret width.
    pub size_factor: f32,
    pub health: f32,
    pub damage: f32,
    pub speed: f32,
    /// How scattered the projectiles will be when shot.
    pub scatter_rate: f32,
    /// The lifetime of the projectile.
    pub lifetime: i64,
    /// The absorption factor of the tank.
    pub absorption_factor: f32
}

#[derive(Debug, Clone)]
pub struct TurretIdentity {
    /// The angle of the turret from the horizontal.
    pub angle: f32,
    /// The x offset of the turret from the horizontal.
    pub x_offset: f32,
    /// The y offset of the turret from the vertical (to influence bullet offsets).
    pub y_offset: f32,
    /// The size of the y direction of the turret.
    pub width: f32,
    /// The size of the x direction of the turret.
    pub length: f32,
    /// The delay of the turret, relative to turrets with 0 delay.
    pub delay: f32,
    pub reload: f32,
    /// The cached reload time of the turret.
    pub reload_time: f32,
    /// The position the turret is in the shooting cycle.
    pub cycle_position: f32,
    pub recoil: f32,
    pub force_shoot: bool,

    /// Hints as to how to render the turret.
    pub rendering_hints: Vec<TurretRenderingHints>,

    /// The maximum number of projectiles this turret can spawn.
    pub max_projectiles: isize,
    /// The current number of projectiles the turret has spawned.
    pub projectiles_spawned: isize,
    /// The identity of the projectiles the turret shoots.
    pub projectile_identity: ProjectileIdentity
}

impl TurretIdentity {
    pub fn can_fire(&mut self, reload: f32, shooting: bool) -> bool {
        if self.reload_time == 0.0 && self.cycle_position == 0.0 {
            self.reload_time = reload * self.reload;
            self.cycle_position = self.reload_time;
        }

        let reload_time = reload * self.reload;
        if self.reload_time != reload_time {
            self.cycle_position *= reload_time / self.reload_time;
            self.reload_time = reload_time;
        }

        self.cycle_position += 1.0;

        if (self.cycle_position >= reload_time) && !shooting {
            self.cycle_position = reload_time;
            return false;
        }

        if self.cycle_position >= reload_time * (1.0 + self.delay) {
            self.reload_time = reload_time;
            self.cycle_position = reload_time * self.delay;

            shooting
        } else {
            false
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct TurretStructure {
    pub id: TurretIdentityIds,
    pub turrets: Vec<TurretIdentity>,
    pub level_requirement: usize,
    pub upgrades: Vec<TurretIdentityIds>
}

#[derive(Debug, Default, Clone, Copy, PartialEq, num_enum::TryFromPrimitive)]
#[repr(usize)]
pub enum TurretIdentityIds {
    #[default]
    Base       = 0,
    Mono       = 1,
    Spawner    = 2,
    Warden     = 3,
    Couplet    = 4,
    Flurry     = 5,
    Flank      = 6,
    Sniper     = 7,
    Pounder    = 8 // -> Eradicator -> Dreadnought
}

impl TryInto<TurretStructure> for TurretIdentityIds {
    type Error = ();

    fn try_into(self) -> Result<TurretStructure, Self::Error> {
        match self {
            TurretIdentityIds::Base => Ok(get_turret_base_identity()),
            TurretIdentityIds::Mono => Ok(get_turret_mono_identity()),
            TurretIdentityIds::Spawner => Ok(get_turret_spawner_identity()),
            TurretIdentityIds::Warden => Ok(get_turret_warden_identity()),
            TurretIdentityIds::Couplet => Ok(get_turret_couplet_identity()),
            TurretIdentityIds::Flurry => Ok(get_turret_flurry_identity()),
            TurretIdentityIds::Flank => Ok(get_turret_flank_identity()),
            TurretIdentityIds::Sniper => Ok(get_turret_sniper_identity()),
            TurretIdentityIds::Pounder => Ok(get_turret_pounder_identity())
        }
    }
}

impl Display for TurretIdentityIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant_name = format!("{:?}", self);
        let formatted_name: String = variant_name
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i > 0 && c.is_uppercase() {
                    vec![' ', c]
                } else {
                    vec![c]
                }
            })
            .collect();

        write!(f, "{}", formatted_name)
    }
}

pub fn get_turret_base_identity() -> TurretStructure {
    TurretStructure {
        id: TurretIdentityIds::Base,
        turrets: vec![],
        level_requirement: 0,
        upgrades: vec![TurretIdentityIds::Mono, TurretIdentityIds::Spawner, TurretIdentityIds::Warden]
    }
}

pub fn get_turret_mono_identity() -> TurretStructure {
    TurretStructure {
        id: TurretIdentityIds::Mono,
        turrets: vec![
            TurretIdentity {
                angle: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
                length: 57.0,
                width: 24.0,
                delay: 0.0,
                reload: 1.0,
                reload_time: 0.0,
                cycle_position: 0.0,
                recoil: 1.0,
                force_shoot: false,
                rendering_hints: vec![],
                max_projectiles: -1,
                projectiles_spawned: -1,
                projectile_identity: ProjectileIdentity {
                    projectile_type: EntityType::Bullet,
                    size_factor: 1.0,
                    health: 1.0,
                    damage: 1.0,
                    speed: 1.0,
                    scatter_rate: 1.0,
                    lifetime: 1,
                    absorption_factor: 1.0
                },
            }
        ],
        level_requirement: 0,
        upgrades: vec![TurretIdentityIds::Couplet, TurretIdentityIds::Flurry, TurretIdentityIds::Flank, TurretIdentityIds::Sniper, TurretIdentityIds::Pounder]
    }
}

pub fn get_turret_spawner_identity() -> TurretStructure {
    TurretStructure {
        id: TurretIdentityIds::Spawner,
        turrets: vec![
            TurretIdentity {
                angle: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
                length: 42.0,
                width: 24.0,
                delay: 0.0,
                reload: 6.0,
                reload_time: 0.0,
                cycle_position: 0.0,
                recoil: 1.0,
                force_shoot: true,
                rendering_hints: vec![TurretRenderingHints::Trapezoidal(0.0)],
                max_projectiles: 4,
                projectiles_spawned: 0,
                projectile_identity: ProjectileIdentity {
                    projectile_type: EntityType::Drone,
                    size_factor: 1.0,
                    health: 2.0,
                    damage: 0.7,
                    speed: 0.8,
                    scatter_rate: 1.0,
                    lifetime: -1,
                    absorption_factor: 1.0
                },
            }
        ],
        level_requirement: 0,
        upgrades: vec![]
    }
}

pub fn get_turret_warden_identity() -> TurretStructure {
    TurretStructure {
        id: TurretIdentityIds::Warden,
        turrets: vec![
            TurretIdentity {
                angle: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
                length: 36.0,
                width: 24.0,
                delay: 0.0,
                reload: 1.5,
                reload_time: 0.0,
                cycle_position: 0.0,
                recoil: 1.0,
                force_shoot: false,
                rendering_hints: vec![TurretRenderingHints::Trapper],
                max_projectiles: -1,
                projectiles_spawned: -1,
                projectile_identity: ProjectileIdentity {
                    projectile_type: EntityType::Trap,
                    size_factor: 0.8,
                    health: 2.0,
                    damage: 1.0,
                    speed: 2.0,
                    scatter_rate: 1.0,
                    lifetime: 8,
                    absorption_factor: 1.0
                },
            }
        ],
        level_requirement: 0,
        upgrades: vec![]
    }
}

pub fn get_turret_couplet_identity() -> TurretStructure {
    TurretStructure {
        id: TurretIdentityIds::Couplet,
        turrets: vec![
            TurretIdentity {
                angle: 0.0,
                x_offset: 0.0,
                y_offset: -16.0,
                length: 57.0,
                width: 24.0,
                delay: 0.0,
                reload: 1.0,
                reload_time: 0.0,
                cycle_position: 0.0,
                recoil: 0.75,
                force_shoot: false,
                rendering_hints: vec![],
                max_projectiles: -1,
                projectiles_spawned: -1,
                projectile_identity: ProjectileIdentity {
                    projectile_type: EntityType::Bullet,
                    size_factor: 1.0,
                    health: 0.9,
                    damage: 0.65,
                    speed: 1.0,
                    scatter_rate: 1.0,
                    lifetime: 1,
                    absorption_factor: 1.0
                },
            },
            TurretIdentity {
                angle: 0.0,
                x_offset: 0.0,
                y_offset: 16.0,
                length: 57.0,
                width: 24.0,
                delay: 0.5,
                reload: 1.0,
                reload_time: 0.0,
                cycle_position: 0.0,
                recoil: 0.75,
                force_shoot: false,
                rendering_hints: vec![],
                max_projectiles: -1,
                projectiles_spawned: -1,
                projectile_identity: ProjectileIdentity {
                    projectile_type: EntityType::Bullet,
                    size_factor: 1.0,
                    health: 0.9,
                    damage: 0.65,
                    speed: 1.0,
                    scatter_rate: 1.0,
                    lifetime: 1,
                    absorption_factor: 1.0
                },
            }
        ],
        level_requirement: 15,
        upgrades: vec![]
    }
}

pub fn get_turret_flurry_identity() -> TurretStructure {
    TurretStructure {
        id: TurretIdentityIds::Flurry,
        turrets: vec![
            TurretIdentity {
                angle: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
                length: 57.0,
                width: 22.0,
                delay: 0.0,
                reload: 0.5,
                reload_time: 0.0,
                cycle_position: 0.0,
                recoil: 1.0,
                force_shoot: false,
                rendering_hints: vec![TurretRenderingHints::Trapezoidal(0.0)],
                max_projectiles: -1,
                projectiles_spawned: -1,
                projectile_identity: ProjectileIdentity {
                    projectile_type: EntityType::Bullet,
                    size_factor: 1.0,
                    health: 1.0,
                    damage: 0.7,
                    speed: 1.0,
                    scatter_rate: 3.0,
                    lifetime: 1,
                    absorption_factor: 1.0
                },
            }
        ],
        level_requirement: 15,
        upgrades: vec![]
    }
}

pub fn get_turret_flank_identity() -> TurretStructure {
    TurretStructure {
        id: TurretIdentityIds::Flank,
        turrets: vec![
            TurretIdentity {
                angle: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
                length: 57.0,
                width: 24.0,
                delay: 0.0,
                reload: 1.0,
                reload_time: 0.0,
                cycle_position: 0.0,
                recoil: 1.0,
                force_shoot: false,
                rendering_hints: vec![],
                max_projectiles: -1,
                projectiles_spawned: -1,
                projectile_identity: ProjectileIdentity {
                    projectile_type: EntityType::Bullet,
                    size_factor: 1.0,
                    health: 1.0,
                    damage: 1.0,
                    speed: 1.0,
                    scatter_rate: 1.0,
                    lifetime: 1,
                    absorption_factor: 1.0
                },
            },
            TurretIdentity {
                angle: std::f32::consts::PI,
                x_offset: 0.0,
                y_offset: 0.0,
                length: 48.0,
                width: 24.0,
                delay: 0.0,
                reload: 1.0,
                reload_time: 0.0,
                cycle_position: 0.0,
                recoil: 1.0,
                force_shoot: false,
                rendering_hints: vec![],
                max_projectiles: -1,
                projectiles_spawned: -1,
                projectile_identity: ProjectileIdentity {
                    projectile_type: EntityType::Bullet,
                    size_factor: 1.0,
                    health: 1.0,
                    damage: 1.0,
                    speed: 1.0,
                    scatter_rate: 1.0,
                    lifetime: 1,
                    absorption_factor: 1.0
                },
            }
        ],
        level_requirement: 15,
        upgrades: vec![]
    }
}

pub fn get_turret_sniper_identity() -> TurretStructure {
    TurretStructure {
        id: TurretIdentityIds::Sniper,
        turrets: vec![
            TurretIdentity {
                angle: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
                length: 66.0,
                width: 24.0,
                delay: 0.0,
                reload: 1.5,
                reload_time: 0.0,
                cycle_position: 0.0,
                recoil: 3.0,
                force_shoot: false,
                rendering_hints: vec![],
                max_projectiles: -1,
                projectiles_spawned: -1,
                projectile_identity: ProjectileIdentity {
                    projectile_type: EntityType::Bullet,
                    size_factor: 1.0,
                    health: 1.0,
                    damage: 1.0,
                    speed: 1.5,
                    scatter_rate: 1.0,
                    lifetime: 1,
                    absorption_factor: 1.0
                },
            }
        ],
        level_requirement: 15,
        upgrades: vec![]
    }
}

pub fn get_turret_pounder_identity() -> TurretStructure {
    TurretStructure {
        id: TurretIdentityIds::Pounder,
        turrets: vec![
            TurretIdentity {
                angle: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
                length: 57.0,
                width: 30.0,
                delay: 0.0,
                reload: 2.0,
                reload_time: 0.0,
                cycle_position: 0.0,
                recoil: 7.5,
                force_shoot: false,
                rendering_hints: vec![],
                max_projectiles: -1,
                projectiles_spawned: -1,
                projectile_identity: ProjectileIdentity {
                    projectile_type: EntityType::Bullet,
                    size_factor: 1.0,
                    health: 1.5,
                    damage: 1.5,
                    speed: 0.85,
                    scatter_rate: 1.0,
                    lifetime: 1,
                    absorption_factor: 1.0
                },
            }
        ],
        level_requirement: 15,
        upgrades: vec![]
    }
}