use image::GenericImageView;

use crate::EmeraldError;
pub const EMERALD_DEFAULT_TEXTURE_NAME: &str = "emerald_default_texture";

pub(crate) struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub key: TextureKey,
    pub size: wgpu::Extent3d,
}
impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float; // 1.

    pub fn new(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        key: TextureKey,
    ) -> Result<Self, EmeraldError> {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&key.0),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
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

        Ok(Self {
            texture,
            view,
            sampler,
            key,
            size,
        })
    }

    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        key: TextureKey,
    ) -> Result<Self, EmeraldError> {
        let img = image::load_from_memory(bytes)?;
        Self::from_image(device, queue, &img, key)
    }

    pub fn from_image(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        img: &image::DynamicImage,
        key: TextureKey,
    ) -> Result<Self, EmeraldError> {
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();
        let texture = Self::new(device, dimensions.0, dimensions.1, key)?;

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * dimensions.0),
                rows_per_image: std::num::NonZeroU32::new(dimensions.1),
            },
            texture.size,
        );

        Ok(texture)
    }
}

// #[derive(Clone, Debug)]
// pub struct Texture {
//     pub(crate) key: TextureKey,
//     pub(crate) inner: miniquad::Texture,
//     pub(crate) width: u16,
//     pub(crate) height: u16,
//     pub(crate) filter: FilterMode,
//     pub(crate) bindings: Bindings,
// }
// impl Texture {
//     pub(crate) fn new(
//         mut ctx: &mut Context,
//         key: TextureKey,
//         data: Vec<u8>,
//     ) -> Result<Self, EmeraldError> {
//         Self::from_png_bytes(&mut ctx, key, &data)
//     }

//     pub fn default(mut ctx: &mut Context) -> Result<Self, EmeraldError> {
//         let pixels: [u8; 4 * 4 * 4] = [
//             0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
//             0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
//             0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
//             0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
//             0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
//         ];

//         let texture = miniquad::Texture::from_rgba8(ctx, 4, 4, &pixels);

//         Self::from_texture(
//             &mut ctx,
//             TextureKey::new(EMERALD_DEFAULT_TEXTURE_NAME),
//             texture,
//         )
//     }

//     pub fn from_png_bytes(
//         ctx: &mut Context,
//         key: TextureKey,
//         bytes: &[u8],
//     ) -> Result<Self, EmeraldError> {
//         let img = image::load_from_memory(bytes)?.to_rgba8();
//         let img = image::imageops::flip_vertical(&img);

//         let width = img.width() as u16;
//         let height = img.height() as u16;
//         let bytes = img.into_raw();

//         Self::from_rgba8(ctx, key, width, height, &bytes)
//     }

//     pub(crate) fn from_rgba8(
//         mut ctx: &mut Context,
//         key: TextureKey,
//         width: u16,
//         height: u16,
//         bytes: &[u8],
//     ) -> Result<Self, EmeraldError> {
//         let texture = miniquad::Texture::from_rgba8(&mut ctx, width, height, bytes);

//         Self::from_texture(&mut ctx, key, texture)
//     }

//     pub(crate) fn from_texture(
//         ctx: &mut miniquad::Context,
//         key: TextureKey,
//         texture: miniquad::Texture,
//     ) -> Result<Self, EmeraldError> {
//         #[rustfmt::skip]
//         let vertices: [Vertex; 4] = [
//             Vertex { position: Vec2::new(0.0, 0.0) },
//             Vertex { position: Vec2::new(1.0, 0.0) },
//             Vertex { position: Vec2::new(1.0, 1.0) },
//             Vertex { position: Vec2::new(0.0, 1.0) },
//         ];

//         let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, &vertices);
//         let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
//         let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, &indices);
//         let bindings = Bindings {
//             vertex_buffers: vec![vertex_buffer],
//             index_buffer,
//             images: vec![texture],
//         };

//         Ok(Texture {
//             key,
//             width: texture.width as u16,
//             height: texture.height as u16,
//             inner: texture,
//             bindings,
//             filter: FilterMode::Nearest,
//         })
//     }

//     pub(crate) fn update(&mut self, font_image: &FontImage) {
//         assert_eq!(self.inner.width, font_image.width as u32);
//         assert_eq!(self.inner.height, font_image.height as u32);

//         self.inner.update(ctx, &font_image.bytes);
//     }
// }

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct TextureKey(pub(crate) String);
impl TextureKey {
    pub(crate) fn new<T: Into<String>>(texture_path: T) -> Self {
        TextureKey(texture_path.into())
    }

    pub fn get_name(&self) -> String {
        self.0.clone()
    }
}
impl Default for TextureKey {
    fn default() -> TextureKey {
        TextureKey(String::from(EMERALD_DEFAULT_TEXTURE_NAME))
    }
}
