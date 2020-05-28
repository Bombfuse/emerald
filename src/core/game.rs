use crate::core::*;

pub trait Game {
    fn initialize(&mut self, emd: Emerald) {}
    fn update(&mut self, emd: Emerald) {}
}