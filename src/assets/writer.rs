use crate::EmeraldError;
use std::fs::File;
use std::io::prelude::Write;

#[derive(Clone)]
pub struct Writer {
    user_directory: String,
}
impl Writer {
    pub(crate) fn new(user_directory: String) -> Self {
        Writer { user_directory }
    }

    pub fn write_to_user_file<T: Into<String>>(
        &mut self,
        bytes: &[u8],
        relative_path: T,
    ) -> Result<(), EmeraldError> {
        let path = self.user_directory.clone() + &relative_path.into();
        let mut file = File::create(path)?;
        file.write_all(bytes)?;

        Ok(())
    }
}
