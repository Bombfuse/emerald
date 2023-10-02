use emerald::assets::asset_engine::AssetEngine;
use emerald::font::{Font, FontKey};
use emerald::render_settings::RenderSettings;
use emerald::rendering::components::get_bounding_box_of_triangle;
use emerald::rendering_engine::{
    DrawTexturedQuadCommand, DrawTexturedTriCommand, RenderingEngine, ScreenSize,
};
use emerald::{Color, EmeraldError, Rectangle, Transform, Vector2};
use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    ops::Range,
};

use emerald::asset_key::{AssetId, AssetKey};
use fontdue::layout::Layout;
use wgpu::{util::DeviceExt, TextureView};
use wgpu::{
    Adapter, BindGroup, BindGroupLayout, Device, InstanceDescriptor, Queue, RenderPipeline, Surface,
};
use winit::dpi::PhysicalSize;

use super::shaders;
use super::shaders::textured_quad::Vertex;
use super::texture::Texture;
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BindGroupLayoutId {
    TextureQuad,
}

/// A set of textured tris that will be drawn.
#[derive(Debug)]
pub(crate) struct TexturedTriDraw {
    pub texture_asset_id: AssetId,
    pub texture_bind_group_asset_id: AssetId,
    pub vertices_range: Range<u64>,
    pub indices_range: Range<u64>,
    count: usize,
    indices_per_draw: usize,
    vertices_per_draw: usize,
}
impl TexturedTriDraw {
    pub fn new(
        texture_asset_id: AssetId,
        texture_bind_group_asset_id: AssetId,
        vertices_start: u64,
        vertices_set_size: u64,
        vertices_per_draw: usize,
        indices_start: u64,
        indices_set_size: u64,
        indices_per_draw: usize,
    ) -> Self {
        Self {
            texture_asset_id,
            texture_bind_group_asset_id,
            vertices_range: vertices_start..vertices_start + vertices_set_size,
            indices_range: indices_start..indices_start + indices_set_size,
            count: 1,
            vertices_per_draw,
            indices_per_draw,
        }
    }

    /// Add a new vertices_set and indices_set to the call.
    /// Returns the index start for where to add the next indices_set
    pub fn add(&mut self, vertices_set_size: u64, indices_set_size: u64) {
        self.vertices_range.end += vertices_set_size;
        self.indices_range.end += indices_set_size;
        self.count += 1;
    }

