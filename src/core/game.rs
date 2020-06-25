use crate::core::*;

pub trait Game {
    fn initialize(&mut self, _emd: Emerald) { }
    fn update(&mut self, _emd: Emerald) { }
}