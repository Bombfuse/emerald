#![deny(future_incompatible, nonstandard_style)]

pub mod assets;
pub mod audio;
pub mod colors;
pub mod core;
pub mod events;
pub mod input;
pub mod rendering;
pub mod resources;
pub mod schedule;
pub mod system;
pub mod types;
pub mod world;
pub mod world_stack;

use crate::core::game_engine::{GameEngine, GameEngineContext};

pub use crate::assets::*;
pub use crate::colors::*;
pub use crate::core::*;
pub use crate::input::*;
pub use crate::rendering::*;
pub use crate::schedule::*;
pub use crate::types::*;
pub use crate::world::physics::*;
pub use crate::world::*;
pub use audio::*;

pub use serde;
pub use serde_json;
pub use toml;

pub use rapier2d::{
    crossbeam,
    dynamics::{RigidBody, RigidBodyBuilder, RigidBodyHandle},
    geometry::{Collider, ColliderBuilder, ColliderHandle, InteractionGroups, Ray},
    na as nalgebra,
    na::Vector2,
    parry,
};
