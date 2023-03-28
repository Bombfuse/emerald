use hecs::Entity;
use rapier2d::{
    control::{CharacterLength, KinematicCharacterController},
    na::{UnitVector2, Vector2},
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

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq)]
pub(crate) enum CharacterLengthSchema {
    Absolute,
    Relative,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct CharacterOffsetSchema {
    length_type: CharacterLengthSchema,
    value: f32,
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct KinematicCharacterControllerMeta {
    /// Excludes the entities own body from the movement queries.
    #[serde(default = "exclude_self_default")]
    pub exclude_self: bool,

    #[serde(default)]
    pub offset: Option<CharacterOffsetSchema>,

    #[serde(default)]
    pub up: Option<Vec2f32Schema>,
}
fn exclude_self_default() -> bool {
    true
}

#[derive(Deserialize, Serialize)]
pub(crate) struct EntRigidBodySchema {
    pub body_type: String,
    pub colliders: Option<Vec<EntColliderSchema>>,
    pub lock_rotations: Option<bool>,
    pub lock_translations: Option<bool>,
    pub character: Option<KinematicCharacterControllerMeta>,
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

    let mut rigid_body_builder = RigidBodyBuilder::new(body_type);

    if schema.lock_rotations.filter(|l| *l).is_some() {
        rigid_body_builder = rigid_body_builder.lock_rotations();
    }

    if schema.lock_translations.filter(|l| *l).is_some() {
        rigid_body_builder = rigid_body_builder.lock_translations();
    }

    let rbh = world.physics().build_body(entity, rigid_body_builder)?;

    if let Some(character) = schema.character {
        let mut controller = KinematicCharacterController::default();
        character.offset.map(|offset| {
            controller.offset = match offset.length_type {
                CharacterLengthSchema::Absolute => CharacterLength::Absolute(offset.value),
                CharacterLengthSchema::Relative => CharacterLength::Relative(offset.value),
            };
        });
        character
            .up
            .map(|v| controller.up = UnitVector2::new_normalize(v.to_vector2()));

        world
            .physics()
            .build_kinematic_character_controller(entity, controller)?;
    }

    if let Some(collider_schemas) = schema.colliders {
        for collider_schema in collider_schemas {
            load_ent_collider(rbh, world, collider_schema)?;
        }
    }

    Ok(rbh)
}

#[cfg(test)]
mod test {
    use crate::ent::ent_rigid_body_loader::CharacterLengthSchema;

    use super::KinematicCharacterControllerMeta;

    fn deser_kinematic_controller_meta() {
        let toml = r#"
            exclude_self = false
            offset = { length_type = "Relative", value = 0.01 }
            up = {x = 0.0, y = 1.0}
        "#;

        let meta: KinematicCharacterControllerMeta = crate::toml::from_str(toml).unwrap();
        assert_eq!(
            meta.offset.as_ref().unwrap().length_type,
            CharacterLengthSchema::Relative
        );
        assert_eq!(meta.offset.unwrap().value, 0.01);

        assert_eq!(meta.up.unwrap().y, 1.0);
        assert_eq!(meta.exclude_self, false);
    }
}
