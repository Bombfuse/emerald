use std::collections::{HashMap, VecDeque};

use crate::EmeraldError;

use super::{
    profile_settings::ProfileSettings,
    profiler::{ProfileFloat, ProfileName},
};

pub(crate) struct ProfileCache {
    settings: ProfileSettings,

    profiles: HashMap<ProfileName, Profile>,

    /// Map of ProfileName to (start, has_ended).
    in_progress_profiles: HashMap<ProfileName, (ProfileFloat, bool)>,
}
impl ProfileCache {
    pub(crate) fn new(settings: ProfileSettings) -> Self {
        Self {
            settings,
            profiles: HashMap::new(),
            in_progress_profiles: HashMap::new(),
        }
    }

    pub fn start_frame<T: Into<ProfileName>>(
        &mut self,
        profile_name: T,
        now: ProfileFloat,
    ) -> Result<(), EmeraldError> {
        let profile_name: ProfileName = profile_name.into();
        if self.in_progress_profiles.contains_key(&profile_name) {
            return Err(EmeraldError::new(format!(
                "Profile Error: {} has already been started.",
                profile_name
            )));
        }

        if !self.profiles.contains_key(&profile_name) {
            self.profiles
                .insert(profile_name.clone(), Profile::new(&self.settings));
        }

        self.in_progress_profiles.insert(profile_name, (now, false));

        Ok(())
    }

    pub fn finish_frame<T: Into<String>>(
        &mut self,
        profile_name: T,
        now: ProfileFloat,
    ) -> Result<ProfileFloat, EmeraldError> {
        let profile_name: ProfileName = profile_name.into();
        if let Some(in_progress_profile) = self.in_progress_profiles.get_mut(&profile_name) {
            if in_progress_profile.1 {
                return Err(EmeraldError::new(format!(
                    "Profile Error: {} has already ended.",
                    profile_name
                )));
            }

            in_progress_profile.1 = true;
            let new_frame = (in_progress_profile.0, now);

            if let Some(profile) = self.profiles.get_mut(&profile_name) {
                profile.add_frame(new_frame, &self.settings);
                self.in_progress_profiles.remove(&profile_name);
                return Ok(new_frame.1 - new_frame.0);
            } else {
                return Err(EmeraldError::new(format!(
                    "Profile Error: Profile {} does not exist.",
                    profile_name
                )));
            }
        }

        Err(EmeraldError::new(format!(
            "Profile Error: {} was never started.",
            profile_name
        )))
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Profile {
    /// (start, finish)
    pub frames: VecDeque<(ProfileFloat, ProfileFloat)>,
}
impl Profile {
    pub fn new(settings: &ProfileSettings) -> Self {
        Self {
            frames: VecDeque::with_capacity(settings.frame_limit),
        }
    }

    pub fn add_frame(
        &mut self,
        new_frame: (ProfileFloat, ProfileFloat),
        settings: &ProfileSettings,
    ) {
        self.frames.push_front(new_frame);

        if self.frames.len() >= settings.frame_limit {
            self.frames.pop_back();
        }
    }
}
