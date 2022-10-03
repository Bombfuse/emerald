use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{AssetLoader, EmeraldError, World};

use super::Vec2f32Schema;

#[derive(Deserialize, Serialize)]
pub(crate) struct EntSpriteSchema {
    pub texture: String,
    pub offset: Option<Vec2f32Schema>,
    pub visible: Option<bool>,
    pub scale: Option<Vec2f32Schema>,
    pub z_index: Option<f32>,
}

pub(crate) fn load_ent_sprite<'a>(
    loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: &toml::Value,
) -> Result<(), EmeraldError> {
    if !toml.is_table() {
        return Err(EmeraldError::new(
            "Cannot load sprite from a non-table toml value.",
        ));
    }

    let schema: EntSpriteSchema = toml::from_str(&toml.to_string())?;
    let mut sprite = loader.sprite(schema.texture)?;
    sprite.z_index = schema.z_index.unwrap_or(0.0);
    sprite.visible = schema.visible.unwrap_or(true);

    if let Some(offset) = schema.offset {
        sprite.offset.x = offset.x;
        sprite.offset.y = offset.y;
    }
    if let Some(scale) = schema.scale {
        sprite.scale.x = scale.x;
        sprite.scale.y = scale.y;
    }

    world.insert_one(entity, sprite)?;

    Ok(())
}
