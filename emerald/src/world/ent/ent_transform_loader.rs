use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{AssetLoader, EmeraldError, Scale, Transform, Translation, World};

use super::Vec2f32Schema;

#[derive(Deserialize, Serialize)]
pub(crate) struct EntTransformSchema {
    pub translation: Option<Vec2f32Schema>,
    pub rotation: Option<f32>,
    pub scale: Option<Vec2f32Schema>,
}

pub(crate) fn load_ent_transform<'a>(
    loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: &toml::Value,
) -> Result<(), EmeraldError> {
    if !toml.is_table() {
        return Err(EmeraldError::new(
            "Cannot load transform from a non-table toml value.",
        ));
    }

    let schema: EntTransformSchema = toml::from_str(&toml.to_string())?;
    let mut transform = Transform::default();

    if let Some(translation) = schema.translation {
        transform.translation = Translation::new(translation.x, translation.y);
    }

    if let Some(rotation) = schema.rotation {
        transform.rotation = rotation;
    }

    if let Some(scale) = schema.scale {
        transform.scale = Scale::new(scale.x, scale.y);
    }

    world.insert_one(entity, transform)?;

    Ok(())
}
