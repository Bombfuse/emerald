use serde::{Deserialize, Serialize};

use super::Vector2Schema;

#[derive(Serialize, Deserialize)]
pub(crate) struct SpriteSchema {
    pub offset: Vector2Schema,
    pub texture: String,
    pub scale: Vector2Schema,
}
