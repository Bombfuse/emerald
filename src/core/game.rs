use crate::core::*;

pub trait Game {
    fn initialize(&mut self, _emd: Emerald) -> Result<(), EmeraldError> { Ok(()) }
    fn update(&mut self, _emd: Emerald) -> Result<(), EmeraldError> { Ok(()) }
}