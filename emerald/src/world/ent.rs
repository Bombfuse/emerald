use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{AssetLoader, EmeraldError, Transform, World};

use self::ent_sprite_loader::load_ent_sprite;
pub(crate) mod ent_sprite_loader;

pub(crate) fn load_ent<'a>(
    loader: &mut AssetLoader<'a>,
    world: &mut World,
    transform: Transform,
    toml: String,
) -> Result<Entity, EmeraldError> {
    let entity = world.spawn((transform,));

    let mut toml = toml.parse::<toml::Value>()?;
    load_ent_sprite(loader, entity, world, &mut toml)?;

    Ok(entity)
}

#[derive(Deserialize, Serialize)]
pub(crate) struct Vec2f32Schema {
    pub x: f32,
    pub y: f32,
}
