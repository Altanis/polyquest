use shared::rand;
use rand::Rng;

#[derive(Default, Debug, Copy, Clone)]
pub struct Color(pub u8, pub u8, pub u8);

impl Color {
    pub const BLACK: Color = Color(0, 0, 0);
    pub const WHITE: Color = Color(255, 255, 255);
    pub const RED: Color = Color(255, 0, 0);
    pub const GREEN: Color = Color(0, 255, 0);
    pub const BLUE: Color = Color(0, 0, 255);
    pub const YELLOW: Color = Color(255, 255, 0);
    pub const CYAN: Color = Color(0, 255, 255);
    pub const MAGENTA: Color = Color(255, 0, 255);
    pub const GRAY: Color = Color(128, 128, 128);
    pub const ORANGE: Color = Color(255, 165, 0);
    pub const PURPLE: Color = Color(128, 0, 128);

    pub fn random() -> Color {
        let colors = [
            Color::RED, 
            Color::from_hex("ffa500"),
            Color::from_hex("0000ff"),
            Color::from_hex("008000")
        ];

        colors[rand!(0, colors.len() - 1)]
    }

    pub fn from_rgb(r: u8, g: u8, b: u8) -> Color {
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

    pub fn blend_colors(primary: Color, secondary: Color, factor: f32) -> Color {
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

    pub fn blend_with(&mut self, factor: f32, color: Color) -> &mut Color {
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
}