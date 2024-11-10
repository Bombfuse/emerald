use alloc::string::String;

use crate::EmeraldError;

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
        todo!()
    }
}
