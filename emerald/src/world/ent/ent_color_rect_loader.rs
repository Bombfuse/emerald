use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{rendering::components::ColorRect, AssetLoader, Color, EmeraldError, World};

#[derive(Deserialize, Serialize)]
pub(crate) struct EntColorRectSchema {
    pub color: Color,
    pub width: u32,
    pub height: u32,
}

pub(crate) fn load_ent_color_rect<'a>(
    _loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: &toml::Value,
) -> Result<(), EmeraldError> {
    if !toml.is_table() {
        return Err(EmeraldError::new(
            "Cannot load color_rect from a non-table toml value.",
        ));
    }

    let schema: EntColorRectSchema = toml::from_str(&toml.to_string())?;
    let color_rect = ColorRect::new(schema.color, schema.width, schema.height);
    world.insert_one(entity, color_rect)?;

    Ok(())
}
