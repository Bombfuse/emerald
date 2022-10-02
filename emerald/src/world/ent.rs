use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{AssetLoader, EmeraldError, Transform, World};

use self::ent_sprite_loader::load_ent_sprite;
pub(crate) mod ent_aseprite_loader;
pub(crate) mod ent_rigid_body_loader;
pub(crate) mod ent_sprite_loader;

const SPRITE_SCHEMA_KEY: &str = "sprite";
const RIGID_BODY_SCHEMA_KEY: &str = "rigid_body";
const ASEPRITE_SCHEMA_KEY: &str = "aseprite";

#[derive(Default)]
pub struct EntLoadConfig<'a> {
    pub world: World,
    pub transform: Transform,
    pub custom_component_loader: Option<
        &'a dyn Fn(
            &mut AssetLoader<'_>,
            Entity,
            &mut World,
            toml::Value,
            String,
        ) -> Result<(), EmeraldError>,
    >,
}

pub(crate) fn load_ent(
    loader: &mut AssetLoader<'_>,
    world: &mut World,
    toml: String,
    config: EntLoadConfig<'_>,
) -> Result<Entity, EmeraldError> {
    let entity = world.spawn((config.transform,));

    let mut toml = toml.parse::<toml::Value>()?;

    if let Some(table) = toml.as_table_mut() {
        let table_keys = table
            .keys()
            .into_iter()
            .map(|key| key.clone())
            .collect::<Vec<String>>();
        for key in table_keys {
            match key.as_str() {
                SPRITE_SCHEMA_KEY => {
                    if let Some(sprite_value) = table.remove(SPRITE_SCHEMA_KEY) {
                        load_ent_sprite(loader, entity, world, &sprite_value)?;
                    }
                }
                _ => {
                    if let Some(custom_component_loader) = config.custom_component_loader {
                        if let Some(value) = table.remove(&key) {
                            custom_component_loader(loader, entity, world, value, key)?;
                        }
                    }
                }
            }
        }
    }

    Ok(entity)
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Vec2f32Schema {
    pub x: f32,
    pub y: f32,
}
