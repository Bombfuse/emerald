use alloc::string::String;
use fixed::traits::ToFixed;
use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{AssetLoader, EmeraldError, Rectangle, Sprite, World};

use super::Vec2f32Schema;

#[derive(Deserialize, Serialize)]
pub(crate) struct EntSpriteSchema {
    pub texture: String,

    #[serde(default)]
    pub offset: Option<Vec2f32Schema>,

    #[serde(default)]
    pub visible: Option<bool>,

    #[serde(default)]
    pub scale: Option<Vec2f32Schema>,

    #[serde(default)]
    pub z_index: Option<f32>,

    #[serde(default)]
    pub target: Option<Rectangle>,
}

pub(crate) fn load_sprite<'a>(
    loader: &mut AssetLoader<'a>,
    schema: EntSpriteSchema,
) -> Result<Sprite, EmeraldError> {
    let mut sprite = loader.sprite(schema.texture)?;
    sprite.z_index = schema.z_index.unwrap_or(0.0);
    sprite.visible = schema.visible.unwrap_or(true);

    if let Some(offset) = schema.offset {
        sprite.offset.x = offset.x.to_fixed();
        sprite.offset.y = offset.y.to_fixed();
    }
    if let Some(scale) = schema.scale {
        sprite.scale.x = scale.x.to_fixed();
        sprite.scale.y = scale.y.to_fixed();
    }

    schema.target.map(|t| sprite.target = t);

    Ok(sprite)
}

pub(crate) fn load_ent_sprite<'a>(
    loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: &serde_json::Value,
) -> Result<(), EmeraldError> {
    if !toml.is_object() {
        return Err(EmeraldError::new(
            "Cannot load sprite from a non-table toml value.",
        ));
    }

    let schema: EntSpriteSchema = serde_json::from_value(toml.clone())?;
    let sprite = load_sprite(loader, schema)?;

    world.insert_one(entity, sprite)?;

    Ok(())
}
