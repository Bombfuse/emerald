use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{AssetLoader, EmeraldError, Transform, World};

use self::ent_sprite_loader::load_ent_sprite;
pub(crate) mod ent_sprite_loader;

const SPRITE_SCHEMA_KEY: &str = "sprite";

pub(crate) fn load_ent<'a>(
    loader: &mut AssetLoader<'a>,
    world: &mut World,
    transform: Transform,
    toml: String,
) -> Result<Entity, EmeraldError> {
    let entity = world.spawn((transform,));

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
                _ => {}
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
