

pub(crate) const DEFAULT_FONT_TEXTURE_PATH: &str = "ghosty_spooky_mister_mime_dude";

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FontKey(String);
impl FontKey {
    pub fn new(font_path: &str) -> Self {
        FontKey(font_path.to_string())
    }
}
impl Default for FontKey {
    fn default() -> FontKey {
        FontKey::new(DEFAULT_FONT_TEXTURE_PATH)
    }
}
