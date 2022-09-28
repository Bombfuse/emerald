use crate::EmeraldError;

use super::profile_cache::ProfileCache;

pub type ProfileFloat = f64;

pub type ProfileName = String;

/// Captures the current time at instantiation, profiler should only be used inline.
/// Ex.
/// emd.profiler().start_frame("game_loop").unwrap();
/// do_my_game_logic();
/// emd.profiler().finish_frame("game_loop").unwrap();
pub struct Profiler<'c> {
    profile_cache: &'c mut ProfileCache,
    profile_name: ProfileName,
    now: ProfileFloat,
}
impl<'c> Profiler<'c> {
    pub(crate) fn new<T: Into<ProfileName>>(
        profile_cache: &'c mut ProfileCache,
        profile_name: T,
        now: ProfileFloat,
    ) -> Self {
        Self {
            profile_cache,
            profile_name: profile_name.into(),
            now,
        }
    }

    pub fn start_frame(&mut self) -> Result<(), EmeraldError> {
        self.profile_cache
            .start_frame(self.profile_name.clone(), self.now)
    }

    pub fn finish_frame(&mut self) -> Result<ProfileFloat, EmeraldError> {
        self.profile_cache
            .finish_frame(self.profile_name.clone(), self.now)
    }
}
