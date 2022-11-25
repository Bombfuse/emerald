use rapier2d::na::Vector2;
use serde::{Deserialize, Serialize};

use crate::{ent::Vec2f32Schema, AssetLoader, EmeraldError, World};

#[derive(Deserialize, Serialize)]
struct WorldPhysicsSchema {
    pub gravity: Option<Vec2f32Schema>,
}

pub(crate) fn load_world_physics<'a>(
    loader: &mut AssetLoader<'a>,
    world: &mut World,
    toml: &toml::Value,
) -> Result<(), EmeraldError> {
    if !toml.is_table() {
        return Err(EmeraldError::new(
            "Cannot load world physics from a non-table toml value.",
        ));
    }

    let schema: WorldPhysicsSchema = toml::from_str(&toml.to_string())?;

    if let Some(gravity) = schema.gravity {
        world
            .physics()
            .set_gravity(Vector2::new(gravity.x, gravity.y));
    }

    Ok(())
}
