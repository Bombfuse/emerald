use hecs::Entity;
use rapier2d::na::Vector2;
use serde::{Deserialize, Serialize};

use crate::{
    autotilemap::load_ent_autotilemap, tilemap::load_ent_tilemap, AssetLoader, EmeraldError,
    Transform, World,
};

use self::{
    ent_color_rect_loader::load_ent_color_rect, ent_label_loader::load_ent_label,
    ent_sprite_loader::load_ent_sprite, ent_transform_loader::load_ent_transform,
};
#[cfg(feature = "aseprite")]
pub(crate) mod ent_aseprite_loader;

pub(crate) mod ent_color_rect_loader;
pub(crate) mod ent_label_loader;
pub(crate) mod ent_rigid_body_loader;
pub(crate) mod ent_sprite_loader;
pub(crate) mod ent_transform_loader;

const SPRITE_SCHEMA_KEY: &str = "sprite";
const RIGID_BODY_SCHEMA_KEY: &str = "rigid_body";
const ASEPRITE_SCHEMA_KEY: &str = "aseprite";
const TRANSFORM_SCHEMA_KEY: &str = "transform";
const LABEL_SCHEMA_KEY: &str = "label";
const COLOR_RECT_SCHEMA_KEY: &str = "color_rect";
const AUTOTILEMAP_SCHEMA_KEY: &str = "autotilemap";
const TILEMAP_SCHEMA_KEY: &str = "tilemap";

#[derive(Default)]
pub struct EntLoadConfig {
    pub transform: Transform,
}

pub(crate) fn load_ent(
    loader: &mut AssetLoader<'_>,
    world: &mut World,
    toml: &mut crate::toml::Value,
    transform: Transform,
) -> Result<Entity, EmeraldError> {
    let entity = world.spawn((transform,));
    let mut custom_components = Vec::new();

    if let Some(table) = toml.as_table_mut() {
        let table_keys = table
            .keys()
            .into_iter()
            .map(|key| key.clone())
            .collect::<Vec<String>>();
        for key in table_keys {
            match key.as_str() {
                AUTOTILEMAP_SCHEMA_KEY => {
                    if let Some(value) = table.remove(AUTOTILEMAP_SCHEMA_KEY) {
                        load_ent_autotilemap(loader, entity, world, &value)?;
                    }
                }
                TILEMAP_SCHEMA_KEY => {
                    if let Some(value) = table.remove(TILEMAP_SCHEMA_KEY) {
                        load_ent_tilemap(loader, entity, world, &value)?;
                    }
                }
                COLOR_RECT_SCHEMA_KEY => {
                    if let Some(value) = table.remove(COLOR_RECT_SCHEMA_KEY) {
                        load_ent_color_rect(loader, entity, world, &value)?;
                    }
                }
                LABEL_SCHEMA_KEY => {
                    if let Some(label_value) = table.remove(LABEL_SCHEMA_KEY) {
                        load_ent_label(loader, entity, world, &label_value)?;
                    }
                }
                TRANSFORM_SCHEMA_KEY => {
                    if let Some(transform_value) = table.remove(TRANSFORM_SCHEMA_KEY) {
                        load_ent_transform(loader, entity, world, &transform_value)?;
                    }
                }
                SPRITE_SCHEMA_KEY => {
                    if let Some(sprite_value) = table.remove(SPRITE_SCHEMA_KEY) {
                        load_ent_sprite(loader, entity, world, &sprite_value)?;
                    }
                }
                RIGID_BODY_SCHEMA_KEY => {
                    if let Some(rigid_body_value) = table.remove(RIGID_BODY_SCHEMA_KEY) {
                        ent_rigid_body_loader::load_ent_rigid_body(
                            loader,
                            entity,
                            world,
                            &rigid_body_value,
                        )?;
                    }
                }
                ASEPRITE_SCHEMA_KEY => {
                    #[cfg(feature = "aseprite")]
                    {
                        if let Some(aseprite_value) = table.remove(ASEPRITE_SCHEMA_KEY) {
                            ent_aseprite_loader::load_ent_aseprite(
                                loader,
                                entity,
                                world,
                                &aseprite_value,
                            )?;
                        }
                    }
                }
                _ => {
                    if loader
                        .asset_store
                        .load_config
                        .custom_component_loader
                        .is_some()
                    {
                        if let Some(value) = table.remove(&key) {
                            custom_components.push((key, value));
                        }
                    }
                }
            }
        }
    }

    // Custom components are loaded after all engine components
    if let Some(custom_component_loader) = loader.asset_store.load_config.custom_component_loader {
        for (key, value) in custom_components {
            custom_component_loader(loader, entity, world, value, key)?;
        }
    }

    Ok(entity)
}

pub(crate) fn load_ent_from_toml(
    loader: &mut AssetLoader<'_>,
    world: &mut World,
    toml: String,
    transform: Transform,
) -> Result<Entity, EmeraldError> {
    let mut value = toml.parse::<toml::Value>()?;
    load_ent(loader, world, &mut value, transform)
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Vec2f32Schema {
    pub x: f32,
    pub y: f32,
}
impl Vec2f32Schema {
    pub fn to_vector2(self) -> Vector2<f32> {
        Vector2::new(self.x, self.y)
    }
}
