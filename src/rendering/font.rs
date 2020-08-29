

pub(crate) const DEFAULT_FONT_TEXTURE_PATH: &str = "ghosty_spooky_mister_mime_dude";

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FontKey(String, u32);
impl FontKey {
    pub fn new<T: Into<String>>(font_path: T, size: u32) -> Self {
        FontKey(font_path.into(), size)
    }
}
impl Default for FontKey {
    fn default() -> FontKey {
        FontKey::new(DEFAULT_FONT_TEXTURE_PATH, 32)
    }
}
