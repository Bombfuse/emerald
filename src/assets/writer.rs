use crate::EmeraldError;
use std::fs::File;
use std::io::prelude::Write;

#[derive(Copy, Clone)]
pub struct Writer {}
impl Writer {
    pub(crate) fn new() -> Self {
        Writer {}
    }

    pub fn write<T: Into<String>>(&mut self, bytes: &[u8], path: T) -> Result<(), EmeraldError> {
        let mut file = File::create(path.into())?;
        file.write_all(bytes)?;

        Ok(())
    }
}