    pub fn index_start(&self) -> u32 {
        (self.count() * self.vertices_per_draw) as u32
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

pub(crate) type BindGroupLayouts = HashMap<BindGroupLayoutId, BindGroupLayout>;

pub(crate) struct DesktopRenderingEngine {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub settings: RenderSettings,

    pub texture_quad_render_pipeline: RenderPipeline,
    pub bind_group_layouts: BindGroupLayouts,
    pub draw_queue: VecDeque<TexturedTriDraw>,

    pub index_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,

    vertices: Vec<Vertex>,
    indices: Vec<u32>,

    pub render_texture_uid: usize,

    color_rect_texture: AssetKey,

    pub active_render_texture_asset_id: Option<AssetId>,
    pub active_size: winit::dpi::PhysicalSize<u32>,

    layout: Layout,
}
impl DesktopRenderingEngine {
    pub fn new(
        window: &winit::window::Window,
        settings: RenderSettings,
        asset_store: &mut AssetEngine,
    ) -> Result<Self, EmeraldError> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(InstanceDescriptor::default());
        let surface = unsafe { instance.create_surface(window).unwrap() };
        let adapter = pollster::block_on(get_adapter(&instance, &surface))?;
        let (device, queue) = pollster::block_on(get_device_and_queue(&adapter))?;
        let capabilities = surface.get_capabilities(&adapter);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: capabilities.formats[0],
            view_formats: capabilities.formats.clone(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };
        surface.configure(&device, &config);

        let mut bind_group_layouts = HashMap::new();
        let draw_queue = VecDeque::new();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Textured Quad Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("./shaders/textured_quad.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let texture_quad_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
            });

        bind_group_layouts.insert(BindGroupLayoutId::TextureQuad, texture_bind_group_layout);

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(shaders::textured_quad::VERTICES),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(shaders::textured_quad::INDICES),
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
        });

        let color_rect_texture = Texture::new(
            "emd_default_texture",
            &bind_group_layouts,
            asset_store,
            &device,
            &queue,
            1,
            1,
            &[255, 255, 255, 255],
        )?;

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            active_size: size,
            settings,

            texture_quad_render_pipeline,
            bind_group_layouts,
            draw_queue,

            vertex_buffer,
            index_buffer,
            vertices: Vec::new(),
            indices: Vec::new(),

            color_rect_texture,

            render_texture_uid: 0,

            active_render_texture_asset_id: None,
            layout: Layout::new(fontdue::layout::CoordinateSystem::PositiveYUp),
        })
    }

    fn render_to_view(
        &mut self,
        asset_store: &mut AssetEngine,
        view: TextureView,
        view_name: &str,
    ) -> Result<(), EmeraldError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("Encoder: {:?}", view_name)),
            });

        {
            let (r, g, b, a) = self.settings.background_color.to_percentage_linear();
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(&format!("Render Pass {:?}", view_name)),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.consume_draw_queue(&mut render_pass, asset_store)?;
        }

        self.queue.submit([encoder.finish()]);

        Ok(())
    }

    #[inline]
    fn consume_draw_queue<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        asset_engine: &'a mut AssetEngine,
    ) -> Result<(), EmeraldError> {
        render_pass.set_pipeline(&self.texture_quad_render_pipeline);

        // Calculate vertices for every texture to be drawn, paired with their sprite data and vertex indices
        // for every tuple, draw that sprites texture bind group using that vertex index as the slice
        let vertex_set_size = (std::mem::size_of::<Vertex>() * 4) as u64;
        let indices_set_size: u64 = std::mem::size_of::<u32>() as u64 * 6;
        let vertices_set_count = self.vertex_buffer.size() / vertex_set_size;
        let indices_set_count = self.index_buffer.size() / indices_set_size as u64;

        if self.indices.len() as u64 > indices_set_count {
            self.index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&self.indices),
                    usage: self.index_buffer.usage(),
                });
        } else {
            self.queue
                .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices));
        }

        if self.vertices.len() as u64 > vertices_set_count {
            self.vertex_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(&self.vertices),
                        usage: self.vertex_buffer.usage(),
                    });
        } else {
            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        }

        while let Some(draw_call) = self.draw_queue.pop_back() {
            let indices_count = draw_call.count() as u32 * draw_call.indices_per_draw as u32;

            if let Some(texture_bind_group) =
                asset_engine.get_asset::<BindGroup>(&draw_call.texture_bind_group_asset_id)
            {
                render_pass.set_bind_group(0, texture_bind_group, &[]);

                render_pass
                    .set_vertex_buffer(0, self.vertex_buffer.slice(draw_call.vertices_range));
                render_pass.set_index_buffer(
                    self.index_buffer.slice(draw_call.indices_range),
                    wgpu::IndexFormat::Uint32,
                );

                render_pass.draw_indexed(0..indices_count, 0, 0..1);
            } else {
                return Err(EmeraldError::new(format!(
                    "Unable to find texture bind group for AssetId {:?}",
                    &draw_call.texture_bind_group_asset_id
                )));
            }
        }
        Ok(())
    }
}

impl RenderingEngine for DesktopRenderingEngine {
    fn initialize(&mut self, asset_engine: &mut AssetEngine) {}

