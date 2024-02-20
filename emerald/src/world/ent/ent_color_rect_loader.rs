use hecs::Entity;
use rapier2d::na::Vector2;
use serde::{Deserialize, Serialize};

use crate::{
    rendering::components::ColorRect, AssetLoader, Color, EmeraldError, Translation, World,
};

fn default_visibility() -> bool {
    true
}

#[derive(Deserialize, Serialize)]
pub(crate) struct EntColorRectSchema {
    pub color: Color,
    pub width: u32,
    pub height: u32,

    #[serde(default)]
    pub offset: Translation,

    #[serde(default)]
    pub z_index: f32,

    #[serde(default = "default_visibility")]
    pub visible: bool,
}

pub(crate) fn load_color_rect<'a>(
    _loader: &mut AssetLoader<'a>,
    schema: EntColorRectSchema,
) -> Result<ColorRect, EmeraldError> {
    let mut color_rect = ColorRect::new(schema.color, schema.width, schema.height);
    color_rect.z_index = schema.z_index;
    color_rect.visible = schema.visible;
    color_rect.offset = Vector2::new(schema.offset.x, schema.offset.y);
    Ok(color_rect)
}

pub(crate) fn load_ent_color_rect<'a>(
    loader: &mut AssetLoader<'a>,
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
    let color_rect = load_color_rect(loader, schema)?;
    world.insert_one(entity, color_rect)?;

    Ok(())
}
