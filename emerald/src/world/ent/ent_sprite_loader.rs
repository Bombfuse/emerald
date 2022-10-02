use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{AssetLoader, EmeraldError, World};

use super::Vec2f32Schema;

#[derive(Deserialize, Serialize)]
pub(crate) struct SpriteSchema {
    pub texture: String,
    pub offset: Vec2f32Schema,
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

    let schema: SpriteSchema = toml::from_str(&toml.to_string())?;
    let mut sprite = loader.sprite(schema.texture)?;
    sprite.offset.x = schema.offset.x;
    sprite.offset.y = schema.offset.y;

    world.insert_one(entity, sprite)?;

    Ok(())
}
