#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectileType {
    Bullet
}

#[derive(Debug, Clone)]
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
    pub lifetime: f32
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TurretRenderingType {
    None
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
    pub recoil: f32,

    /// Whether or not the turret is trapezoidal.
    pub is_trapezoidal: bool,
    /// The angle the turret has rotated w.r.t the turret's center.
    pub trapezoidal_angle: f32,
    /// A specific rendering method for the turret.
    pub render_type: TurretRenderingType,

    /// The maximum number of projectiles this turret can spawn.
    pub max_projectiles: isize,
    /// The identity of the projectiles the turret shoots.
    pub projectile_identity: ProjectileIdentity,

    pub level_requirement: usize,
    pub upgrades: Vec<TurretStructure>
}

#[derive(Default, Debug, Clone)]
pub struct TurretStructure {
    pub id: usize,
    pub turrets: Vec<TurretIdentity>
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum TurretIdentityIds {
    #[default]
    Mono       = 0,
    // Producer   = 1
}

impl TryInto<TurretIdentityIds> for usize {
    type Error = ();

    fn try_into(self) -> Result<TurretIdentityIds, Self::Error> {
        match self {
            0 => Ok(TurretIdentityIds::Mono),
            _ => Err(())
        }
    }
}

impl TryInto<TurretStructure> for TurretIdentityIds {
    type Error = ();

    fn try_into(self) -> Result<TurretStructure, Self::Error> {
        match self {
            TurretIdentityIds::Mono => Ok(get_turret_mono_identity()),
        }
    }
}

pub fn get_turret_mono_identity() -> TurretStructure {
    TurretStructure {
        id: TurretIdentityIds::Mono as usize,
        turrets: vec![
            TurretIdentity {
                angle: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
                width: 24.0,
                length: 57.0,
                delay: 0.0,
                reload: 1.0,
                recoil: 1.0,
                is_trapezoidal: false,
                trapezoidal_angle: 0.0,
                render_type: TurretRenderingType::None,
                max_projectiles: -1,
                projectile_identity: ProjectileIdentity {
                    projectile_type: ProjectileType::Bullet,
                    size_factor: 1.0,
                    health: 1.0,
                    damage: 1.0,
                    penetration: 1.0,
                    speed: 1.0,
                    scatter_rate: 1.0,
                    lifetime: 1.0
                },
                level_requirement: 0,
                upgrades: vec![]
            }
        ]
    }
}