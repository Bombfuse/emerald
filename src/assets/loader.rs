use crate::*;

pub struct AssetLoader {

}
impl AssetLoader {
    pub fn new() -> Self {
        AssetLoader {

        }
    }

    pub fn sprite(&mut self, _path: &str) -> Result<Sprite, EmeraldError> {
        Ok(Sprite::default())
    }
}