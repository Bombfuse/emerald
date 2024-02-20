use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{Aseprite, AssetLoader, EmeraldError, World};

use super::Vec2f32Schema;

#[derive(Deserialize, Serialize)]
pub(crate) struct EntAsepriteSchema {
    pub aseprite: Option<String>,
    pub texture: Option<String>,
    pub animations: Option<String>,
    pub offset: Option<Vec2f32Schema>,
    pub visible: Option<bool>,
    pub centered: Option<bool>,
    pub scale: Option<Vec2f32Schema>,
    pub default_animation: Option<AsepriteDefaultAnimationSchema>,
    pub z_index: Option<f32>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct AsepriteDefaultAnimationSchema {
    pub name: String,
    pub looping: Option<bool>,
}

pub(crate) fn load_aseprite<'a>(
    loader: &mut AssetLoader<'a>,
    schema: EntAsepriteSchema,
) -> Result<Aseprite, EmeraldError> {
    if (schema.animations.is_none() || schema.texture.is_none()) && schema.aseprite.is_none() {
        return Err(EmeraldError::new("Failed to load Aseprite for entity. Either (animations AND texture) OR aseprite must be provided."));
    }

    let mut aseprite = None;

    if let Some(aseprite_path) = schema.aseprite {
        aseprite = Some(loader.aseprite(aseprite_path)?);
    }

    if let (Some(texture), Some(animations)) = (schema.texture, schema.animations) {
        aseprite = Some(loader.aseprite_with_animations(texture, animations)?);
    }

    let mut aseprite = aseprite.unwrap();
    aseprite.z_index = schema.z_index.unwrap_or(0.0);
    aseprite.visible = schema.visible.unwrap_or(true);
    aseprite.centered = schema.centered.unwrap_or(true);

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

    Ok(aseprite)
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
    let aseprite = load_aseprite(loader, schema)?;
    world.insert_one(entity, aseprite)?;

    Ok(())
}