    #[inline]
    fn update_font_texture(
        &mut self,
        asset_store: &mut AssetEngine,
        key: &FontKey,
    ) -> Result<(), EmeraldError> {
        if let Some(font) = asset_store.get_asset::<Font>(&key.asset_key().asset_id()) {
            if let Some(texture) =
                asset_store.get_asset::<Texture>(&font.font_texture_key.asset_id())
            {
                self.queue.write_texture(
                    wgpu::ImageCopyTexture {
                        aspect: wgpu::TextureAspect::All,
                        texture: &texture.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                    },
                    &font.font_image.bytes,
                    wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * font.font_image.width as u32),
                        rows_per_image: Some(font.font_image.height as u32),
                    },
                    texture.size,
                );

                return Ok(());
            }
        }

        Err(EmeraldError::new(format!(
            "Unable to update font texture {:?}",
            key
        )))
    }

    fn draw_textured_quad(
        &mut self,
        cmd: emerald::rendering_engine::DrawTexturedQuadCommand,
    ) -> Result<(), emerald::EmeraldError> {
        let mut target = cmd.texture_target_area;
        let texture_asset_id = cmd.texture_asset_id;
        let transform = cmd.transform;
        let active_size = cmd.current_render_target_size;
        let offset = cmd.offset;
        let texture_bind_group_asset_id = cmd
            .asset_engine
            .get_asset::<Texture>(&cmd.texture_asset_id)
            .map(|t| t.bind_group_key.as_ref().map(|k| k.asset_id()))
            .flatten();

        if texture_bind_group_asset_id.is_none() {
            return Err(EmeraldError::new(format!(
                "Unable to find bind group for Texture {}",
                texture_asset_id
            )));
        }
        let texture_bind_group_asset_id = texture_bind_group_asset_id.unwrap();

        let texture_size;
        if let Some(texture) = cmd.asset_engine.get_asset::<Texture>(&texture_asset_id) {
            texture_size = (texture.size.width as f32, texture.size.height as f32);

            // Zeroed target means display entire texture
            if target.is_zero_sized() {
                target.width = texture_size.0;
                target.height = texture_size.1;
            }
        } else {
            return Err(EmeraldError::new(format!(
                "Unable to find Texture for AssetId {:?}",
                texture_asset_id
            )));
        }

        // Add magic numbers to target semi-middle of pixels
        target.x += 0.275;
        target.y += 0.275;

        let mut x = transform.translation.x + offset.x;
        let mut y = transform.translation.y + offset.y;

        if cmd.pixel_snap {
            x = x.floor();
            y = y.floor();
        }

        let x = x / (active_size.width as f32 / 2.0);
        let y = y / (active_size.height as f32 / 2.0);

        let normalized_texture_size = (
            target.width / (active_size.width as f32 / 2.0),
            target.height / (active_size.height as f32 / 2.0),
        );

        {
            let x = target.x / texture_size.0;
            let y = target.y / texture_size.1;
            let width = target.width / texture_size.0;
            let height = target.height / texture_size.1;
            target = Rectangle::new(x, y, width, height);
        }

        let width = normalized_texture_size.0 * cmd.scale.x;
        let height = normalized_texture_size.1 * cmd.scale.y;
        let mut vertex_rect = Rectangle::new(x, y, width, height);

        if cmd.centered {
            vertex_rect.x -= width / 2.0;
            vertex_rect.y -= height / 2.0;
        }
        let center_x = vertex_rect.x + vertex_rect.width / 2.0;
        let center_y = vertex_rect.y + vertex_rect.height / 2.0;

        fn rotate_vertex(center_x: f32, center_y: f32, x: f32, y: f32, rotation: f32) -> [f32; 2] {
            let diff_x = x - center_x;
            let diff_y = y - center_y;
            [
                center_x + (rotation.cos() * diff_x) - (rotation.sin() * diff_y),
                center_y + (rotation.sin() * diff_x) + (rotation.cos() * diff_y),
            ]
        }

        let rotation = cmd.rotation;
        let color = cmd.color.to_percentage_slice();
        let vertex_set = [
            // Changed
            Vertex {
                position: rotate_vertex(
                    center_x,
                    center_y,
                    vertex_rect.x,
                    vertex_rect.y + vertex_rect.height,
                    rotation,
                ),
                tex_coords: [target.x, target.y],
                color,
            }, // A
            Vertex {
                position: rotate_vertex(center_x, center_y, vertex_rect.x, vertex_rect.y, rotation),
                tex_coords: [target.x, target.y + target.height],
                color,
            }, // B
            Vertex {
                position: rotate_vertex(
                    center_x,
                    center_y,
                    vertex_rect.x + vertex_rect.width,
                    vertex_rect.y,
                    rotation,
                ),
                tex_coords: [target.x + target.width, target.y + target.height],
                color,
            }, // C
            Vertex {
                position: rotate_vertex(
                    center_x,
                    center_y,
                    vertex_rect.x + vertex_rect.width,
                    vertex_rect.y + vertex_rect.height,
                    rotation,
                ),
                tex_coords: [target.x + target.width, target.y],
                color,
            },
        ];

        if cmd.frustrum_culling {
            // Use vertex set bounding box for frustrum culling
            if !Rectangle::new(-1.0, -1.0, 2.0, 2.0).intersects_with(&vertex_rect) {
                return Ok(());
            }
        }

        let len = self.draw_queue.len();
        let same_texture = len > 0
            && self
                .draw_queue
                .front()
                .filter(|draw| {
                    draw.texture_asset_id == texture_asset_id
                        && draw.indices_per_draw == TEXTURED_QUAD_INDICES_PER_DRAW // check that the previous draw is also for quads
                        && draw.vertices_per_draw == TEXTURED_QUAD_VERTICES_PER_DRAW
                })
                .is_some();

        let mut index_start: u32 = 0;
        if same_texture {
            if let Some(draw) = self.draw_queue.front_mut() {
                index_start = draw.index_start();
                draw.add(
                    TEXTURED_QUAD_VERTEX_SET_SIZE,
                    TEXTURED_QUAD_INDICES_SET_SIZE,
                );
            }
        }

        let indices_set = [
            index_start,
            index_start + 1,
            index_start + 2,
            index_start,
            index_start + 2,
            index_start + 3,
        ];

        self.vertices.extend(vertex_set);
        self.indices.extend(indices_set);

        if !same_texture {
            let mut indices_start = 0;
            let mut vertices_start = 0;
            if let Some(draw) = self.draw_queue.front() {
                indices_start = draw.indices_range.end;
                vertices_start = draw.vertices_range.end;
            }

            self.draw_queue.push_front(TexturedTriDraw::new(
                texture_asset_id,
                texture_bind_group_asset_id,
                vertices_start,
                TEXTURED_QUAD_VERTEX_SET_SIZE,
                TEXTURED_QUAD_VERTICES_PER_DRAW,
                indices_start,
                TEXTURED_QUAD_INDICES_SET_SIZE,
                TEXTURED_QUAD_INDICES_PER_DRAW,
            ));
        }

        Ok(())
    }

    fn draw_textured_tri(
        &mut self,
        _command: emerald::rendering_engine::DrawTexturedTriCommand,
    ) -> Result<(), emerald::EmeraldError> {
        todo!()
    }

    fn screen_size(&self) -> ScreenSize {
        ScreenSize {
            width: self.size.width,
            height: self.size.height,
        }
    }

    fn resize_window(&mut self, new_size: ScreenSize) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = PhysicalSize::new(new_size.width, new_size.height);
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            // future todo: resize any depth textures here
        }
    }

    fn get_texture_key(&self, asset_engine: &mut AssetEngine, label: &str) -> Option<AssetKey> {
        asset_engine.get_asset_key_by_label::<Texture>(label)
    }

    fn begin(&mut self, _asset_store: &mut AssetEngine) -> Result<(), EmeraldError> {
        if self.active_render_texture_asset_id.is_some() {
            return Err(EmeraldError::new("Cannot begin render. There is an active render_texture. Please finish rendering to your texture before beginning the final render pass."));
        }

        self.vertices.clear();
        self.indices.clear();
        self.active_size = self.size;

        Ok(())
    }

    fn begin_texture(
        &mut self,
        texture_key: &AssetKey,
        asset_engine: &mut AssetEngine,
    ) -> Result<(), EmeraldError> {
        if self.active_render_texture_asset_id.is_some() {
            return Err(EmeraldError::new("Unable to begin_texture, a render texture is already active. Please complete your render pass on the texture before beginning another."));
        }
        self.vertices.clear();
        self.indices.clear();

        if let Some(texture) = asset_engine.get_asset::<Texture>(&texture_key.asset_id()) {
            self.active_size = PhysicalSize::new(texture.size.width, texture.size.height);
        } else {
            return Err(EmeraldError::new(format!(
                "Cannot begin rendering to texture. Texture {:?} does not exist.",
                texture_key
            )));
        }
        self.active_render_texture_asset_id = Some(texture_key.asset_id());

        Ok(())
    }

    fn render_texture(&mut self, asset_store: &mut AssetEngine) -> Result<(), EmeraldError> {
        match self.active_render_texture_asset_id.take() {
            None => {
                return Err(EmeraldError::new(
                "Unable to render_texture, there is no active render texture. Please user begin_texture to set the active render texture.",
            ));
            }
            Some(id) => {
                if let Some(texture) = asset_store.get_asset::<Texture>(&id) {
                    let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
                        format: Some(self.config.format),
                        ..Default::default()
                    });

                    self.render_to_view(asset_store, view, &format!("render texture {:?}", id))?;

                    return Ok(());
                }

                Err(EmeraldError::new(format!(
                    "Unable to find texture {:?}",
                    id
                )))
            }
        }
    }

    fn load_texture(
        &mut self,
        label: &str,
        asset_store: &mut AssetEngine,
        data: &[u8],
    ) -> Result<AssetKey, EmeraldError> {
        Texture::from_bytes(
            label,
            &self.bind_group_layouts,
            asset_store,
            &self.device,
            &self.queue,
            &data,
        )
    }

    fn load_texture_ext(
        &mut self,
        label: &str,
        asset_store: &mut AssetEngine,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Result<AssetKey, EmeraldError> {
        Texture::new(
            label,
            &self.bind_group_layouts,
            asset_store,
            &self.device,
            &self.queue,
            width,
            height,
            &data,
        )
    }

    fn render(&mut self, asset_store: &mut AssetEngine) -> Result<(), EmeraldError> {
        let surface_texture = match self.surface.get_current_texture() {
            Ok(surface_texture) => Ok(surface_texture),
            Err(e) => {
                match e {
                    wgpu::SurfaceError::Lost => self.resize_window(ScreenSize {
                        width: self.size.width,
                        height: self.size.height,
                    }),
                    // outdated surface texture, no point rendering to it, just skip
                    wgpu::SurfaceError::Outdated => return Ok(()),
                    _ => {}
                };
                Err(EmeraldError::new(format!("{:?}", e)))
            }
        }?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_to_view(asset_store, view, "Surface Pass")?;

        surface_texture.present();
        Ok(())
    }

    fn create_render_texture(
        &mut self,
        width: u32,
        height: u32,
        asset_store: &mut AssetEngine,
    ) -> Result<AssetKey, EmeraldError> {
        let data = (0..(width * height * 4))
            .into_iter()
            .map(|_| 0)
            .collect::<Vec<u8>>();
        let label = format!("emd_rt_{}", self.render_texture_uid);
        let key = Texture::new_render_target(
            &label,
            &self.bind_group_layouts,
            asset_store,
            &self.device,
            &self.queue,
            width,
            height,
            &data,
            self.config.format,
        )?;
        self.render_texture_uid += 1;

        Ok(key)
    }

    fn handle_window_resize(&mut self, screen_size: ScreenSize) {
        if self.size.width != screen_size.width || self.size.height != screen_size.height {
            self.resize_window(screen_size)
        }
    }

    fn current_render_target_size(&self) -> ScreenSize {
        ScreenSize {
            width: self.active_size.width,
            height: self.active_size.height,
        }
    }

    fn layout(&self) -> &Layout {
        &self.layout
    }

    fn layout_mut(&mut self) -> &mut Layout {
        &mut self.layout
    }

    fn draw_color_rect(
        &mut self,
        asset_engine: &mut AssetEngine,
        color_rect: &emerald::ColorRect,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        if !color_rect.visible {
            return Ok(());
        }
        // let texture_asset_id = self.get_default_texture_asset_id()?;
        // self.draw_textured_tri(DrawTexturedQuadCommand {
        //     texture_target_area: todo!(),
        //     asset_engine,
        //     texture_asset_id: todo!(),
        //     offset: todo!(),
        //     scale: todo!(),
        //     rotation: todo!(),
        //     centered: todo!(),
        //     color: todo!(),
        //     transform,
        //     current_render_target_size: todo!(),
        //     pixel_snap: todo!(),
        //     frustrum_culling: todo!(),
        // })
        Ok(())
    }

    fn draw_color_tri(
        &mut self,
        asset_engine: &mut AssetEngine,
        color_tri: &emerald::ColorTri,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        if !color_tri.visible {
            return Ok(());
        }
        // self.draw_textured_tri(DrawTexturedTriCommand {
        //     texture_target_area: Rectangle::zeroed(),
        //     asset_engine,
        //     texture_asset_id: texture_asset_id,
        //     offset: Vector2::new(0.0, 0.0),
        //     scale: Vector2::new(1.0, 1.0),
        //     rotation: 0.0,
        //     centered: true,
        //     color: color_tri.color,
        //     transform,
        //     current_render_target_size: self.current_render_target_size(),
        // })
        Ok(())
    }
}

