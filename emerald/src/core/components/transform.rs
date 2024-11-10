use serde::{Deserialize, Serialize};

use crate::math::Vector2;

/// The core piece of an entity, determines it's transformative state and position in the world.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct Transform {
    #[serde(default)]
    pub translation: Translation,

    #[serde(default)]
    pub rotation: f32,

    #[serde(default)]
    pub scale: Scale,
}
impl Transform {
    pub fn from_translation<T: Into<Translation>>(into_translation: T) -> Self {
        let mut transform = Transform::default();
        transform.translation = into_translation.into();

        transform
    }
}
impl Default for Transform {
    fn default() -> Self {
        Transform {
            translation: Translation::default(),
            rotation: 0.0,
            scale: Scale::default(),
        }
    }
}
impl core::ops::Sub for Transform {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        let translation = self.translation - other.translation;
        let rotation = self.rotation - other.rotation;
        let scale = self.scale - other.scale;

        Self {
            translation,
            rotation,
            scale,
        }
    }
}
impl core::ops::Add for Transform {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        let translation = self.translation + other.translation;
        let rotation = self.rotation + other.rotation;
        let scale = self.scale + other.scale;

        Self {
            translation,
            rotation,
            scale,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
}
impl Scale {
    pub fn new(x: f32, y: f32) -> Self {
        Scale { x, y }
    }
}
impl Default for Scale {
    fn default() -> Self {
        Scale::new(1.0, 1.0)
    }
}
impl core::ops::Sub for Scale {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl core::ops::Add for Scale {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
pub struct Translation {
    #[serde(default)]
    pub x: f32,

    #[serde(default)]
    pub y: f32,
}
impl Translation {
    pub fn new(x: f32, y: f32) -> Self {
        Translation { x, y }
    }
}
impl Default for Translation {
    fn default() -> Self {
        Translation::new(0.0, 0.0)
    }
}
impl From<Vector2> for Translation {
    #[inline]
    fn from(v: Vector2) -> Self {
        Self::new(v.x.to_bits() as f32, v.y.to_bits() as f32)
    }
}
impl From<Translation> for Vector2 {
    #[inline]
    fn from(t: Translation) -> Self {
        Vector2::from_float(t.x, t.y)
    }
}

impl From<(f32, f32)> for Translation {
    fn from((x, y): (f32, f32)) -> Self {
        Translation::new(x, y)
    }
}

impl core::ops::Add for Translation {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl core::ops::AddAssign for Translation {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl core::ops::Sub for Translation {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl core::ops::SubAssign for Translation {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl core::ops::Mul<f32> for Translation {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl core::ops::MulAssign<f32> for Translation {
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
    }
}

impl core::ops::Div<f32> for Translation {
    type Output = Self;

    fn div(self, scalar: f32) -> Self::Output {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl core::ops::DivAssign<f32> for Translation {
    fn div_assign(&mut self, scalar: f32) {
        self.x /= scalar;
        self.y /= scalar;
    }
}
