use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{AssetLoader, EmeraldError, World};

use super::Vec2f32Schema;

#[derive(Deserialize, Serialize)]
pub(crate) struct EntAsepriteSchema {
    pub aseprite: String,
    pub offset: Option<Vec2f32Schema>,
    pub visible: Option<bool>,
    pub scale: Option<Vec2f32Schema>,
    pub default_animation: Option<AsepriteDefaultAnimationSchema>,
    pub z_index: Option<f32>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct AsepriteDefaultAnimationSchema {
    pub name: String,
    pub looping: Option<bool>,
}

pub(crate) fn load_ent_aseprite<'a>(
    loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: &toml::Value,
) -> Result<(), EmeraldError> {
    if !toml.is_table() {
        return Err(EmeraldError::new(
            "Cannot load aseprite from a non-table toml value.",
        ));
    }
    let schema: EntAsepriteSchema = toml::from_str(&toml.to_string())?;

    let mut aseprite = loader.aseprite(schema.aseprite)?;

    aseprite.z_index = schema.z_index.unwrap_or(0.0);
    aseprite.visible = schema.visible.unwrap_or(true);

    if let Some(offset) = schema.offset {
        aseprite.offset.x = offset.x;
        aseprite.offset.y = offset.y;
    }

    if let Some(scale) = schema.scale {
        aseprite.scale.x = scale.x;
        aseprite.scale.y = scale.y;
    }

    if let Some(default_animation_schema) = schema.default_animation {
        let looping = default_animation_schema.looping.unwrap_or(false);
        if looping {
            aseprite.play_and_loop(default_animation_schema.name)?;
        } else {
            aseprite.play(default_animation_schema.name)?;
        }
    }

    world.insert_one(entity, aseprite)?;

    Ok(())
}
