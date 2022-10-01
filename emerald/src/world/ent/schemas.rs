use serde::{Deserialize, Serialize};

pub mod sprite_schema;

#[derive(Serialize, Deserialize)]
pub struct Vector2Schema {
    pub x: f32,
    pub y: f32,
}
