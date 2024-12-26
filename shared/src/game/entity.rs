use std::time::Instant;
use derive_new::new as New;
use crate::{connection::packets::Inputs, utils::{codec::BinaryCodec, interpolatable::Interpolatable, vec2::Vector2D}};

pub const BASE_TANK_RADIUS: f32 = 50.0;
pub const MAX_STAT_INVESTMENT: usize = 7;

#[derive(Debug, Default, Clone, New)]
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

#[derive(Debug, Default, Clone, Copy)]
pub enum EntityType {
    #[default]
    Player,
    Planet,
    Star,
    Comet
}

impl TryInto<EntityType> for u8 {
    type Error = bool;

    fn try_into(self) -> Result<EntityType, Self::Error> {
        match self {
            0 => Ok(EntityType::Player),
            1 => Ok(EntityType::Planet),
            2 => Ok(EntityType::Comet),
            3 => Ok(EntityType::Star),
            _ => Err(true)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, strum_macros::EnumCount)]
pub enum UpgradeStats {
    HealthRegen,
    MaxHealth,
    BodyDamage,
    ProjectileSpeed,
    Projectile
}