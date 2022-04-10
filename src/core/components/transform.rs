use glam::{vec2, Vec2};
use nalgebra::{Isometry2, Translation2};
use nanoserde::DeJson;

/// The core piece of an entity, determines it's transformative state and position in the world.
#[derive(Clone, Copy, Debug, DeJson)]
pub struct Transform {
    pub translation: Translation,
    pub rotation: f32,
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
impl std::ops::Sub for Transform {
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
impl std::ops::Add for Transform {
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

#[derive(Clone, Copy, Debug, DeJson)]
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
impl std::ops::Sub for Scale {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl std::ops::Add for Scale {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

#[derive(Clone, Copy, Debug, DeJson)]
pub struct Translation {
    pub x: f32,
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
impl From<Vec2> for Translation {
    #[inline]
    fn from(v: Vec2) -> Self {
        Self::new(v.x, v.y)
    }
}
impl From<Translation> for Vec2 {
    #[inline]
    fn from(t: Translation) -> Self {
        vec2(t.x, t.y)
    }
}
impl From<Translation2<f32>> for Translation {
    #[inline]
    fn from(t: Translation2<f32>) -> Self {
        Self::from(Vec2::from(t))
    }
}
impl From<Translation> for Translation2<f32> {
    #[inline]
    fn from(t: Translation) -> Self {
        Self::from(Vec2::from(t))
    }
}
impl From<Translation> for Isometry2<f32> {
    #[inline]
    fn from(t: Translation) -> Self {
        Self::from(Translation2::from(t))
    }
}
impl From<(f32, f32)> for Translation {
    fn from((x, y): (f32, f32)) -> Self {
        Translation::new(x, y)
    }
}
impl std::ops::Sub for Translation {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
impl std::ops::Add for Translation {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}
