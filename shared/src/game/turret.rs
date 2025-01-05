use std::fmt::Display;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum ProjectileType {
    #[default]
    Bullet
}

impl TryInto<ProjectileType> for usize {
    type Error = ();

    fn try_into(self) -> Result<ProjectileType, Self::Error> {
        match self {
            0 => Ok(ProjectileType::Bullet),
            _ => Err(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TurretRenderingHints {
    Trapezoidal(f32) // Trapezoidal(angle)
}

#[derive(Default, Debug, Clone)]
pub struct ProjectileIdentity {
    pub projectile_type: ProjectileType,
    /// Multiplier for the size of the projectile relative to the turret width.
    pub size_factor: f32,
    pub health: f32,
    pub damage: f32,
    pub penetration: f32,
    pub speed: f32,
    /// How scattered the projectiles will be when shot.
    pub scatter_rate: f32,
    /// The lifetime of the projectile.
    pub lifetime: u64
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

    /// Hints as to how to render the turret.
    pub rendering_hints: Vec<TurretRenderingHints>,

    /// The maximum number of projectiles this turret can spawn.
    pub max_projectiles: isize,
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum TurretIdentityIds {
    #[default]
    Base       = 0,
    Mono       = 1    
}

impl TryInto<TurretIdentityIds> for usize {
    type Error = ();

    fn try_into(self) -> Result<TurretIdentityIds, Self::Error> {
        match self {
            0 => Ok(TurretIdentityIds::Base),
            1 => Ok(TurretIdentityIds::Mono),
            _ => Err(())
        }
    }
}

impl TryInto<TurretStructure> for TurretIdentityIds {
    type Error = ();

    fn try_into(self) -> Result<TurretStructure, Self::Error> {
        match self {
            TurretIdentityIds::Base => Ok(get_turret_base_identity()),
            TurretIdentityIds::Mono => Ok(get_turret_mono_identity())
        }
    }
}

impl Display for TurretIdentityIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base => write!(f, "Base"),
            Self::Mono => write!(f, "Mono"),
        }
    }
}

pub fn get_turret_base_identity() -> TurretStructure {
    TurretStructure {
        id: TurretIdentityIds::Base,
        turrets: vec![],
        level_requirement: 0,
        upgrades: vec![TurretIdentityIds::Mono]
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
                rendering_hints: vec![],
                max_projectiles: -1,
                projectile_identity: ProjectileIdentity {
                    projectile_type: ProjectileType::Bullet,
                    size_factor: 1.0,
                    health: 1.0,
                    damage: 1.0,
                    penetration: 1.0,
                    speed: 1.0,
                    scatter_rate: 1.0,
                    lifetime: 1
                },
            }
        ],
        level_requirement: 0,
        upgrades: vec![]
    }
}