use miniquad::FilterMode;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct TextureMeta {
    pub(crate) inner: miniquad::Texture,
    pub(crate) filter: FilterMode,
}
impl TextureMeta {
    pub fn width(&self) -> u32 {
        self.inner.width
    }

    pub fn height(&self) -> u32 {
        self.inner.height
    }
}
