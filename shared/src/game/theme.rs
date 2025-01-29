use strum::EnumCount;

use crate::utils::color::Color;

use super::entity::UpgradeStats;

pub const OUTBOUNDS_FILL: Color = Color::from_numeric(0xCDCDCD);
pub const INBOUNDS_FILL: Color = Color::from_numeric(0xD9D9D9);
pub const GRID_ALPHA: f32 = 0.1;
pub const GRID_COLOR: Color = Color::from_numeric(0x000000);
pub const GRID_SIZE: f32 = 50.0;
pub const STROKE_SIZE: f32 = 7.5;

pub const STROKE_INTENSITY: f32 = 0.25;

pub const PLAYER_FILL: Color = Color::from_numeric(0x00B1DE);
pub const PLAYER_STROKE: Color = Color::blend_colors(PLAYER_FILL, Color::BLACK, STROKE_INTENSITY);
pub const ENEMY_FILL: Color = Color::from_numeric(0xF14E54);
pub const ENEMY_STROKE: Color = Color::blend_colors(ENEMY_FILL, Color::BLACK, STROKE_INTENSITY);

pub const TURRET_FILL: Color = Color::from_numeric(0x999999);
pub const TURRET_STROKE: Color = Color::blend_colors(TURRET_FILL, Color::BLACK, STROKE_INTENSITY);

pub const BAR_BACKGROUND: Color = Color::SOFT_BLACK;
pub const SCORE_BAR_FOREGROUND: Color = Color::from_numeric(0x58FA96);
pub const LEVEL_BAR_FOREGROUND: Color = Color::from_numeric(0xF5DA64);

pub const SMASHER_GUARD_FILL: Color = Color::from_numeric(0x4F4F4F);
pub const SMASHER_GUARD_STROKE: Color = Color::blend_colors(SMASHER_GUARD_FILL, Color::BLACK, STROKE_INTENSITY);

pub const UPGRADE_STAT_COLORS: [Color; UpgradeStats::COUNT] = [
    Color::from_numeric(0xE69F6C), 
    Color::from_numeric(0xFF73FF),
    Color::from_numeric(0xC980FF), 
    Color::from_numeric(0x71B4FF), 
    Color::from_numeric(0xFFED3F), 
    Color::from_numeric(0xFF7979),
    Color::from_numeric(0x88FF41), 
    Color::from_numeric(0x41FFFF)
];

pub const LOW_HEALTH_BAR: Color = Color::RED;
pub const MEDIUM_HEALTH_BAR: Color = Color::from_numeric(0xFBFF00);
pub const HIGH_HEALTH_BAR: Color = Color::from_numeric(0x15FF00);

pub const MINIMAP_STROKE: Color = Color::from_numeric(0x555555);
pub const MINIMAP_FILL: Color = Color::from_numeric(0xCDCDCD);
pub const MINIMAP_PLAYER_FILL: Color = Color::from_numeric(0x000000);
pub const MINIMAP_SIZE: f32 = 175.0;
pub const MINIMAP_PADDING: f32 = 15.0;

pub const LEADER_ARROW_COLOR: Color = Color::BLACK;