const VERTEX_SIZE: u64 = std::mem::size_of::<Vertex>() as u64;
const INDEX_SIZE: u64 = std::mem::size_of::<u32>() as u64;

const TEXTURED_QUAD_VERTICES_PER_DRAW: usize = 4;
const TEXTURED_QUAD_INDICES_PER_DRAW: usize = 6;
const TEXTURED_QUAD_VERTEX_SET_SIZE: u64 = VERTEX_SIZE * TEXTURED_QUAD_VERTICES_PER_DRAW as u64; // 4 vertices, 1 for each corner of the quad
const TEXTURED_QUAD_INDICES_SET_SIZE: u64 = INDEX_SIZE * TEXTURED_QUAD_INDICES_PER_DRAW as u64; // 6 indices to draw a quad using 4 vertices

fn draw_textured_quad(
    asset_store: &mut AssetEngine,
    texture_asset_id: AssetId,
    texture_bind_group_asset_id: AssetId,
    mut target: Rectangle,
    offset: Vector2<f32>,
    scale: Vector2<f32>,
    rotation: f32,
    centered: bool,
    color: Color,
    transform: &Transform,
    active_size: PhysicalSize<u32>,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    textured_tri_draws: &mut VecDeque<TexturedTriDraw>,
    settings: &RenderSettings,
) -> Result<(), EmeraldError> {
    let texture_size;
    if let Some(texture) = asset_store.get_asset::<Texture>(&texture_asset_id) {
        texture_size = (texture.size.width as f32, texture.size.height as f32);

        // Zeroed target means display entire texture
        if target.is_zero_sized() {
            target.width = texture_size.0;
            target.height = texture_size.1;
        }
    } else {
        return Err(EmeraldError::new(format!(
            "Unable to find Texture for AssetId {:?}",
            texture_asset_id
        )));
    }

    // Add magic numbers to target semi-middle of pixels
    target.x += 0.275;
    target.y += 0.275;

    let mut x = transform.translation.x + offset.x;
    let mut y = transform.translation.y + offset.y;

    if settings.pixel_snap {
        x = x.floor();
        y = y.floor();
    }

    let x = x / (active_size.width as f32 / 2.0);
    let y = y / (active_size.height as f32 / 2.0);

    let normalized_texture_size = (
        target.width / (active_size.width as f32 / 2.0),
        target.height / (active_size.height as f32 / 2.0),
    );

    {
        let x = target.x / texture_size.0;
        let y = target.y / texture_size.1;
        let width = target.width / texture_size.0;
        let height = target.height / texture_size.1;
        target = Rectangle::new(x, y, width, height);
    }

    let width = normalized_texture_size.0 * scale.x;
    let height = normalized_texture_size.1 * scale.y;
    let mut vertex_rect = Rectangle::new(x, y, width, height);

    if centered {
        vertex_rect.x -= width / 2.0;
        vertex_rect.y -= height / 2.0;
    }
    let center_x = vertex_rect.x + vertex_rect.width / 2.0;
    let center_y = vertex_rect.y + vertex_rect.height / 2.0;

    fn rotate_vertex(center_x: f32, center_y: f32, x: f32, y: f32, rotation: f32) -> [f32; 2] {
        let diff_x = x - center_x;
        let diff_y = y - center_y;
        [
            center_x + (rotation.cos() * diff_x) - (rotation.sin() * diff_y),
            center_y + (rotation.sin() * diff_x) + (rotation.cos() * diff_y),
        ]
    }

    let color = color.to_percentage_slice();
    let vertex_set = [
        // Changed
        Vertex {
            position: rotate_vertex(
                center_x,
                center_y,
                vertex_rect.x,
                vertex_rect.y + vertex_rect.height,
                rotation,
            ),
            tex_coords: [target.x, target.y],
            color,
        }, // A
        Vertex {
            position: rotate_vertex(center_x, center_y, vertex_rect.x, vertex_rect.y, rotation),
            tex_coords: [target.x, target.y + target.height],
            color,
        }, // B
        Vertex {
            position: rotate_vertex(
                center_x,
                center_y,
                vertex_rect.x + vertex_rect.width,
                vertex_rect.y,
                rotation,
            ),
            tex_coords: [target.x + target.width, target.y + target.height],
            color,
        }, // C
        Vertex {
            position: rotate_vertex(
                center_x,
                center_y,
                vertex_rect.x + vertex_rect.width,
                vertex_rect.y + vertex_rect.height,
                rotation,
            ),
            tex_coords: [target.x + target.width, target.y],
            color,
        },
    ];

    if settings.frustrum_culling {
        // Use vertex set bounding box for frustrum culling
        if !Rectangle::new(-1.0, -1.0, 2.0, 2.0).intersects_with(&vertex_rect) {
            return Ok(());
        }
    }

    let len = textured_tri_draws.len();
    let same_texture = len > 0
        && textured_tri_draws
            .front()
            .filter(|draw| {
                draw.texture_asset_id == texture_asset_id
                    && draw.indices_per_draw == TEXTURED_QUAD_INDICES_PER_DRAW // check that the previous draw is also for quads
                    && draw.vertices_per_draw == TEXTURED_QUAD_VERTICES_PER_DRAW
            })
            .is_some();

    let mut index_start: u32 = 0;
    if same_texture {
        if let Some(draw) = textured_tri_draws.front_mut() {
            index_start = draw.index_start();
            draw.add(
                TEXTURED_QUAD_VERTEX_SET_SIZE,
                TEXTURED_QUAD_INDICES_SET_SIZE,
            );
        }
    }

    let indices_set = [
        index_start,
        index_start + 1,
        index_start + 2,
        index_start,
        index_start + 2,
        index_start + 3,
    ];

    vertices.extend(vertex_set);
    indices.extend(indices_set);

    if !same_texture {
        let mut indices_start = 0;
        let mut vertices_start = 0;
        if let Some(draw) = textured_tri_draws.front() {
            indices_start = draw.indices_range.end;
            vertices_start = draw.vertices_range.end;
        }

        textured_tri_draws.push_front(TexturedTriDraw::new(
            texture_asset_id,
            texture_bind_group_asset_id,
            vertices_start,
            TEXTURED_QUAD_VERTEX_SET_SIZE,
            TEXTURED_QUAD_VERTICES_PER_DRAW,
            indices_start,
            TEXTURED_QUAD_INDICES_SET_SIZE,
            TEXTURED_QUAD_INDICES_PER_DRAW,
        ));
    }

    Ok(())
}

