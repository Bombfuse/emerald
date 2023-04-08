use image::GenericImageView;
use wgpu::{BindGroup, BindGroupLayout};

use crate::{
    asset_key::AssetKey,
    rendering_engine::{BindGroupLayoutId, BindGroupLayouts},
    AssetEngine, EmeraldError,
};
pub const EMERALD_DEFAULT_TEXTURE_NAME: &str = "emerald_default_texture";

pub(crate) struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
}
impl Texture {
    pub fn new(
        label: &str,
        bind_group_layouts: &BindGroupLayouts,
        asset_store: &mut AssetEngine,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Result<TextureKey, EmeraldError> {
        Self::new_ext(
            label,
            bind_group_layouts,
            asset_store,
            device,
            queue,
            width,
            height,
            data,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            wgpu::TextureFormat::Rgba8UnormSrgb,
        )
    }

    pub fn new_render_target(
        label: &str,
        bind_group_layouts: &BindGroupLayouts,
        asset_store: &mut AssetEngine,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        data: &[u8],
        format: wgpu::TextureFormat,
    ) -> Result<TextureKey, EmeraldError> {
        Self::new_ext(
            label,
            bind_group_layouts,
            asset_store,
            device,
            queue,
            width,
            height,
            data,
            wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
        )
    }

    fn new_ext(
        label: &str,
        bind_group_layouts: &BindGroupLayouts,
        asset_engine: &mut AssetEngine,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        data: &[u8],
        usage: wgpu::TextureUsages,
        format: wgpu::TextureFormat,
    ) -> Result<TextureKey, EmeraldError> {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture = Self {
            texture,
            view,
            sampler,
            size,
        };

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * width),
                rows_per_image: std::num::NonZeroU32::new(height),
            },
            texture.size,
        );

        if let Some(texture_bind_group_layout) =
            bind_group_layouts.get(&BindGroupLayoutId::TextureQuad)
        {
            let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&texture.sampler),
                    },
                ],
                label: Some(&format!("{:?}_group", label)),
            });
            let cached_size = (texture.size.width, texture.size.height);

            let bind_group_key =
                asset_engine.add_asset_with_label(Box::new(texture_bind_group), label)?;
            let asset_key = asset_engine.add_asset_with_label(Box::new(texture), label)?;
            return Ok(TextureKey::new(
                label,
                cached_size,
                asset_key,
                bind_group_key,
            ));
        }
        return Err(EmeraldError::new(
            "Unable to get TextureQuad bind group layout",
        ));
    }

    pub fn from_bytes(
        label: &str,
        bind_group_layouts: &BindGroupLayouts,
        asset_store: &mut AssetEngine,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
    ) -> Result<TextureKey, EmeraldError> {
        match image::load_from_memory(bytes) {
            Ok(img) => {
                Self::from_image(label, bind_group_layouts, asset_store, device, queue, &img)
            }
            Err(e) => Err(EmeraldError::new(format!(
                "Error loading image from memory. Texture Key: {:?} Err: {:?}",
                label, e
            ))),
        }
    }

    pub fn from_image(
        label: &str,
        bind_group_layouts: &BindGroupLayouts,
        asset_store: &mut AssetEngine,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
    ) -> Result<TextureKey, EmeraldError> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        Self::new(
            label,
            bind_group_layouts,
            asset_store,
            device,
            queue,
            dimensions.0,
            dimensions.1,
            &rgba,
        )
    }
}

pub(crate) fn get_texture_key(asset_engine: &mut AssetEngine, label: &str) -> Option<TextureKey> {
    if let (Some(texture_asset_key), Some(bind_group_key)) = (
        asset_engine.get_asset_key_by_label::<Texture>(label),
        asset_engine.get_asset_key_by_label::<BindGroup>(label),
    ) {
        if let Some(texture) = asset_engine.get_asset::<Texture>(&texture_asset_key.asset_id) {
            return Some(TextureKey::new(
                label,
                (texture.size.width, texture.size.height),
                texture_asset_key,
                bind_group_key,
            ));
        }
    }

    None
}

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
