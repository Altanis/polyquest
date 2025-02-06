use strum::EnumCount;

use crate::utils::color::Color;

use super::entity::UpgradeStats;

pub const OUTBOUNDS_FILL: Color = Color::from_numeric(0x252833); // Dark blue-gray for outside arena  
pub const INBOUNDS_FILL: Color = Color::from_numeric(0x323644); // Slightly lighter blue-gray for inside arena  

pub const PLAYER_FILL: Color = Color::from_numeric(0x00B1DE);
pub const ENEMY_FILL: Color = Color::from_numeric(0xF14E54);
pub const TURRET_FILL: Color = Color::from_numeric(0x878787);

// fills of shapes scattered across the arena
pub const ORB_FLICKERING_FILL: Color = Color::from_numeric(0xA0A0A0); // Brighter gray for flickering orb  
pub const ORB_BASIC_FILL: Color = Color::MATERIAL_CYAN; // Brighter teal for basic orb  
pub const ORB_STABLE_FILL: Color = Color::MATERIAL_YELLOW; // Vibrant cyan for stability  
pub const ORB_HEAVY_FILL: Color = Color::MATERIAL_PURPLE; // Rich purple for heavy orb  
pub const ORB_RADIANT_FILL: Color = Color::MATERIAL_RED; // Warm, glowing orange for radiant orb  
pub const ORB_CELESTIAL_FILL: Color = Color::MATERIAL_GREEN; // Vibrant yellow-green for celestial orb  

pub const SCORE_BAR_FOREGROUND: Color = Color::from_numeric(0x1FCC6F); // Deep emerald green  
pub const LEVEL_BAR_FOREGROUND: Color = Color::from_numeric(0xE0B13D); // Warm gold  

pub const SMASHER_GUARD_FILL: Color = Color::from_numeric(0x878787);

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

pub const LOW_HEALTH_BAR: Color = Color::from_numeric(0xD14040); // Deep crimson  
pub const MEDIUM_HEALTH_BAR: Color = Color::from_numeric(0xE6D13A); // Warm golden yellow  
pub const HIGH_HEALTH_BAR: Color = Color::from_numeric(0x17C935); // Vivid green


// pub const OUTBOUNDS_FILL: Color = Color::from_numeric(0xAAB3C2);
// pub const INBOUNDS_FILL: Color = Color::from_numeric(0xC6CCD6);
pub const GRID_ALPHA: f32 = 0.12;
pub const GRID_COLOR: Color = Color::from_numeric(0x7A85A5);
pub const GRID_SIZE: f32 = 50.0;
pub const STROKE_SIZE: f32 = 7.5;

pub const STROKE_INTENSITY: f32 = 0.25;

// pub const PLAYER_FILL: Color = Color::from_numeric(0x66D2E8);
pub const PLAYER_STROKE: Color = Color::blend_colors(PLAYER_FILL, Color::BLACK, STROKE_INTENSITY);
// pub const ENEMY_FILL: Color = Color::from_numeric(0xE88B8B);
pub const ENEMY_STROKE: Color = Color::blend_colors(ENEMY_FILL, Color::BLACK, STROKE_INTENSITY);

// pub const TURRET_FILL: Color = Color::from_numeric(0xB8B8B8);
pub const TURRET_STROKE: Color = Color::blend_colors(TURRET_FILL, Color::BLACK, STROKE_INTENSITY);

pub const BAR_BACKGROUND: Color = Color::SOFT_BLACK;

pub const SMASHER_GUARD_STROKE: Color = Color::blend_colors(SMASHER_GUARD_FILL, Color::BLACK, STROKE_INTENSITY);

pub const MINIMAP_STROKE: Color = Color::from_numeric(0x555555);
pub const MINIMAP_FILL: Color = Color::from_numeric(0xCDCDCD);
pub const MINIMAP_PLAYER_FILL: Color = Color::from_numeric(0x000000);
pub const MINIMAP_SIZE: f32 = 175.0;
pub const MINIMAP_PADDING: f32 = 15.0;

pub const LEADER_ARROW_COLOR: Color = Color::BLACK;

// pub const ORB_FLICKERING_FILL: Color = Color::from_numeric(0xE3E3E3);
// pub const ORB_BASIC_FILL: Color = Color::from_numeric(0xA7D7E8);
// pub const ORB_STABLE_FILL: Color = Color::from_numeric(0x66D2E8);
// pub const ORB_HEAVY_FILL: Color = Color::from_numeric(0xCBA4E8);
// pub const ORB_RADIANT_FILL: Color = Color::from_numeric(0xE8A76C);
// pub const ORB_CELESTIAL_FILL: Color = Color::from_numeric(0xD4E86C);

pub const ORB_FLICKERING_STROKE: Color = Color::blend_colors(ORB_FLICKERING_FILL, Color::BLACK, STROKE_INTENSITY);
pub const ORB_BASIC_STROKE: Color = Color::blend_colors(ORB_BASIC_FILL, Color::BLACK, STROKE_INTENSITY);
pub const ORB_STABLE_STROKE: Color = Color::blend_colors(ORB_STABLE_FILL, Color::BLACK, STROKE_INTENSITY);
pub const ORB_HEAVY_STROKE: Color = Color::blend_colors(ORB_HEAVY_FILL, Color::BLACK, STROKE_INTENSITY);
pub const ORB_RADIANT_STROKE: Color = Color::blend_colors(ORB_RADIANT_FILL, Color::BLACK, STROKE_INTENSITY);
pub const ORB_CELESTIAL_STROKE: Color = Color::blend_colors(ORB_CELESTIAL_FILL, Color::BLACK, STROKE_INTENSITY);