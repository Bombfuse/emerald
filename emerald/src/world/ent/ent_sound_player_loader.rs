use alloc::{string::String, vec::Vec};
use hecs::Entity;
use serde::{Deserialize, Serialize};

use crate::{audio::components::sound_player::SoundPlayer, AssetLoader, EmeraldError, World};

#[derive(Deserialize, Serialize)]
pub(crate) struct EntSoundSchema {
    /// Label for the sound key
    pub label: String,

    /// Path to the sound file
    pub sound: String,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct EntSoundPlayerSchema {
    sounds: Vec<EntSoundSchema>,
    mixer: String,
}

pub const SOUND_PLAYER_SCHEMA_KEY: &str = "sound_player";

pub(crate) fn load_ent_sound_player<'a>(
    loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: &serde_json::Value,
) -> Result<(), EmeraldError> {
    if !toml.is_object() {
        return Err(EmeraldError::new(
            "Cannot load sprite from a non-table toml value.",
        ));
    }

    let schema: EntSoundPlayerSchema = serde_json::from_value(toml.clone())?;
    let mut sound_player = SoundPlayer::new(schema.mixer);

    for sound_schema in schema.sounds {
        let key = loader.sound(sound_schema.sound)?;
        sound_player.add_sound(&sound_schema.label, key);
    }

    world.insert_one(entity, sound_player)?;

    Ok(())
}
