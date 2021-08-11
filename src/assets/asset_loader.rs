use crate::assets::*;
use crate::audio::*;
use crate::rendering::*;
use crate::*;

use std::ffi::OsStr;

#[cfg(target_arch = "wasm32")]
fn read_file(path: &str) -> Result<Vec<u8>, EmeraldError> {
    Err(EmeraldError::new(format!(
        "Unable to get bytes for {}",
        path
    )))
}

#[cfg(target_os = "android")]
fn read_file(path: &str) -> Result<Vec<u8>, EmeraldError> {
    // Based on https://github.com/not-fl3/miniquad/blob/4be5328760ff356494caf59cc853bcb395bce5d2/src/fs.rs#L38-L53

    let filename = std::ffi::CString::new(path).unwrap();

    let mut data: sapp_android::android_asset = unsafe { std::mem::zeroed() };

    unsafe { sapp_android::sapp_load_asset(filename.as_ptr(), &mut data as _) };

    if data.content.is_null() == false {
        let slice = unsafe { std::slice::from_raw_parts(data.content, data.content_length as _) };
        let response = slice.iter().map(|c| *c as _).collect::<Vec<_>>();
        Ok(response)
    } else {
        Err(EmeraldError::new(format!(
            "Unable to load asset `{}`",
            path
        )))
    }
}

#[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
fn read_file(path: &str) -> Result<Vec<u8>, EmeraldError> {
    use std::fs::File;
    use std::io::Read;

    let current_dir = std::env::current_dir()?;
    let path = current_dir.join(path);
    let path = path.into_os_string().into_string()?;

    let mut contents = vec![];
    let mut file = File::open(path)?;
    file.read_to_end(&mut contents)?;
    Ok(contents)
}

pub struct AssetLoader<'a> {
    pub(crate) quad_ctx: &'a mut miniquad::Context,
    asset_store: &'a mut AssetStore,
    rendering_engine: &'a mut RenderingEngine,
    _audio_engine: &'a mut AudioEngine,
}
impl<'a> AssetLoader<'a> {
    pub(crate) fn new(
        quad_ctx: &'a mut miniquad::Context,
        asset_store: &'a mut AssetStore,
        rendering_engine: &'a mut RenderingEngine,
        _audio_engine: &'a mut AudioEngine,
    ) -> Self {
        AssetLoader {
            quad_ctx,
            asset_store,
            rendering_engine,
            _audio_engine,
        }
    }

    pub fn bytes<T: Into<String>>(&mut self, file_path: T) -> Result<Vec<u8>, EmeraldError> {
        let path: String = file_path.into();
        if let Some(bytes) = self.asset_store.get_bytes(&path) {
            return Ok(bytes);
        }

        let bytes = read_file(&path)?;
        self.asset_store.insert_bytes(path, bytes.clone());

        Ok(bytes)
    }

    /// Loads bytes from given path as a string
    pub fn string<T: Into<String>>(&mut self, file_path: T) -> Result<String, EmeraldError> {
        let bytes = self.bytes(file_path)?;
        let string = String::from_utf8(bytes)?;

        Ok(string)
    }

    pub fn font<T: Into<String>>(
        &mut self,
        file_path: T,
        font_size: u32,
    ) -> Result<FontKey, EmeraldError> {
        let file_path: String = file_path.into();
        let key = FontKey::new(file_path.clone(), font_size);

        if self.asset_store.get_font(&key).is_some() {
            return Ok(key);
        }

        let font_image = FontImage::gen_image_color(512, 512, Color::new(0, 0, 0, 0));
        let font_texture_key = TextureKey::new(key.0.clone());
        let font_texture = Texture::from_rgba8(
            &mut self.quad_ctx,
            font_texture_key.clone(),
            font_image.width,
            font_image.height,
            &font_image.bytes,
        )?;
        let font_bytes = self.bytes(file_path)?;
        let mut font_settings = fontdue::FontSettings::default();
        font_settings.scale = font_size as f32;
        let inner_font = fontdue::Font::from_bytes(font_bytes, font_settings)?;
        let font = Font::new(key.clone(), font_texture_key.clone(), font_image)?;

        self.asset_store
            .insert_texture(&mut self.quad_ctx, font_texture_key, font_texture);
        self.asset_store
            .insert_fontdue_font(key.clone(), inner_font);
        self.asset_store
            .insert_font(&mut self.quad_ctx, key.clone(), font)?;

        Ok(key)
    }

