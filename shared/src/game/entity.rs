use std::{fmt::Debug, num::NonZeroU32};

use derive_new::new as New;
use strum::IntoEnumIterator;
use crate::{connection::packets::Inputs, utils::{color::Color, consts::MAX_LEVEL, interpolatable::Interpolatable, vec2::Vector2D}};

use super::{body::BodyIdentityIds, turret::TurretIdentityIds};

pub const BASE_TANK_RADIUS: f32 = 50.0;
pub const FICTITIOUS_TANK_RADIUS: f32 = 30.0;
pub const MAX_STAT_INVESTMENT: usize = 7;

#[derive(Default, Clone, Copy, New)]
pub struct InputFlags(u32);
impl InputFlags {
    pub fn is_set(&self, flag: Inputs) -> bool {
        self.0 & flag as u32 == flag as u32
    }

    pub fn set_flag(&mut self, flag: Inputs) {
        self.0 |= flag as u32;
    }

    pub fn clear_flag(&mut self, flag: Inputs) {
        self.0 &= !(flag as u32);
    }

    pub fn get_value(&self) -> u32 {
        self.0
    }
}

impl Debug for InputFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for flag in Inputs::iter() {
            if self.is_set(flag) {
                write!(f, "{:?}, ", flag)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum EntityType {
    #[default]
    Player,
    Bullet, // you can add stuff like drone, minion, etc.
    Drone,
    Trap
    // Planet,
    // Star,
    // Comet
}

impl EntityType {
    pub fn is_projectile(&self) -> bool {
        matches!(self, EntityType::Bullet | EntityType::Drone | EntityType::Trap)
    }

    pub fn is_drone(&self) -> bool {
        matches!(self, EntityType::Drone)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, strum_macros::EnumCount, strum_macros::EnumIter)]
pub enum UpgradeStats {
    HealthRegen,
    MaxHealth,
    BodyDamage,
    ProjectileSpeed,
    ProjectilePenetration,
    ProjectileDamage,
    Reload,
    MovementSpeed
}

impl std::fmt::Display for UpgradeStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stat_str = match self {
            UpgradeStats::HealthRegen => "Health Regen",
            UpgradeStats::MaxHealth => "Max Health",
            UpgradeStats::BodyDamage => "Body Damage",
            UpgradeStats::ProjectileSpeed => "Projectile Speed",
            UpgradeStats::ProjectilePenetration => "Projectile Penetration",
            UpgradeStats::ProjectileDamage => "Projectile Damage",
            UpgradeStats::Reload => "Reload",
            UpgradeStats::MovementSpeed => "Movement Speed",
        };
        
        write!(f, "{}", stat_str)
    }
}

pub const LEVEL_TO_SCORE_TABLE: [usize; MAX_LEVEL] = [
    0, 4, 14, 29, 50, 78, 114, 158, 211, 275, 350, 438, 539, 655, 
    788, 939, 1109, 1301, 1516, 1758, 2026, 2326, 2658, 3027, 3434, 3884, 
    4380, 4926, 5526, 6185, 6907, 7698, 8537, 9426, 10369, 11368, 12427, 
    13549, 14739, 16001, 17337, 18755, 20257, 21849, 23537
];

pub fn get_min_score_from_level(level: usize) -> usize {
    if level > LEVEL_TO_SCORE_TABLE.len() {
        return LEVEL_TO_SCORE_TABLE[MAX_LEVEL - 1];
    }

    LEVEL_TO_SCORE_TABLE[level - 1]
}

pub fn get_level_from_score(score: usize) -> usize {
    for (level, &level_score) in LEVEL_TO_SCORE_TABLE.iter().enumerate() {
        if score < level_score {
            return level;
        }
    }
    
    MAX_LEVEL
}

/// Generates an identity name given identity ids.
pub fn generate_identity(body: BodyIdentityIds, turret: TurretIdentityIds) -> String {
    let (body, turret) = (format!("{}", body), format!("{}", turret));
    if body == turret {
        turret
    } else {
        format!("{}-{}", turret, body)
    }
}

/// A struct encapsulating ownership.
/// The shallow and deep owners may be identical.
#[derive(Debug, Default, Clone, Copy)]
pub struct Ownership {
    /// The immediate cause of creation.
    pub shallow: u32,
    /// The ultimate cause of creation.
    pub deep: u32
}

impl Ownership {
    pub fn new(shallow: u32, deep: u32) -> Ownership {
        Ownership { shallow, deep }
    }

    pub fn from_single_owner(owner: u32) -> Ownership {
        Ownership::new(owner, owner)
    }

    pub fn to_tuple(&self) -> (u32, u32) {
        (self.shallow, self.deep)
    }

    pub fn has_owner(&self, owner: u32) -> bool {
        self.shallow == owner || self.deep == owner
    }
}

#[derive(Default, Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub color: Color,
    pub lifetime: u64,
    pub opacity: Interpolatable<f32>,
    pub position: Interpolatable<Vector2D>
}

#[derive(Default, Clone)]
pub struct TankUpgrades {
    pub body: Vec<BodyIdentityIds>,
    pub turret: Vec<TurretIdentityIds>
}