use emerald::{asset_key::AssetKey, AssetEngine, EmeraldError};
use image::GenericImageView;


use super::rendering_engine::{BindGroupLayoutId, BindGroupLayouts};

pub const EMERALD_DEFAULT_TEXTURE_NAME: &str = "emerald_default_texture";

pub(crate) struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: wgpu::Extent3d,
    pub bind_group_key: Option<AssetKey>,
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
    ) -> Result<AssetKey, EmeraldError> {
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
    ) -> Result<AssetKey, EmeraldError> {
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
    ) -> Result<AssetKey, EmeraldError> {
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
            view_formats: &[format],
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

        let mut texture = Self {
            texture,
            view,
            sampler,
            size,
            bind_group_key: None,
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
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
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

            let bind_group_key =
                asset_engine.add_asset_with_label(Box::new(texture_bind_group), label)?;
            texture.bind_group_key = Some(bind_group_key);
            let asset_key = asset_engine.add_asset_with_label(Box::new(texture), label)?;
            return Ok(asset_key);
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
    ) -> Result<AssetKey, EmeraldError> {
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
    ) -> Result<AssetKey, EmeraldError> {
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