    /// TODO(bombfuse): Automatically load texture and animations from a .aseprite
    // fn aseprite() {}

    pub fn aseprite_with_animations<T: Into<String>>(
        &mut self,
        path_to_texture: T,
        path_to_animations: T,
    ) -> Result<Aseprite, EmeraldError> {
        let texture_path: String = path_to_texture.into();
        let animation_path: String = path_to_animations.into();

        let aseprite_data = self.bytes(animation_path)?;

        let sprite = self.sprite(texture_path)?;
        let aseprite = Aseprite::new(sprite, aseprite_data)?;

        Ok(aseprite)
    }

    pub fn texture<T: Into<String>>(&mut self, path: T) -> Result<TextureKey, EmeraldError> {
        let path: String = path.into();
        let key = TextureKey::new(path.clone());

        if self.asset_store.get_texture(&key).is_some() {
            return Ok(key);
        }

        let data = self.bytes(path)?;
        let texture = Texture::new(&mut self.quad_ctx, key.clone(), data)?;
        self.asset_store
            .insert_texture(&mut self.quad_ctx, key.clone(), texture);

        Ok(key)
    }

    /// Creating render textures is slightly expensive and should be used conservatively.
    /// Please re-use render textures you've created before if possible.
    /// If you need a render texture with a new size, you should create a new render texture.
    pub fn render_texture(&mut self, w: usize, h: usize) -> Result<TextureKey, EmeraldError> {
        self.rendering_engine
            .create_render_texture(w, h, &mut self.quad_ctx, &mut self.asset_store)
    }

    pub fn sprite<T: Into<String>>(&mut self, path: T) -> Result<Sprite, EmeraldError> {
        let path: String = path.into();
        let texture_key = self.texture(path)?;

        Ok(Sprite::from_texture(texture_key))
    }

    /// Load the sound at the given path into the given mixer.
    /// Returns the sound handle to play the sound with.
    pub fn sound<T: Into<String>>(&mut self, path: T) -> Result<SoundKey, EmeraldError> {
        let path: String = path.into();
        let file_path = std::path::Path::new(&path);
        let sound_format = match file_path.extension().and_then(OsStr::to_str) {
            Some("wav") => SoundFormat::Wav,
            Some("ogg") => SoundFormat::Ogg,
            _ => {
                return Err(EmeraldError::new(format!(
                    "File must be wav or ogg. Found {:?}",
                    file_path
                )))
            }
        };

        let key = SoundKey::new(path.clone(), sound_format);
        if self.asset_store.sound_map.contains_key(&key) {
            return Ok(key);
        }

        let sound_bytes = self.bytes(path.clone())?;
        let sound = Sound::new(sound_bytes, sound_format)?;

        if !self.asset_store.sound_map.contains_key(&key) {
            self.asset_store.sound_map.insert(key.clone(), sound);
        }

        Ok(key)
    }

    pub fn pack_bytes(&mut self, name: &str, bytes: Vec<u8>) -> Result<(), EmeraldError> {
        self.asset_store.insert_bytes(name.into(), bytes);

        Ok(())
    }

    pub fn preload_texture<T: Into<String>>(&mut self, name: T) -> Result<(), EmeraldError> {
        let name: String = name.into();

        if let Ok(_sprite) = self.sprite(name.clone()) {
            return Ok(());
        }

        Err(EmeraldError::new(format!(
            "Unable to preload texture {}",
            name
        )))
    }
}
