use rand::Rng;

use crate::{fuzzy_compare, rand};

#[derive(Default, Debug, Copy, Clone, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub const BLACK: Color = Color(0, 0, 0);
    pub const WHITE: Color = Color(255, 255, 255);
    pub const SOFT_BLACK: Color = Color(14, 14, 14);
    pub const SOFT_RED: Color = Color(200, 100, 100);    
    pub const SOFT_GREEN: Color = Color(100, 200, 100);  
    pub const SOFT_BLUE: Color = Color(100, 100, 200);    
    pub const SOFT_YELLOW: Color = Color(200, 200, 100);  
    pub const SOFT_CYAN: Color = Color(100, 200, 200);   
    pub const SOFT_MAGENTA: Color = Color(200, 100, 200); 
    pub const SOFT_GRAY: Color = Color(160, 160, 160);    
    pub const SOFT_ORANGE: Color = Color(255, 140, 70);   
    pub const SOFT_PURPLE: Color = Color(160, 70, 160);
    pub const RED: Color = Color(255, 0, 0);
    pub const GREEN: Color = Color(0, 255, 0);
    pub const BLUE: Color = Color(0, 0, 255);
    pub const YELLOW: Color = Color(255, 255, 0);
    pub const CYAN: Color = Color(0, 255, 255);
    pub const MAGENTA: Color = Color(255, 0, 255);
    pub const GRAY: Color = Color(128, 128, 128);
    pub const ORANGE: Color = Color(255, 165, 0);
    pub const PURPLE: Color = Color(128, 0, 128);
    pub const MATERIAL_SILVER: Color = Color::from_numeric(0xD1D1D1);
    pub const MATERIAL_CYAN: Color = Color::from_numeric(0x07adfa);
    pub const MATERIAL_YELLOW: Color = Color::from_numeric(0xEEC643);
    pub const MATERIAL_PURPLE: Color = Color::from_numeric(0xA855F7);
    pub const MATERIAL_GREEN: Color = Color::from_numeric(0x03fc2d);
    pub const MATERIAL_RED: Color = Color::from_numeric(0xF56C6C);
    pub const MATERIAL_GRAY: Color = Color::from_numeric(0x8E8E93);
    pub const MATERIAL_ORANGE: Color = Color::from_numeric(0xFF9F0A);    

    pub fn random() -> Color {
        let colors = [
            Color::MATERIAL_CYAN,
            Color::MATERIAL_YELLOW,
            Color::MATERIAL_PURPLE,
            Color::MATERIAL_GREEN,
            Color::MATERIAL_RED,
            Color::MATERIAL_ORANGE,
        ];
        
        colors[rand!(0, colors.len() - 1)]
    }

    pub const fn from_numeric(hex: u32) -> Color {
        Color::from_rgb(((hex >> 16) & 0xFF) as u8, ((hex >> 8) & 0xFF) as u8, (hex & 0xFF) as u8)
    }

    pub const fn from_rgb(r: u8, g: u8, b: u8) -> Color {
        Color(r, g, b)
    }

    pub fn from_hex(hex: &str) -> Color {
        if hex.starts_with('#') {
            panic!("use of # inside Color::from_hex; please remove");
        }

        let parsed = u32::from_str_radix(hex, 16).expect("couldnt parse color");
        Color(
            ((parsed >> 16) & 255) as u8,
            ((parsed >> 8) & 255) as u8,
            (parsed & 255) as u8,
        )
    }

    pub const fn blend_colors(primary: Color, secondary: Color, factor: f32) -> Color {
        let mut c = primary;
        c.blend_with(factor, secondary);
        
        c
    }

    pub fn int(&self) -> u32 {
        (self.0 as u32) << 16 | (self.1 as u32) << 8 | (self.2 as u32)
    }

    pub fn css(&self) -> String {
        format!("rgb({}, {}, {})", self.0, self.1, self.2)
    }

    pub const fn blend_with(&mut self, factor: f32, color: Color) -> &mut Color {
        self.0 = (color.0 as f32 * factor + self.0 as f32 * (1.0 - factor)) as u8;
        self.1 = (color.1 as f32 * factor + self.1 as f32 * (1.0 - factor)) as u8;
        self.2 = (color.2 as f32 * factor + self.2 as f32 * (1.0 - factor)) as u8;

        self
    }

    pub fn grayscale(&mut self) -> &mut Color {
        let avg = (self.0 as u16 + self.1 as u16 + self.2 as u16) / 3;
        self.0 = avg as u8;
        self.1 = avg as u8;
        self.2 = avg as u8;

        self
    }

    pub fn invert(&mut self) -> &mut Color {
        self.0 = 255 - self.0;
        self.1 = 255 - self.1;
        self.2 = 255 - self.2;

        self
    }

    pub fn partial_eq(&self, other: Color, tolerance: f32) -> bool {
        fuzzy_compare!(self.0 as f32, other.0 as f32, tolerance) && 
        fuzzy_compare!(self.1 as f32, other.1 as f32, tolerance) &&
        fuzzy_compare!(self.2 as f32, other.2 as f32, tolerance)   
    }
}