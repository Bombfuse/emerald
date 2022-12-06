use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }

    pub fn to_percentage(&self) -> (f32, f32, f32, f32) {
        let r = self.r as f32 / 255.0;
        let g = self.g as f32 / 255.0;
        let b = self.b as f32 / 255.0;
        let a = self.a as f32 / 255.0;

        (r, g, b, a)
    }

    pub fn to_percentage_linear(&self) -> (f64, f64, f64, f64) {
        let r = to_linear(self.r as f64 / 255.0);
        let g = to_linear(self.g as f64 / 255.0);
        let b = to_linear(self.b as f64 / 255.0);
        let a = self.a as f64 / 255.0;

        (r, g, b, a)
    }

    pub fn to_percentage_slice(&self) -> [f32; 4] {
        let (r, g, b, a) = self.to_percentage();
        [r, g, b, a]
    }

    pub fn with_alpha(&self, a: u8) -> Color {
        Color::new(self.r, self.g, self.b, a)
    }
}

fn to_linear(x: f64) -> f64 {
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

pub const BLACK: Color = Color {
    r: 0,
    g: 0,
    b: 0,
    a: 255,
};
pub const WHITE: Color = Color {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
};
pub const CORNFLOWER_BLUE: Color = Color {
    r: 100,
    g: 149,
    b: 237,
    a: 255,
};
