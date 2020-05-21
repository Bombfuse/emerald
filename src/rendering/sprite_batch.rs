use crate::*;

pub struct SpriteKey(usize);
impl SpriteKey { 
    pub fn new(index: usize) -> Self {
        SpriteKey(index)
    }
}

pub struct SpriteBatch {
    texture: Texture,
    sprites: Vec<Sprite>,
}
impl SpriteBatch {
    pub fn new(texture: Texture) -> Self {
        SpriteBatch {
            texture,
            sprites: Vec::new(),
        }
    }

    pub fn add(&mut self, sprite: Sprite) -> SpriteKey {
        self.sprites.push(sprite);

        SpriteKey::new(self.sprites.len() - 1)
    }
}