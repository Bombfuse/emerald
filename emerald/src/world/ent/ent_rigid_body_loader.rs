use hecs::Entity;
use rapier2d::{
    parry::shape::Cuboid,
    prelude::{ColliderBuilder, ColliderHandle, RigidBodyBuilder, RigidBodyHandle, RigidBodyType},
};
use serde::{Deserialize, Serialize};

use crate::{AssetLoader, EmeraldError, World};

use super::Vec2f32Schema;

#[derive(Deserialize, Serialize)]
pub(crate) struct EntColliderSchema {
    pub shape: String,
    pub translation: Option<Vec2f32Schema>,
    pub half_width: Option<f32>,
    pub half_height: Option<f32>,
    pub sensor: Option<bool>,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct EntRigidBodySchema {
    pub body_type: String,
    pub colliders: Option<Vec<EntColliderSchema>>,
}

fn load_ent_collider(
    rbh: RigidBodyHandle,
    world: &mut World,
    collider_schema: EntColliderSchema,
) -> Result<ColliderHandle, EmeraldError> {
    // Load collider attributes
    let shape = Cuboid::new(Vector2::new(0.0, 0.0));
    let builder = ColliderBuilder::new(shape);
    Ok(world.physics().build_collider(rbh, builder))
}

pub(crate) fn load_ent_rigid_body<'a>(
    loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: &toml::Value,
) -> Result<RigidBodyHandle, EmeraldError> {
    if !toml.is_table() {
        return Err(EmeraldError::new(
            "Cannot load sprite from a non-table toml value.",
        ));
    }
    let schema: EntRigidBodySchema = toml::from_str(&toml.to_string())?;

    let mut body_type = RigidBodyType::Dynamic;
    match schema.body_type.as_str() {
        "dynamic" => {}
        "fixed" => body_type = RigidBodyType::Fixed,
        "kinematic_velocity_based" => body_type = RigidBodyType::KinematicVelocityBased,
        "kinematic_position_based" => body_type = RigidBodyType::KinematicPositionBased,
        _ => {
            return Err(EmeraldError::new(format!(
                "{:?} does not match a valid body type.",
                body_type_str
            )));
        }
    }

    let mut rigid_body_builder = RigidBodyBuilder::new(body_type);

    let rbh = world.physics().build_body(entity, body_builder)?;
    if let Some(collider_schemas) = schema.colliders {
        for collider_schema in collider_schemas {
            load_ent_collider(rbh, world, collider_schema)?;
        }
    }

    Ok(rbh)
}
