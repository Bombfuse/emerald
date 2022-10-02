use hecs::Entity;

use crate::{AssetLoader, EmeraldError, World};

pub(crate) fn load_ent_sprite<'a>(
    loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: &mut toml::Value,
) -> Result<(), EmeraldError> {
    if !toml.is_table() {
        return Err(EmeraldError::new(
            "Cannot load sprite from a non-table toml value.",
        ));
    }

    if let Some(sprite_value) = toml.get("sprite") {
        if let Some(sprite_texture_value) = sprite_value.get("texture") {
            if let Some(texture_path) = sprite_texture_value.as_str() {
                let sprite = loader.sprite(texture_path)?;

                world.insert_one(entity, sprite)?;
            } else {
                return Err(EmeraldError::new("Expected a string as the texture value."));
            }
        } else {
            return Err(EmeraldError::new("Expected a texture field"));
        }
    }

    Ok(())
}
