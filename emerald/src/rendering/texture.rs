use image::GenericImageView;

use crate::{
    asset_key::AssetKey,
    rendering_engine::{BindGroupLayoutId, BindGroupLayouts},
    AssetEngine, EmeraldError,
};
pub const EMERALD_DEFAULT_TEXTURE_NAME: &str = "emerald_default_texture";

#[derive(Clone, Debug, PartialEq)]
pub struct TextureKey {
    label: String,
    cached_size: (u32, u32),
    pub(crate) asset_key: AssetKey,
    pub(crate) bind_group_key: AssetKey,
}
impl TextureKey {
    pub(crate) fn new(
        label: &str,
        cached_size: (u32, u32),
        asset_key: AssetKey,
        bind_group_key: AssetKey,
    ) -> Self {
        TextureKey {
            label: label.to_string(),
            asset_key,
            bind_group_key,
            cached_size,
        }
    }

    /// Returns the size of the texture at the time this key was created.
    /// This can be innacurate if the texture has been resized since then.
    pub fn size(&self) -> (u32, u32) {
        self.cached_size
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}
