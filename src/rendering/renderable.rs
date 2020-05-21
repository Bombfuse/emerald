use crate::*;

use miniquad::*;

pub trait Renderable {
    pub fn render(&self, ctx: &mut Context) -> Result<(), EmeraldError>;
}