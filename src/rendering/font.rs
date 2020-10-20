pub(crate) const DEFAULT_FONT_TEXTURE_PATH: &str = "ghosty_spooky_mister_mime_dude";

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FontId(String, u32);
impl FontId {
    pub fn new<T: Into<String>>(font_path: T, size: u32) -> Self {
        FontId(font_path.into(), size)
    }
}
impl Default for FontId {
    fn default() -> FontId {
        FontId::new(DEFAULT_FONT_TEXTURE_PATH, 32)
    }
}
