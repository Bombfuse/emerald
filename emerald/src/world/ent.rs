use alloc::{string::String, vec::Vec};
use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{
    autotilemap::load_ent_autotilemap, tilemap::load_ent_tilemap, AssetLoader, EmeraldError,
    Transform, World,
};

use self::{
    ent_color_rect_loader::load_ent_color_rect, ent_label_loader::load_ent_label,
    ent_sound_player_loader::SOUND_PLAYER_SCHEMA_KEY, ent_sprite_loader::load_ent_sprite,
    ent_transform_loader::load_ent_transform,
};

pub(crate) mod ent_color_rect_loader;
pub(crate) mod ent_label_loader;
pub(crate) mod ent_sound_player_loader;
pub(crate) mod ent_sprite_loader;
pub(crate) mod ent_transform_loader;

const SPRITE_SCHEMA_KEY: &str = "sprite";
const RIGID_BODY_SCHEMA_KEY: &str = "rigid_body";
const ASEPRITE_SCHEMA_KEY: &str = "aseprite";
const GRAPHICS_STACK_SCHEMA_KEY: &str = "gfx_stack";
pub(crate) const TRANSFORM_SCHEMA_KEY: &str = "transform";
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
    toml: &mut crate::serde_json::Value,
    transform: Transform,
) -> Result<Entity, EmeraldError> {
    let entity = world.spawn((transform,));
    let mut custom_components = Vec::new();

    if let Some(table) = toml.as_object_mut() {
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
                SOUND_PLAYER_SCHEMA_KEY => {
                    if let Some(value) = table.remove(SOUND_PLAYER_SCHEMA_KEY) {
                        ent_sound_player_loader::load_ent_sound_player(
                            loader, entity, world, &value,
                        )?;
                    }
                }
                _ => {
                    if let Some(value) = table.remove(&key) {
                        custom_components.push((key, value));
                    }
                }
            }
        }
    }

    // Custom components are loaded after all engine components
    for (key, value) in custom_components {
        loader
            .asset_engine
            .load_config
            .custom_component_loader
            .map(|loader_fn| loader_fn(loader, entity, world, value.clone(), key.clone()).unwrap());

        loader
            .asset_engine
            .load_config
            .component_deser_registry
            .get(&key)
            .map(|load_fn| load_fn(value, world, entity));
    }

    Ok(entity)
}

pub(crate) fn load_ent_from_toml(
    loader: &mut AssetLoader<'_>,
    world: &mut World,
    toml: String,
    transform: Transform,
) -> Result<Entity, EmeraldError> {
    let mut value = toml.parse::<serde_json::Value>()?;
    load_ent(loader, world, &mut value, transform)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Vec2f32Schema {
    pub x: f32,
    pub y: f32,
}