const TEXTURED_TRI_VERTICES_PER_DRAW: usize = 3;
const TEXTURED_TRI_INDICES_PER_DRAW: usize = 3;
/// 1 vertex per triangle point
const TEXTURED_TRI_VERTEX_SET_SIZE: u64 = VERTEX_SIZE * TEXTURED_TRI_VERTICES_PER_DRAW as u64;
/// 1 index per triangle vertex
const TEXTURED_TRI_INDICES_SET_SIZE: u64 = INDEX_SIZE * TEXTURED_TRI_INDICES_PER_DRAW as u64;
fn draw_textured_tri(
    asset_store: &mut AssetEngine,
    texture_asset_id: AssetId,
    texture_bind_group_asset_id: AssetId,
    mut points: [Vector2<f32>; 3],
    mut target: [Vector2<f32>; 3],
    color: Color,
    transform: &Transform,
    active_size: PhysicalSize<u32>,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    textured_tri_draws: &mut VecDeque<TexturedTriDraw>,
    settings: &RenderSettings,
) -> Result<(), EmeraldError> {
    let texture_size;
    if let Some(texture) = asset_store.get_asset::<Texture>(&texture_asset_id) {
        texture_size = (texture.size.width as f32, texture.size.height as f32);
    } else {
        return Err(EmeraldError::new(format!(
            "Unable to find Texture for AssetId {:?}",
            texture_asset_id
        )));
    }

    for target_point in &mut target {
        // Add magic numbers to target semi-middle of pixels
        target_point.x += 0.275;
        target_point.y += 0.275;
    }

    let color = color.to_percentage_slice();

    for point in &mut points {
        point.x += transform.translation.x;
        point.y += transform.translation.y;

        if settings.pixel_snap {
            point.x = point.x.floor();
            point.y = point.y.floor();
        }

        point.x = point.x / (active_size.width as f32 / 2.0);
        point.y = point.y / (active_size.height as f32 / 2.0);
    }

    if settings.frustrum_culling {
        // Use vertex set bounding box for frustrum culling
        if !Rectangle::new(-1.0, -1.0, 2.0, 2.0)
            .intersects_with(&get_bounding_box_of_triangle(&points))
        {
            return Ok(());
        }
    }

    let len = textured_tri_draws.len();
    let same_texture = len > 0
        && textured_tri_draws
            .front()
            .filter(|draw| {
                draw.texture_asset_id == texture_asset_id
                    && draw.indices_per_draw == TEXTURED_TRI_INDICES_PER_DRAW
                    && draw.vertices_per_draw == TEXTURED_TRI_VERTICES_PER_DRAW
            })
            .is_some();

    let mut index_start: u32 = 0;
    if same_texture {
        if let Some(draw) = textured_tri_draws.front_mut() {
            index_start = draw.index_start();
            draw.add(TEXTURED_TRI_VERTEX_SET_SIZE, TEXTURED_TRI_INDICES_SET_SIZE);
        }
    }

    let indices_set = [index_start, index_start + 1, index_start + 2];
    let vertex_set = [
        Vertex {
            position: [points[0].x, points[0].y],
            tex_coords: [target[0].x, target[0].y],
            color,
        },
        Vertex {
            position: [points[1].x, points[1].y],
            tex_coords: [target[1].x, target[1].y],
            color,
        },
        Vertex {
            position: [points[2].x, points[2].y],
            tex_coords: [target[2].x, target[2].y],
            color,
        },
    ];

    vertices.extend(vertex_set);
    indices.extend(indices_set);

    if !same_texture {
        let mut indices_start = 0;
        let mut vertices_start = 0;
        if let Some(draw) = textured_tri_draws.front() {
            indices_start = draw.indices_range.end;
            vertices_start = draw.vertices_range.end;
        }

        textured_tri_draws.push_front(TexturedTriDraw::new(
            texture_asset_id,
            texture_bind_group_asset_id,
            vertices_start,
            TEXTURED_TRI_VERTEX_SET_SIZE,
            TEXTURED_TRI_VERTICES_PER_DRAW,
            indices_start,
            TEXTURED_TRI_INDICES_SET_SIZE,
            TEXTURED_TRI_INDICES_PER_DRAW,
        ));
    }

    Ok(())
}

async fn get_adapter(
    instance: &wgpu::Instance,
    surface: &Surface,
) -> Result<Adapter, EmeraldError> {
    let adapter_result = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await;

    match adapter_result {
        None => Err(EmeraldError::new("No graphics adapters found.")),
        Some(adapter) => Ok(adapter),
    }
}

async fn get_device_and_queue(adapter: &Adapter) -> Result<(Device, Queue), EmeraldError> {
    let result = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),

                #[cfg(target_arch = "wasm32")]
                limits: wgpu::Limits::downlevel_webgl2_defaults(),
                #[cfg(not(target_arch = "wasm32"))]
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        )
        .await;

    match result {
        Err(e) => Err(EmeraldError::new(format!(
            "Unable to get device and queue. {:?}",
            e
        ))),
        Ok(val) => Ok(val),
    }
}
