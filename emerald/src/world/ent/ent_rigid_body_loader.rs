use hecs::Entity;
use nalgebra::Vector2;
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
    pub radius: Option<f32>,
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
    let mut builder = match collider_schema.shape.as_str() {
        "cuboid" => {
            if let (Some(half_width), Some(half_height)) =
                (collider_schema.half_width, collider_schema.half_height)
            {
                ColliderBuilder::cuboid(half_width, half_height)
            } else {
                return Err(EmeraldError::new(
                    "Cuboid colliders expect both a half_width and half_height.",
                ));
            }
        }
        "ball" => {
            if let Some(radius) = collider_schema.radius {
                ColliderBuilder::ball(radius)
            } else {
                return Err(EmeraldError::new("Ball colliders require a radius"));
            }
        }
        _ => {
            return Err(EmeraldError::new(
                "Collider shape does not match an expected shape.",
            ))
        }
    };

    if let Some(translation_value) = collider_schema.translation {
        builder = builder.translation(Vector2::new(translation_value.x, translation_value.y));
    }

    if let Some(sensor) = collider_schema.sensor {
        builder = builder.sensor(sensor);
    }

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
            "Cannot load rigid_body from a non-table toml value.",
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
                schema.body_type.as_str()
            )));
        }
    }

    let rigid_body_builder = RigidBodyBuilder::new(body_type);

    let rbh = world.physics().build_body(entity, rigid_body_builder)?;
    if let Some(collider_schemas) = schema.colliders {
        for collider_schema in collider_schemas {
            load_ent_collider(rbh, world, collider_schema)?;
        }
    }

    Ok(rbh)
}
