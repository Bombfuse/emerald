use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    ops::Range,
};

use fontdue::layout::{Layout, LayoutSettings, TextStyle};
use hecs::Entity;
use rapier2d::na::{Quaternion, Vector2, Vector4};
use wgpu::{util::DeviceExt, Buffer, RenderPass, TextureFormat, TextureView};
use wgpu::{Adapter, BindGroup, BindGroupLayout, Device, Queue, RenderPipeline, Surface};
use winit::dpi::PhysicalSize;

use crate::{
    autotilemap::AutoTilemap,
    font::FontKey,
    render_settings::RenderSettings,
    shaders::{
        self,
        textured_quad::{Camera2D, CameraUniform, Vertex, INDICES, VERTICES},
    },
    texture::{Texture, TextureKey},
    tilemap::Tilemap,
    AssetStore, Color, EmeraldError, Rectangle, Scale, Transform, Translation, UIButton, World,
};

use super::components::{Camera, ColorRect, Label, Sprite};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BindGroupLayoutId {
    TextureQuad,
    Camera2D,
}

type BindGroupId = String;

const BIND_GROUP_ID_CAMERA_2D: &str = "emd_camera_2d";

/// A set of textured tris that will be drawn.
struct TexturedTriDraw {
    pub key: TextureKey,
    pub vertices_range: Range<u64>,
    pub indices_range: Range<u64>,
    pub count: u32,
}

pub(crate) type BindGroups = HashMap<String, BindGroup>;
pub(crate) type BindGroupLayouts = HashMap<BindGroupLayoutId, BindGroupLayout>;

pub(crate) struct RenderingEngine {
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub settings: RenderSettings,

    pub texture_quad_render_pipeline: RenderPipeline,
    pub bind_group_layouts: BindGroupLayouts,
    pub bind_groups: BindGroups,
    pub draw_queue: VecDeque<DrawCommand>,

    pub index_buffer: wgpu::Buffer,
    pub vertex_buffer: wgpu::Buffer,

    vertices: Vec<Vertex>,
    indices: Vec<u32>,

    pub render_texture_uid: usize,

    color_rect_texture: TextureKey,

    pub active_render_texture_key: Option<TextureKey>,
    pub active_size: winit::dpi::PhysicalSize<u32>,

    layout: Layout,
}
impl RenderingEngine {
    pub async fn new(
        window: &winit::window::Window,
        settings: RenderSettings,
        asset_store: &mut AssetStore,
    ) -> Result<Self, EmeraldError> {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = get_adapter(&instance, &surface).await?;
        let (device, queue) = get_device_and_queue(&adapter).await?;
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
        };

        let mut bind_groups = HashMap::new();
        let mut bind_group_layouts = HashMap::new();
        let draw_queue = VecDeque::new();

        let camera_uniform = CameraUniform::new(config.width as f32, config.height as f32);
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

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
                bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
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
        bind_group_layouts.insert(BindGroupLayoutId::Camera2D, camera_bind_group_layout);

        bind_groups.insert(BIND_GROUP_ID_CAMERA_2D.into(), camera_bind_group);

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
            &mut bind_groups,
            &bind_group_layouts,
            asset_store,
            &device,
            &queue,
            1,
            1,
            &[0, 0, 0, 255],
            TextureKey::default(),
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
            bind_groups,
            draw_queue,

            vertex_buffer,
            index_buffer,
            vertices: Vec::new(),
            indices: Vec::new(),

            color_rect_texture,

            render_texture_uid: 0,

            active_render_texture_key: None,
            layout: Layout::new(fontdue::layout::CoordinateSystem::PositiveYUp),
        })
    }

    pub fn resize_window(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            // future todo: resize any depth textures here
        }
    }

    #[inline]
    pub fn draw_world(
        &mut self,
        world: &mut World,
        asset_store: &mut AssetStore,
    ) -> Result<(), EmeraldError> {
        let (camera, camera_transform) = get_camera_and_camera_transform(world);

        let cmd_adder = DrawCommandAdder::new(self, world);
        let mut draw_queue = Vec::new();

        #[cfg(feature = "aseprite")]
        cmd_adder.add_draw_commands::<Aseprite>(&mut draw_queue, world, asset_store);

        cmd_adder.add_draw_commands::<AutoTilemap>(&mut draw_queue, world, asset_store);
        cmd_adder.add_draw_commands::<Tilemap>(&mut draw_queue, world, asset_store);
        cmd_adder.add_draw_commands::<Sprite>(&mut draw_queue, world, asset_store);
        cmd_adder.add_draw_commands::<UIButton>(&mut draw_queue, world, asset_store);
        cmd_adder.add_draw_commands::<ColorRect>(&mut draw_queue, world, asset_store);
        cmd_adder.add_draw_commands::<Label>(&mut draw_queue, world, asset_store);
        draw_queue.sort_by(|a, b| a.z_index.partial_cmp(&b.z_index).unwrap());
        for mut draw_command in draw_queue {
            let translation = {
                let mut translation =
                    draw_command.transform.translation - camera_transform.translation;

                translation += Translation::from(camera.offset);
                translation
            };

            draw_command.transform.translation = translation;
            self.push_draw_command(asset_store, draw_command)?;
        }

        Ok(())
    }

    pub fn draw_colliders(&mut self, world: &World, color: Color) -> Result<(), EmeraldError> {
        Ok(())
    }

    #[inline]
    pub fn push_draw_command(
        &mut self,
        asset_store: &mut AssetStore,
        draw_command: DrawCommand,
    ) -> Result<(), EmeraldError> {
        match &draw_command.drawable {
            Drawable::Label { label } => {
                // prepass for caching glyphs, ideally we can do this inline
                {
                    self.layout.reset(&Default::default());

                    if let Some(font) = asset_store.get_fontdue_font(&label.font_key) {
                        self.layout.append(
                            &[font],
                            &TextStyle::new(&label.text, label.font_size as f32, 0),
                        );
                    }

                    let mut to_cache = Vec::new();
                    for glyph in self.layout.glyphs() {
                        if let Some(font) = asset_store.get_font(&label.font_key) {
                            if !font.characters.contains_key(&glyph.key) {
                                to_cache.push(glyph.key);
                            }
                        }
                    }

                    for glyph_key in to_cache {
                        crate::font::cache_glyph(
                            self,
                            asset_store,
                            &label.font_key,
                            glyph_key,
                            label.font_size,
                        )?;
                    }
                }
            }
            _ => {}
        }

        self.draw_queue.push_front(draw_command);
        Ok(())
    }

    pub fn begin(&mut self, asset_store: &mut AssetStore) -> Result<(), EmeraldError> {
        if self.active_render_texture_key.is_some() {
            return Err(EmeraldError::new("Cannot begin render. There is an active render_texture. Please finish rendering to your texture before beginning the final render pass."));
        }

        Ok(())
    }

    pub fn begin_texture(
        &mut self,
        texture_key: TextureKey,
        asset_store: &mut AssetStore,
    ) -> Result<(), EmeraldError> {
        if self.active_render_texture_key.is_some() {
            return Err(EmeraldError::new("Unable to begin_texture, a render texture is already active. Please complete your render pass on the texture before beginning another."));
        }

        // TODO: Begin rendering the given texture.
        self.active_render_texture_key = Some(texture_key);

        Ok(())
    }

    pub fn render_texture(
        &mut self,
        asset_store: &mut AssetStore,
    ) -> Result<TextureKey, EmeraldError> {
        match self.active_render_texture_key.take() {
            None => {
                return Err(EmeraldError::new(
                "Unable to render_texture, there is no active render texture. Please user begin_texture to set the active render texture.",
            ));
            }
            Some(texture_key) => {
                // TODO: consume draw queue and finish render pass
                if let Some(texture) = asset_store.get_texture(&texture_key) {
                    let view = texture.texture.create_view(&wgpu::TextureViewDescriptor {
                        format: Some(self.config.format),
                        ..Default::default()
                    });

                    self.render_to_view(
                        asset_store,
                        view,
                        PhysicalSize::new(texture.size.width, texture.size.height),
                        &format!("render texture {:?}", texture_key),
                    )?;
                    return Ok(texture_key);
                }

                Err(EmeraldError::new(format!(
                    "Unable to find texture {:?}",
                    texture_key
                )))
            }
        }
    }

    pub fn load_texture(
        &mut self,
        asset_store: &mut AssetStore,
        data: &[u8],
        key: TextureKey,
    ) -> Result<TextureKey, EmeraldError> {
        if asset_store.get_texture(&key).is_some() {
            return Ok(key);
        }

        Texture::from_bytes(
            &mut self.bind_groups,
            &self.bind_group_layouts,
            asset_store,
            &self.device,
            &self.queue,
            &data,
            key.clone(),
        )
    }

    pub fn load_texture_ext(
        &mut self,
        asset_store: &mut AssetStore,
        width: u32,
        height: u32,
        data: &[u8],
        key: TextureKey,
    ) -> Result<TextureKey, EmeraldError> {
        if asset_store.get_texture(&key).is_some() {
            return Ok(key);
        }

        Texture::new(
            &mut self.bind_groups,
            &self.bind_group_layouts,
            asset_store,
            &self.device,
            &self.queue,
            width,
            height,
            &data,
            key.clone(),
        )
    }

    fn render_to_view(
        &mut self,
        asset_store: &mut AssetStore,
        view: TextureView,
        view_size: PhysicalSize<u32>,
        view_name: &str,
    ) -> Result<(), EmeraldError> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some(&format!("Encoder: {:?}", view_name)),
            });

        self.active_size = view_size;
        {
            let (r, g, b, a) = self.settings.background_color.to_percentage();

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

    pub fn render(&mut self, asset_store: &mut AssetStore) -> Result<(), EmeraldError> {
        let surface_texture = match self.surface.get_current_texture() {
            Ok(surface_texture) => Ok(surface_texture),
            Err(e) => {
                match e {
                    wgpu::SurfaceError::Lost => self.resize_window(self.size),
                    _ => {}
                };
                Err(EmeraldError::new(format!("{:?}", e)))
            }
        }?;
        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_to_view(asset_store, view, self.size, "Surface Pass")?;

        surface_texture.present();
        Ok(())
    }

    pub fn create_render_texture(
        &mut self,
        width: u32,
        height: u32,
        asset_store: &mut AssetStore,
    ) -> Result<TextureKey, EmeraldError> {
        let data = (0..(width * height * 4))
            .into_iter()
            .map(|_| 0)
            .collect::<Vec<u8>>();
        let key = Texture::new_render_target(
            &mut self.bind_groups,
            &self.bind_group_layouts,
            asset_store,
            &self.device,
            &self.queue,
            width,
            height,
            &data,
            TextureKey::new(format!("emd_render_texture_{}", self.render_texture_uid)),
            self.config.format,
        )?;
        self.render_texture_uid += 1;

        Ok(key)
    }

    #[inline]
    fn consume_draw_queue<'a>(
        &'a mut self,
        render_pass: &mut wgpu::RenderPass<'a>,
        asset_store: &mut AssetStore,
    ) -> Result<(), EmeraldError> {
        render_pass.set_pipeline(&self.texture_quad_render_pipeline);

        // Calculate vertices for every texture to be drawn, paired with their sprite data and vertex indices
        // for every tuple, draw that sprites texture bind group using that vertex index as the slice
        let mut counter: u32 = 0;
        let vertex_set_size = (std::mem::size_of::<Vertex>() * 4) as u64;
        let indices_set_size: u64 = std::mem::size_of::<u32>() as u64 * 6;

        let mut textured_quads: Vec<TexturedTriDraw> = Vec::new();
        self.vertices.clear();
        self.indices.clear();
        let mut prev_texture_key = None;

        let draw_queue = &mut self.draw_queue;
        while let Some(draw_command) = draw_queue.pop_back() {
            match draw_command.drawable {
                Drawable::Sprite { sprite } => {
                    let texture_key = sprite.texture_key;
                    let mut target_rect = sprite.target;
                    let texture_size;
                    if let Some(texture) = asset_store.get_texture(&texture_key) {
                        texture_size = (texture.size.width as f32, texture.size.height as f32);

                        // Zeroed target means display entire texture
                        if target_rect.is_zero_sized() {
                            target_rect.width = texture_size.0;
                            target_rect.height = texture_size.1;
                        }
                    } else {
                        return Err(EmeraldError::new(format!(
                            "Unable to find texture {:?}",
                            texture_key
                        )));
                    }

                    let x = (draw_command.transform.translation.x + sprite.offset.x)
                        / (self.active_size.width as f32 / 2.0);
                    let y = (draw_command.transform.translation.y + sprite.offset.y)
                        / (self.active_size.height as f32 / 2.0);

                    let normalized_texture_size = (
                        target_rect.width / (self.active_size.width as f32 / 2.0),
                        target_rect.height / (self.active_size.height as f32 / 2.0),
                    );

                    {
                        let x = target_rect.x / texture_size.0;
                        let y = target_rect.y / texture_size.1;
                        let width = target_rect.width / texture_size.0;
                        let height = target_rect.height / texture_size.1;
                        target_rect = Rectangle::new(x, y, width, height);
                    }

                    let width = normalized_texture_size.0 * sprite.scale.x;
                    let height = normalized_texture_size.1 * sprite.scale.y;
                    let mut vertex_rect = Rectangle::new(x, y, width, height);

                    if sprite.centered {
                        vertex_rect.x -= width / 2.0;
                        vertex_rect.y -= height / 2.0;
                    }

                    let center_x = vertex_rect.x + vertex_rect.width / 2.0;
                    let center_y = vertex_rect.y + vertex_rect.height / 2.0;

                    fn rotate_vertex(
                        center_x: f32,
                        center_y: f32,
                        x: f32,
                        y: f32,
                        rotation: f32,
                    ) -> [f32; 2] {
                        let diff_x = x - center_x;
                        let diff_y = y - center_y;
                        [
                            center_x + (rotation.cos() * diff_x) - (rotation.sin() * diff_y),
                            center_y + (rotation.sin() * diff_x) + (rotation.cos() * diff_y),
                        ]
                    }

                    let vertex_set = [
                        // Changed
                        Vertex {
                            position: rotate_vertex(
                                center_x,
                                center_y,
                                vertex_rect.x,
                                vertex_rect.y + vertex_rect.height,
                                sprite.rotation,
                            ),
                            tex_coords: [target_rect.x, target_rect.y],
                        }, // A
                        Vertex {
                            position: rotate_vertex(
                                center_x,
                                center_y,
                                vertex_rect.x,
                                vertex_rect.y,
                                sprite.rotation,
                            ),
                            tex_coords: [target_rect.x, target_rect.y + target_rect.height],
                        }, // B
                        Vertex {
                            position: rotate_vertex(
                                center_x,
                                center_y,
                                vertex_rect.x + vertex_rect.width,
                                vertex_rect.y,
                                sprite.rotation,
                            ),
                            tex_coords: [
                                target_rect.x + target_rect.width,
                                target_rect.y + target_rect.height,
                            ],
                        }, // C
                        Vertex {
                            position: rotate_vertex(
                                center_x,
                                center_y,
                                vertex_rect.x + vertex_rect.width,
                                vertex_rect.y + vertex_rect.height,
                                sprite.rotation,
                            ),
                            tex_coords: [target_rect.x + target_rect.width, target_rect.y],
                        },
                    ];

                    let mut same_texture = false;

                    let len = textured_quads.len();
                    if len > 0 {
                        if let Some(key) = prev_texture_key {
                            if key == texture_key {
                                same_texture = true;
                            }
                        }
                    }

                    let mut add_quad = true;
                    let mut index_start = 0;

                    if same_texture {
                        if let Some(textured_quad_draw) = textured_quads.get_mut(len - 1) {
                            index_start = textured_quad_draw.count * 4;
                            textured_quad_draw.vertices_range.end += vertex_set_size;
                            textured_quad_draw.indices_range.end += indices_set_size;
                            textured_quad_draw.count += 1;
                            add_quad = false;
                        }
                    }

                    let vertices_start = (counter as u64) * vertex_set_size;
                    self.vertices.extend(vertex_set);
                    self.indices.extend([
                        index_start,
                        index_start + 1,
                        index_start + 2,
                        index_start,
                        index_start + 2,
                        index_start + 3,
                    ]);

                    if add_quad {
                        let indices_start = (counter as u64) * indices_set_size;
                        textured_quads.push(TexturedTriDraw {
                            key: texture_key.clone(),
                            vertices_range: (vertices_start..vertices_start + vertex_set_size),
                            indices_range: (indices_start..indices_start + indices_set_size),
                            count: 1,
                        });
                    }

                    prev_texture_key = Some(texture_key);
                }
                Drawable::Aseprite { sprite, .. } => {
                    // Aseprites can be broken down into a sprite draw
                    draw_queue.push_back(DrawCommand {
                        drawable: Drawable::Sprite { sprite },
                        ..draw_command
                    });
                    continue;
                }
                Drawable::ColorRect { color_rect } => {}
                Drawable::Label { label } => {
                    self.layout.reset(&LayoutSettings {
                        max_width: label.max_width,
                        max_height: label.max_height,
                        wrap_style: label.wrap_style,
                        horizontal_align: label.horizontal_align,
                        vertical_align: label.vertical_align,
                        ..LayoutSettings::default()
                    });

                    if let Some(font) = asset_store.get_fontdue_font(&label.font_key) {
                        self.layout.append(
                            &[font],
                            &TextStyle::new(&label.text, label.font_size as f32, 0),
                        );
                    } else {
                        return Err(EmeraldError::new(format!(
                            "Font {:?} was not found in the asset store.",
                            &label.font_key
                        )));
                    }

                    let mut remaining_char_count = if label.visible_characters < 0 {
                        label.text.len() as i64
                    } else {
                        label.visible_characters
                    };

                    for glyph in self.layout.glyphs() {
                        let glyph_key = glyph.key;
                        let x = glyph.x;
                        let y = glyph.y;

                        if let Some(font) = asset_store.get_font_mut(&label.font_key) {
                            if !font.characters.contains_key(&glyph_key) {
                                return Err(EmeraldError::new(format!(
                                    "Font {:?} does not contain cached glyph {:?}",
                                    font.font_texture_key, glyph_key
                                )));
                            }

                            let font_data = &font.characters[&glyph_key];
                            let left_coord = (font_data.offset_x as f32 + x) * label.scale;
                            let top_coord = y * label.scale;

                            let target = Rectangle::new(
                                font_data.glyph_x as f32,
                                font_data.glyph_y as f32,
                                font_data.glyph_w as f32,
                                font_data.glyph_h as f32,
                            );

                            let mut transform = draw_command.transform;
                            transform.translation.x += label.offset.x + left_coord;
                            transform.translation.y += label.offset.y + top_coord;

                            let mut sprite = Sprite::from_texture(font.font_texture_key.clone());
                            sprite.target = target;
                            sprite.scale = Vector2::new(label.scale, label.scale);
                            sprite.offset = label.offset;
                            sprite.centered = false;

                            if label.centered {
                                if let Some(width) = &label.max_width {
                                    transform.translation.x -= width / 2.0;
                                }
                            }

                            if remaining_char_count < 0 || sprite.target.is_zero_sized() {
                                println!("{:?}", (glyph.parent, remaining_char_count));
                                continue;
                            }

                            let texture_key = font.font_texture_key.clone();
                            let mut target_rect = target;
                            let texture_size;
                            if let Some(texture) = asset_store.get_texture(&texture_key) {
                                texture_size =
                                    (texture.size.width as f32, texture.size.height as f32);

                                // Zeroed target means display entire texture
                                if target_rect.is_zero_sized() {
                                    target_rect.width = texture_size.0;
                                    target_rect.height = texture_size.1;
                                }
                            } else {
                                return Err(EmeraldError::new(format!(
                                    "Unable to find texture {:?}",
                                    texture_key
                                )));
                            }

                            let x = (transform.translation.x + label.offset.x)
                                / (self.active_size.width as f32 / 2.0);
                            let y = (transform.translation.y + label.offset.y)
                                / (self.active_size.height as f32 / 2.0);

                            let normalized_texture_size = (
                                target_rect.width / (self.active_size.width as f32 / 2.0),
                                target_rect.height / (self.active_size.height as f32 / 2.0),
                            );

                            {
                                let x = target_rect.x / texture_size.0;
                                let y = target_rect.y / texture_size.1;
                                let width = target_rect.width / texture_size.0;
                                let height = target_rect.height / texture_size.1;
                                target_rect = Rectangle::new(x, y, width, height);
                            }

                            let width = normalized_texture_size.0 * label.scale;
                            let height = normalized_texture_size.1 * label.scale;
                            let mut vertex_rect = Rectangle::new(x, y, width, height);

                            let center_x = vertex_rect.x + vertex_rect.width / 2.0;
                            let center_y = vertex_rect.y + vertex_rect.height / 2.0;

                            fn rotate_vertex(
                                center_x: f32,
                                center_y: f32,
                                x: f32,
                                y: f32,
                                rotation: f32,
                            ) -> [f32; 2] {
                                let diff_x = x - center_x;
                                let diff_y = y - center_y;
                                [
                                    center_x + (rotation.cos() * diff_x)
                                        - (rotation.sin() * diff_y),
                                    center_y
                                        + (rotation.sin() * diff_x)
                                        + (rotation.cos() * diff_y),
                                ]
                            }

                            let vertex_set = [
                                // Changed
                                Vertex {
                                    position: rotate_vertex(
                                        center_x,
                                        center_y,
                                        vertex_rect.x,
                                        vertex_rect.y + vertex_rect.height,
                                        0.0,
                                    ),
                                    tex_coords: [target_rect.x, target_rect.y],
                                }, // A
                                Vertex {
                                    position: rotate_vertex(
                                        center_x,
                                        center_y,
                                        vertex_rect.x,
                                        vertex_rect.y,
                                        0.0,
                                    ),
                                    tex_coords: [target_rect.x, target_rect.y + target_rect.height],
                                }, // B
                                Vertex {
                                    position: rotate_vertex(
                                        center_x,
                                        center_y,
                                        vertex_rect.x + vertex_rect.width,
                                        vertex_rect.y,
                                        0.0,
                                    ),
                                    tex_coords: [
                                        target_rect.x + target_rect.width,
                                        target_rect.y + target_rect.height,
                                    ],
                                }, // C
                                Vertex {
                                    position: rotate_vertex(
                                        center_x,
                                        center_y,
                                        vertex_rect.x + vertex_rect.width,
                                        vertex_rect.y + vertex_rect.height,
                                        0.0,
                                    ),
                                    tex_coords: [target_rect.x + target_rect.width, target_rect.y],
                                },
                            ];

                            let mut same_texture = false;

                            let len = textured_quads.len();
                            if len > 0 {
                                if let Some(key) = prev_texture_key {
                                    if key == texture_key {
                                        same_texture = true;
                                    }
                                }
                            }

                            let mut add_quad = true;
                            let mut index_start = 0;

                            if same_texture {
                                if let Some(textured_quad_draw) = textured_quads.get_mut(len - 1) {
                                    index_start = textured_quad_draw.count * 4;
                                    textured_quad_draw.vertices_range.end += vertex_set_size;
                                    textured_quad_draw.indices_range.end += indices_set_size;
                                    textured_quad_draw.count += 1;
                                    add_quad = false;
                                }
                            }

                            let vertices_start = (counter as u64) * vertex_set_size;
                            self.vertices.extend(vertex_set);
                            self.indices.extend([
                                index_start,
                                index_start + 1,
                                index_start + 2,
                                index_start,
                                index_start + 2,
                                index_start + 3,
                            ]);

                            if add_quad {
                                let indices_start = (counter as u64) * indices_set_size;
                                textured_quads.push(TexturedTriDraw {
                                    key: texture_key.clone(),
                                    vertices_range: (vertices_start
                                        ..vertices_start + vertex_set_size),
                                    indices_range: (indices_start
                                        ..indices_start + indices_set_size),
                                    count: 1,
                                });
                            }

                            prev_texture_key = Some(texture_key);
                            remaining_char_count -= 1;
                        } else {
                            return Err(EmeraldError::new(format!(
                                "Font not found: {:?}",
                                label.font_key
                            )));
                        }
                    }
                }
                Drawable::Tilemap {
                    texture_key,
                    tiles,
                    tile_size,
                    width,
                    height,
                    visible,
                    z_index,
                } => todo!(),
            }

            counter += 1;
        }
        let vertices_set_count = self.vertex_buffer.size() / vertex_set_size;
        let indices_set_count = self.index_buffer.size() / indices_set_size as u64;

        if self.indices.len() as u64 > indices_set_count {
            self.index_buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Index Buffer"),
                    contents: bytemuck::cast_slice(&self.indices),
                    usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
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
                        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    });
        } else {
            self.queue
                .write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));
        }

        if let Some(camera_bind_group) = self.bind_groups.get(BIND_GROUP_ID_CAMERA_2D) {
            render_pass.set_bind_group(1, camera_bind_group, &[]);

            for draw_call in textured_quads {
                if let Some(texture_bind_group) = self.bind_groups.get(&draw_call.key.0) {
                    render_pass.set_bind_group(0, texture_bind_group, &[]);

                    render_pass
                        .set_vertex_buffer(0, self.vertex_buffer.slice(draw_call.vertices_range));
                    render_pass.set_index_buffer(
                        self.index_buffer.slice(draw_call.indices_range),
                        wgpu::IndexFormat::Uint32,
                    );

                    render_pass.draw_indexed(0..(draw_call.count * 6), 0, 0..1);
                } else {
                    return Err(EmeraldError::new(format!(
                        "Unable to find texture bind group for {:?}",
                        &draw_call.key.0
                    )));
                }
            }

            return Ok(());
        }

        Err(EmeraldError::new("Could not find camera bind group."))
    }

    #[inline]
    pub fn draw_label(
        &mut self,
        asset_store: &mut AssetStore,
        label: &Label,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        self.push_draw_command(
            asset_store,
            DrawCommand {
                drawable: Drawable::Label {
                    label: label.clone(),
                },
                transform: *transform,
                z_index: label.z_index,
            },
        )
    }

    #[inline]
    pub fn update_font_texture(
        &mut self,
        asset_store: &mut AssetStore,
        key: &FontKey,
    ) -> Result<(), EmeraldError> {
        if let Some(font) = asset_store.get_font(key) {
            if let Some(texture) = asset_store.get_texture(&font.font_texture_key) {
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
                        bytes_per_row: std::num::NonZeroU32::new(4 * font.font_image.width as u32),
                        rows_per_image: std::num::NonZeroU32::new(font.font_image.height as u32),
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

// use crate::autotilemap::AutoTilemap;
// use crate::rendering::*;
// use crate::tilemap::Tilemap;
// use crate::transform::Transform;
// use crate::world::*;
// use crate::*;
// use crate::{rendering::components::*, transform::Translation};

// use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
// use glam::{vec2, Mat4, Vec2, Vec4};
// use miniquad::*;
// use std::collections::{HashMap, VecDeque};

// const EMERALD_TEXTURE_PIPELINE_NAME: &str = "emerald_default_texture_pipline";

// // The default "screen" pass.
// // Renders to a texture the size of the screen when rendering begins.
// const EMERALD_DEFAULT_RENDER_TARGET: &str = "emerald_default_render_target";

// pub(crate) struct RenderingEngine {
//     pub(crate) settings: RenderSettings,
//     pipelines: HashMap<String, Pipeline>,
//     layout: Layout,
//     render_texture_counter: usize,
//     last_screen_size: (usize, usize),
//     screen_texture_key: TextureKey,
//     render_passes: HashMap<TextureKey, RenderPass>,
//     current_render_texture_key: TextureKey,
//     current_resolution: (usize, usize),

//     draw_queue: VecDeque<DrawCommand>,
// }
// impl RenderingEngine {
//     pub(crate) fn new(
//         ctx: &mut Context,
//         settings: RenderSettings,
//         asset_store: &mut AssetStore,
//     ) -> Self {
//         let mut pipelines = HashMap::new();

//         let shader = Shader::new(ctx, VERTEX, FRAGMENT, shaders::meta()).unwrap();
//         let params = PipelineParams {
//             depth_write: true,
//             color_blend: Some(BlendState::new(
//                 Equation::Add,
//                 BlendFactor::Value(BlendValue::SourceAlpha),
//                 BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
//             )),
//             alpha_blend: Some(BlendState::new(
//                 Equation::Add,
//                 BlendFactor::Zero,
//                 BlendFactor::One,
//             )),
//             ..Default::default()
//         };

//         let texture_pipeline = Pipeline::with_params(
//             ctx,
//             &[BufferLayout::default()],
//             &[VertexAttribute::new("position", VertexFormat::Float2)],
//             shader,
//             params,
//         );

//         pipelines.insert(EMERALD_TEXTURE_PIPELINE_NAME.to_string(), texture_pipeline);

//         let mut render_texture_counter = 0;
//         let key = TextureKey::new(String::from(EMERALD_DEFAULT_RENDER_TARGET));
//         let current_resolution = current_window_resolution(ctx);
//         let screen_texture_key = create_render_texture(
//             current_resolution.0,
//             current_resolution.1,
//             key,
//             ctx,
//             asset_store,
//         )
//         .unwrap();
//         render_texture_counter += 1;

//         let texture = asset_store.get_texture(&screen_texture_key).unwrap();
//         let current_render_texture_key = screen_texture_key.clone();
//         let mut render_passes = HashMap::new();
//         render_passes.insert(
//             screen_texture_key.clone(),
//             RenderPass::new(ctx, texture.inner, None),
//         );

//         RenderingEngine {
//             settings,
//             pipelines,
//             layout: Layout::new(CoordinateSystem::PositiveYDown),
//             render_texture_counter,
//             render_passes,
//             last_screen_size: current_resolution,
//             screen_texture_key,
//             current_render_texture_key,
//             current_resolution,
//             draw_queue: VecDeque::new(),
//         }
//     }

//     #[inline]
//     pub(crate) fn create_render_texture(
//         &mut self,
//         w: usize,
//         h: usize,
//         ctx: &mut Context,
//         asset_store: &mut AssetStore,
//     ) -> Result<TextureKey, EmeraldError> {
//         self.render_texture_counter += 1;
//         let key = TextureKey::new(format!(
//             "emd_render_texture_{}",
//             self.render_texture_counter
//         ));

//         create_render_texture(w, h, key, ctx, asset_store)
//     }

//     #[inline]
//     pub(crate) fn pre_draw(
//         &mut self,
//         ctx: &mut Context,
//         asset_store: &mut AssetStore,
//     ) -> Result<(), EmeraldError> {
//         let (w, h) = current_window_resolution(ctx);
//         let (prev_w, prev_h) = self.last_screen_size;

//         if w as usize != prev_w || h as usize != prev_h {
//             self.update_screen_texture_size(ctx, w as usize, h as usize, asset_store)?;
//         }

//         Ok(())
//     }

//     #[inline]
//     fn update_screen_texture_size(
//         &mut self,
//         ctx: &mut Context,
//         w: usize,
//         h: usize,
//         asset_store: &mut AssetStore,
//     ) -> Result<TextureKey, EmeraldError> {
//         let key = TextureKey::new(String::from(EMERALD_DEFAULT_RENDER_TARGET));

//         if let Some(render_pass) = self.render_passes.get_mut(&key) {
//             render_pass.delete(ctx);
//             self.render_passes.remove(&key);
//         }

//         let screen_texture_key =
//             create_render_texture(w as usize, h as usize, key, ctx, asset_store)?;

//         Ok(screen_texture_key)
//     }

//     #[inline]
//     pub(crate) fn post_draw(&mut self, ctx: &mut Context, _asset_store: &mut AssetStore) {
//         self.last_screen_size = current_window_resolution(ctx);
//     }

//     #[inline]
//     #[cfg(feature = "physics")]
//     pub fn draw_colliders(
//         &mut self,
//         world: &mut World,
//         collider_color: Color,
//     ) -> Result<(), EmeraldError> {
//         let screen_size = (
//             self.current_resolution.0 as f32,
//             self.current_resolution.1 as f32,
//         );
//         let mut color_rect = ColorRect {
//             color: collider_color,
//             ..Default::default()
//         };
//         color_rect.color = collider_color;
//         let (camera, camera_transform) = get_camera_and_camera_transform(world);

//         for (_id, body_handle) in world.inner.query::<&RigidBodyHandle>().iter() {
//             if let Some(body) = world.physics_engine.bodies.get(*body_handle) {
//                 for collider_handle in body.colliders() {
//                     if let Some(collider) = world.physics_engine.colliders.get(*collider_handle) {
//                         let aabb = collider.compute_aabb();
//                         let body_translation = Translation::from(Vec2::from(aabb.center().coords));
//                         color_rect.width = aabb.half_extents().x as u32 * 2;
//                         color_rect.height = aabb.half_extents().y as u32 * 2;

//                         let translation = {
//                             let mut translation = body_translation - camera_transform.translation;

//                             if camera.centered {
//                                 translation = translation
//                                     + Translation::new(screen_size.0 / 2.0, screen_size.1 / 2.0);
//                             }

//                             translation
//                         };

//                         self.push_draw_command(DrawCommand {
//                             drawable: Drawable::ColorRect { color_rect },
//                             transform: Transform::from_translation(translation),
//                             z_index: 0.0,
//                         })?;
//                     }
//                 }
//             }
//         }

//         Ok(())
//     }

//     #[inline]
//     pub(crate) fn begin(
//         &mut self,
//         ctx: &mut Context,
//         asset_store: &mut AssetStore,
//     ) -> Result<(), EmeraldError> {
//         self.current_render_texture_key = self.screen_texture_key.clone();

//         if let Some(texture) = asset_store.get_texture(&self.current_render_texture_key) {
//             self.current_resolution = (texture.width as usize, texture.height as usize);
//         } else {
//             return Err(EmeraldError::new(
//                 "Unable to retrieve default rendering texture",
//             ));
//         }

//         self.begin_texture_pass(ctx, asset_store, self.current_render_texture_key.clone())?;

//         Ok(())
//     }

//     #[inline]
//     pub(crate) fn begin_texture(
//         &mut self,
//         ctx: &mut Context,
//         texture_key: TextureKey,
//         asset_store: &mut AssetStore,
//     ) -> Result<(), EmeraldError> {
//         self.current_render_texture_key = texture_key.clone();

//         if let Some(texture) = asset_store.get_texture(&self.current_render_texture_key) {
//             self.current_resolution = (texture.width as usize, texture.height as usize);
//         } else {
//             return Err(EmeraldError::new(format!(
//                 "Unable to retrieve texture for {:?}",
//                 texture_key
//             )));
//         }

//         self.begin_texture_pass(ctx, asset_store, texture_key)?;

//         Ok(())
//     }

//     /// This will begin a rendering pass that will render to a WxH size texture
//     /// Call `render_to_texture` to retrieve the texture key for this pass.
//     #[inline]
//     fn begin_texture_pass(
//         &mut self,
//         ctx: &mut Context,
//         asset_store: &mut AssetStore,
//         texture_key: TextureKey,
//     ) -> Result<(), EmeraldError> {
//         if let Some(texture) = asset_store.get_texture(&texture_key) {
//             if !self.render_passes.contains_key(&texture_key) {
//                 self.render_passes.insert(
//                     texture_key.clone(),
//                     RenderPass::new(ctx, texture.inner, None),
//                 );
//             }
//         } else {
//             return Err(EmeraldError::new(format!(
//                 "Unable to retrieve texture for {:?}",
//                 texture_key
//             )));
//         }

//         if let Some(render_pass) = self.render_passes.get(&texture_key) {
//             ctx.begin_pass(
//                 *render_pass,
//                 PassAction::Clear {
//                     color: Some(self.settings.background_color.to_percentage()),
//                     depth: None,
//                     stencil: None,
//                 },
//             );

//             return Ok(());
//         }

//         Err(EmeraldError::new(format!(
//             "Unable to retrieve render pass for {:?}",
//             texture_key
//         )))
//     }

//     #[inline]
//     pub(crate) fn render(
//         &mut self,
//         ctx: &mut Context,
//         asset_store: &mut AssetStore,
//     ) -> Result<(), EmeraldError> {
//         self.consume_draw_queue(ctx, asset_store)?;

//         let texture_key = self.render_texture(ctx, asset_store)?;

//         ctx.begin_default_pass(PassAction::Clear {
//             color: Some(self.settings.background_color.to_percentage()),
//             depth: None,
//             stencil: None,
//         });
//         let sprite = Sprite::from_texture(texture_key);
//         let (w, h) = current_window_resolution(ctx);
//         let translation = Translation::new(w as f32 / 2.0, h as f32 / 2.0);

//         self.draw_sprite(ctx, asset_store, &sprite, &translation);
//         ctx.end_render_pass();

//         Ok(())
//     }

//     #[inline]
//     pub(crate) fn render_texture(
//         &mut self,
//         ctx: &mut Context,
//         asset_store: &mut AssetStore,
//     ) -> Result<TextureKey, EmeraldError> {
//         self.consume_draw_queue(ctx, asset_store)?;
//         ctx.end_render_pass();

//         Ok(self.current_render_texture_key.clone())
//     }

//     #[inline]
//     fn consume_draw_queue(
//         &mut self,
//         ctx: &mut Context,
//         asset_store: &mut AssetStore,
//     ) -> Result<(), EmeraldError> {
//         ctx.apply_pipeline(self.pipelines.get(EMERALD_TEXTURE_PIPELINE_NAME).unwrap());

//         while let Some(draw_command) = self.draw_queue.pop_back() {
//             let translation = draw_command.transform.translation;

//             match draw_command.drawable {
//                 Drawable::Tilemap {
//                     texture_key,
//                     tiles,
//                     tile_size,
//                     width,
//                     height,
//                     z_index,
//                     visible,
//                 } => self.draw_tilemap(
//                     ctx,
//                     asset_store,
//                     texture_key,
//                     tiles,
//                     tile_size,
//                     width,
//                     height,
//                     z_index,
//                     visible,
//                     &translation,
//                 ),

//                 Drawable::Aseprite {
//                     sprite,
//                     rotation,
//                     offset,
//                     centered,
//                     visible,
//                     scale,
//                     color,
//                     z_index,
//                 } => self.draw_aseprite(
//                     ctx,
//                     asset_store,
//                     &sprite,
//                     rotation,
//                     &offset,
//                     centered,
//                     visible,
//                     &scale,
//                     &color,
//                     z_index,
//                     &translation,
//                 ),
//                 Drawable::Sprite { sprite } => {
//                     self.draw_sprite(ctx, asset_store, &sprite, &translation)
//                 }
//                 Drawable::ColorRect { color_rect } => {
//                     self.draw_color_rect(ctx, asset_store, &color_rect, &translation)
//                 }
//                 Drawable::Label { label } => {
//                     self.draw_label(ctx, asset_store, &label, &translation)?
//                 }
//             }
//         }
//         Ok(())
//     }

//     pub(crate) fn draw_label(
//         &mut self,
//         mut ctx: &mut Context,
//         mut asset_store: &mut AssetStore,
//         label: &Label,
//         position: &Translation,
//     ) -> Result<(), EmeraldError>
//         self.layout.reset(&LayoutSettings {
//             max_width: label.max_width,
//             max_height: label.max_height,
//             wrap_style: label.wrap_style,
//             horizontal_align: label.horizontal_align,
//             vertical_align: label.vertical_align,
//             ..LayoutSettings::default()
//         });

//         if let Some(font) = asset_store.get_fontdue_font(&label.font_key) {
//             self.layout.append(
//                 &[font],
//                 &TextStyle::new(&label.text, label.font_size as f32, 0),
//             );
//         }

//         let mut font_texture_width = 0;
//         let mut font_texture_height = 0;
//         let mut font_texture_key: Option<TextureKey> = None;

//         if let Some(font) = asset_store.get_font_mut(&label.font_key) {
//             font_texture_key = Some(font.font_texture_key.clone());
//         }

//         if let Some(font_texture_key) = font_texture_key.as_ref() {
//             if let Some(texture) = asset_store.get_texture(font_texture_key) {
//                 font_texture_width = texture.width;
//                 font_texture_height = texture.height;
//             }
//         }

//         let mut draw_calls: Vec<(
//             f32,         // z_index
//             Vec2,        // real_scale
//             Vec2,        // real_position
//             Rectangle,   // target
//             Color,       // color
//             bool,        // centered
//             bool,        // Visible
//             Option<f32>, // max_width
//         )> = Vec::new();

//         let mut remaining_char_count = label.visible_characters;
//         if label.visible_characters < 0 {
//             remaining_char_count = label.text.len() as i64;
//         }

//         for glyph in self.layout.glyphs() {
//             let glyph_key = glyph.key;
//             let x = glyph.x;
//             let y = glyph.y;

//             let mut need_to_cache_glyph = false;
//             if let Some(font) = asset_store.get_font(&label.font_key) {
//                 need_to_cache_glyph = !font.characters.contains_key(&glyph_key);
//             }

//             if need_to_cache_glyph {
//                 cache_glyph(
//                     &mut ctx,
//                     &mut asset_store,
//                     &label.font_key,
//                     glyph_key,
//                     label.font_size,
//                 )?;
//             }

//             if let Some(font) = asset_store.get_font_mut(&label.font_key) {
//                 let font_data = &font.characters[&glyph_key];
//                 {
//                     let left_coord = (font_data.offset_x as f32 + x) * label.scale;
//                     let top_coord = y * label.scale;

//                     let target = Rectangle::new(
//                         (font_data.glyph_x as f32) / font_texture_width as f32,
//                         (font_data.glyph_y as f32) / font_texture_height as f32,
//                         (font_data.glyph_w as f32) / font_texture_width as f32,
//                         (font_data.glyph_h as f32) / font_texture_height as f32,
//                     );

//                     let real_scale = Vec2::new(
//                         label.scale * target.width * font_texture_width as f32,
//                         label.scale * target.height * font_texture_height as f32 * -1.0,
//                     );
//                     let real_position = Vec2::from(*position)
//                         + Vec2::from(label.offset)
//                         + vec2(left_coord, -top_coord);

//                     if remaining_char_count > 0 {
//                         draw_calls.push((
//                             label.z_index,
//                             real_scale,
//                             real_position,
//                             target,
//                             label.color,
//                             label.centered,
//                             label.visible,
//                             label.max_width,
//                         ));
//                     }
//                 }

//                 remaining_char_count -= 1;
//             }
//         }

//         if let Some(font_texture_key) = font_texture_key {
//             for draw_call in draw_calls {
//                 let (
//                     z_index,
//                     real_scale,
//                     mut real_position,
//                     target,
//                     mut color,
//                     centered,
//                     visible,
//                     max_width,
//                 ) = draw_call;

//                 if centered {
//                     if let Some(max_width) = max_width {
//                         real_position.x -= max_width / 2.0;
//                     }
//                 }

//                 if !visible {
//                     color.a = 0;
//                 }

//                 draw_texture(
//                     &self.settings,
//                     &mut ctx,
//                     &mut asset_store,
//                     &font_texture_key,
//                     z_index,
//                     real_scale,
//                     0.0,
//                     Vec2::new(0.0, 0.0),
//                     real_position,
//                     target,
//                     color,
//                     self.current_resolution,
//                 );
//             }
//         }

//         Ok(())
//     }

//     #[inline]
//     pub(crate) fn draw_color_rect(
//         &mut self,
//         mut ctx: &mut Context,
//         mut asset_store: &mut AssetStore,
//         color_rect: &ColorRect,
//         translation: &Translation,
//     ) {
//         if !color_rect.visible {
//             return;
//         }

//         let (width, height) = (color_rect.width, color_rect.height);
//         let mut offset = color_rect.offset;

//         if color_rect.centered {
//             offset.x -= (color_rect.width / 2) as f32;
//             offset.y -= (color_rect.height / 2) as f32;
//         }

//         let real_scale = Vec2::new(width as f32, height as f32);
//         let real_position = Vec2::from(*translation) + Vec2::from(offset);

//         draw_texture(
//             &self.settings,
//             &mut ctx,
//             &mut asset_store,
//             &TextureKey::default(),
//             color_rect.z_index,
//             real_scale,
//             color_rect.rotation,
//             Vec2::new(0.0, 0.0),
//             real_position,
//             Rectangle::new(0.0, 0.0, 1.0, 1.0),
//             color_rect.color,
//             self.current_resolution,
//         )
//     }

//     #[inline]
//     pub(crate) fn draw_tilemap(
//         &mut self,
//         ctx: &mut Context,
//         asset_store: &mut AssetStore,
//         texture_key: TextureKey,
//         tiles: Vec<isize>,
//         tile_size: Vector2<usize>,
//         width: usize,
//         _height: usize,
//         _z_index: f32,
//         visible: bool,
//         translation: &Translation,
//     ) {
//         if !visible {
//             return;
//         }

//         let mut tileset_width = 0;

//         if let Some(texture) = asset_store.get_texture(&texture_key) {
//             tileset_width = texture.width as usize / tile_size.x;
//         }

//         let mut sprite = Sprite::from_texture(texture_key);

//         let tile_width = tile_size.x as f32;
//         let tile_height = tile_size.y as f32;

//         let mut x = 0;
//         let mut y = 0;
//         for tile in tiles {
//             if tile >= 0 {
//                 let tile_id = tile as usize;

//                 let tile_x = tile_id % tileset_width;
//                 let tile_y = tile_id / tileset_width;

//                 sprite.target = Rectangle::new(
//                     tile_x as f32 * tile_width,
//                     tile_y as f32 * tile_height,
//                     tile_width,
//                     tile_height,
//                 );
//                 let translation = translation.clone()
//                     + Translation::new(tile_width * x as f32, tile_height * y as f32);

//                 self.draw_sprite(ctx, asset_store, &sprite, &translation);
//             }

//             x += 1;
//             if x >= width {
//                 x = 0;
//                 y += 1;
//             }
//         }
//     }

//     #[inline]
//     pub(crate) fn draw_aseprite(
//         &mut self,
//         mut ctx: &mut Context,
//         mut asset_store: &mut AssetStore,
//         sprite: &Sprite,
//         rotation: f32,
//         offset: &Vector2<f32>,
//         centered: bool,
//         visible: bool,
//         scale: &Vector2<f32>,
//         color: &Color,
//         z_index: f32,
//         position: &Translation,
//     ) {
//         if !visible {
//             return;
//         }

//         let texture = asset_store.get_texture(&sprite.texture_key).unwrap();
//         let mut target = Rectangle::new(
//             sprite.target.x / texture.width as f32,
//             sprite.target.y / texture.height as f32,
//             sprite.target.width / texture.width as f32,
//             sprite.target.height / texture.height as f32,
//         );

//         if sprite.target.is_zero_sized() {
//             target = Rectangle::new(0.0, 0.0, 1.0, 1.0);
//         }

//         let mut offset = Vec2::from(*offset);
//         if centered {
//             let size = if sprite.target.is_zero_sized() {
//                 vec2(texture.width.into(), texture.height.into())
//             } else {
//                 vec2(sprite.target.width, sprite.target.height)
//             };

//             offset -= Vec2::from(*scale) * size / 2.0;
//         }

//         let real_scale = Vec2::new(
//             scale.x * target.width * (f32::from(texture.width)),
//             scale.y * target.height * (f32::from(texture.height)),
//         );
//         let real_position = Vec2::from(*position) + offset;

//         draw_texture(
//             &self.settings,
//             &mut ctx,
//             &mut asset_store,
//             &sprite.texture_key,
//             z_index,
//             real_scale,
//             rotation,
//             Vec2::new(0.0, 0.0),
//             real_position,
//             target,
//             *color,
//             self.current_resolution,
//         )
//     }

//     #[inline]
//     pub(crate) fn draw_sprite(
//         &mut self,
//         mut ctx: &mut Context,
//         mut asset_store: &mut AssetStore,
//         sprite: &Sprite,
//         position: &Translation,
//     ) {
//         if !sprite.visible {
//             return;
//         }

//         let texture = asset_store.get_texture(&sprite.texture_key).unwrap();
//         let mut target = Rectangle::new(
//             sprite.target.x / texture.width as f32,
//             sprite.target.y / texture.height as f32,
//             sprite.target.width / texture.width as f32,
//             sprite.target.height / texture.height as f32,
//         );

//         if sprite.target.is_zero_sized() {
//             target = Rectangle::new(0.0, 0.0, 1.0, 1.0);
//         }

//         let mut offset = sprite.offset;
//         if sprite.centered {
//             if sprite.target.is_zero_sized() {
//                 offset.x -= sprite.scale.x * texture.width as f32 / 2.0;
//                 offset.y -= sprite.scale.y * texture.height as f32 / 2.0;
//             } else {
//                 offset.x -= sprite.scale.x * sprite.target.width / 2.0;
//                 offset.y -= sprite.scale.y * sprite.target.height / 2.0;
//             }
//         }

//         let real_scale = Vec2::new(
//             sprite.scale.x * target.width * (f32::from(texture.width)),
//             sprite.scale.y * target.height * (f32::from(texture.height)),
//         );
//         let real_position = Vec2::new(position.x + offset.x, position.y + offset.y);

//         draw_texture(
//             &self.settings,
//             &mut ctx,
//             &mut asset_store,
//             &sprite.texture_key,
//             sprite.z_index,
//             real_scale,
//             sprite.rotation,
//             Vec2::new(0.0, 0.0),
//             real_position,
//             target,
//             sprite.color,
//             self.current_resolution,
//         )
//     }
// }

// #[inline]
// fn draw_texture(
//     settings: &RenderSettings,
//     mut ctx: &mut Context,
//     asset_store: &mut AssetStore,
//     texture_key: &TextureKey,
//     _z_index: f32,
//     scale: Vec2,
//     rotation: f32,
//     offset: Vec2,
//     mut position: Vec2,
//     source: Rectangle,
//     color: Color,
//     resolution: (usize, usize),
// ) {
//     // Bump position up by half a unit then floor, for pixel snap
//     if settings.pixel_snap {
//         position = Vec2::new((position.x + 0.5).floor(), (position.y + 0.5).floor());
//     }

//     let projection = Mat4::orthographic_rh_gl(
//         0.0,
//         resolution.0 as f32,
//         0.0,
//         resolution.1 as f32,
//         -1.0,
//         1.0,
//     );

//     let mut uniforms = Uniforms {
//         projection,
//         model: crate::rendering::param_to_instance_transform(rotation, scale, offset, position),
//         ..Default::default()
//     };

//     let color = color.to_percentage();
//     uniforms.source = Vec4::new(source.x, source.y, source.width, source.height);
//     uniforms.color = Vec4::new(color.0, color.1, color.2, color.3);

//     if let Some(texture) = asset_store.get_texture(texture_key) {
//         texture.inner.set_filter(&mut ctx, texture.filter);
//         ctx.apply_bindings(&texture.bindings);
//         ctx.apply_uniforms(&uniforms);
//         ctx.draw(0, 6, 1);
//     }
// }

#[inline]
fn get_camera_and_camera_transform(world: &World) -> (Camera, Transform) {
    let mut cam = Camera::default();
    let mut cam_transform = Transform::from_translation((0.0, 0.0));
    let mut entity_holding_camera: Option<Entity> = None;

    for (id, camera) in world.query::<&Camera>().iter() {
        if camera.is_active {
            cam = *camera;
            entity_holding_camera = Some(id);
        }
    }

    if let Some(entity) = entity_holding_camera {
        if let Ok(transform) = world.get::<&mut Transform>(entity) {
            cam_transform = *transform;
        }
    }

    (cam, cam_transform)
}

pub(crate) enum Drawable {
    Aseprite {
        sprite: Sprite,
        rotation: f32,
        color: Color,
        centered: bool,
        scale: Vector2<f32>,
        offset: Vector2<f32>,
        z_index: f32,
        visible: bool,
    },
    Sprite {
        sprite: Sprite,
    },
    ColorRect {
        color_rect: ColorRect,
    },
    Label {
        label: Label,
    },
    Tilemap {
        texture_key: TextureKey,
        tiles: Vec<isize>,
        tile_size: Vector2<usize>,
        width: usize,
        height: usize,
        visible: bool,
        z_index: f32,
    },
}
trait ToDrawable {
    /// Returns a rectangle representing the visual size of this drawable, if a
    /// culling check should be performed. `None` can be returned to skip the
    /// culling check.
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        asset_store: &mut AssetStore,
    ) -> Option<Rectangle>;

    fn to_drawable(&self) -> Drawable;

    fn z_index(&self) -> f32;
}

impl ToDrawable for Tilemap {
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        _asset_store: &mut AssetStore,
    ) -> Option<Rectangle> {
        let width = self.width * self.tile_size.x;
        let height = self.height * self.tile_size.y;
        let visible_bounds = Rectangle::new(
            transform.translation.x,
            transform.translation.y,
            width as f32,
            height as f32,
        );

        Some(visible_bounds)
    }

    fn to_drawable(&self) -> Drawable {
        Drawable::Tilemap {
            texture_key: self.tilesheet.clone(),
            tiles: self
                .tiles
                .iter()
                .map(|tile| {
                    if let Some(tile_id) = tile {
                        *tile_id as isize
                    } else {
                        -1
                    }
                })
                .collect(),
            width: self.width,
            height: self.height,
            z_index: self.z_index,
            visible: self.visible,
            tile_size: self.tile_size.clone(),
        }
    }

    fn z_index(&self) -> f32 {
        self.z_index
    }
}
impl ToDrawable for AutoTilemap {
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        asset_store: &mut AssetStore,
    ) -> Option<Rectangle> {
        self.tilemap.get_visible_bounds(transform, asset_store)
    }

    fn to_drawable(&self) -> Drawable {
        self.tilemap.to_drawable()
    }

    fn z_index(&self) -> f32 {
        self.tilemap.z_index
    }
}

#[cfg(feature = "aseprite")]
use crate::rendering::components::Aseprite;

#[cfg(feature = "aseprite")]
impl ToDrawable for Aseprite {
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        asset_store: &mut AssetStore,
    ) -> Option<Rectangle> {
        self.get_sprite().get_visible_bounds(transform, asset_store)
    }

    fn to_drawable(&self) -> Drawable {
        Drawable::Aseprite {
            sprite: self.get_sprite().clone(),
            offset: self.offset,
            scale: self.scale,
            centered: self.centered,
            color: self.color,
            rotation: self.rotation,
            z_index: self.z_index,
            visible: self.visible,
        }
    }

    fn z_index(&self) -> f32 {
        self.z_index
    }
}

impl ToDrawable for Sprite {
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        asset_store: &mut AssetStore,
    ) -> Option<Rectangle> {
        let mut bounds = self.target.clone();

        if bounds.is_zero_sized() {
            if let Some(texture) = asset_store.get_texture(&self.texture_key) {
                // bounds.width = texture.width as f32;
                // bounds.height = texture.height as f32;
            }
        }

        // Set the visibility rect at the position of the sprite
        bounds.x = transform.translation.x + self.offset.x;
        bounds.y = transform.translation.y + self.offset.y;

        if self.centered {
            bounds.x -= bounds.width as f32 / 2.0;
            bounds.y -= bounds.height as f32 / 2.0;
        }

        // Take the sprite's scale factor into account
        bounds.width *= self.scale.x;
        bounds.height *= self.scale.y;

        Some(bounds)
    }

    fn to_drawable(&self) -> Drawable {
        Drawable::Sprite {
            sprite: self.clone(),
        }
    }

    fn z_index(&self) -> f32 {
        self.z_index
    }
}

impl ToDrawable for UIButton {
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        asset_store: &mut AssetStore,
    ) -> Option<Rectangle> {
        let sprite = Sprite::from_texture(self.current_texture().clone());
        sprite.get_visible_bounds(transform, asset_store)
    }

    fn to_drawable(&self) -> Drawable {
        let mut sprite = Sprite::from_texture(self.current_texture().clone());
        sprite.visible = self.visible;

        Drawable::Sprite { sprite }
    }

    fn z_index(&self) -> f32 {
        self.z_index
    }
}

impl ToDrawable for ColorRect {
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        _asset_store: &mut AssetStore,
    ) -> Option<Rectangle> {
        let mut bounds = Rectangle::new(
            transform.translation.x + self.offset.x,
            transform.translation.y + self.offset.y,
            self.width as f32,
            self.height as f32,
        );
        if self.centered {
            bounds.x -= self.width as f32 / 2.0;
            bounds.y -= self.height as f32 / 2.0;
        }
        Some(bounds)
    }

    fn to_drawable(&self) -> Drawable {
        // Drawable::ColorRect { color_rect: *self }
        let mut sprite = Sprite::from_texture(TextureKey::default());
        // We multiply our 1 pixel sprite by our size to achieve that size
        sprite.scale.x = self.width as f32;
        sprite.scale.y = self.height as f32;
        sprite.color = self.color;
        sprite.rotation = self.rotation;
        sprite.centered = self.centered;
        sprite.offset = self.offset;

        Drawable::Sprite { sprite }
    }

    fn z_index(&self) -> f32 {
        self.z_index
    }
}

impl ToDrawable for Label {
    fn get_visible_bounds(
        &self,
        _transform: &Transform,
        _asset_store: &mut AssetStore,
    ) -> Option<Rectangle> {
        None
    }

    fn to_drawable(&self) -> Drawable {
        Drawable::Label {
            label: self.clone(),
        }
    }

    fn z_index(&self) -> f32 {
        self.z_index
    }
}

pub(crate) struct DrawCommand {
    pub drawable: Drawable,
    pub transform: Transform,
    pub z_index: f32,
}

struct DrawCommandAdder {
    /// Bounds for culling checks, or None if no culling checks should be
    /// performed.
    camera_bounds: Option<Rectangle>,
    camera: Camera,
    camera_transform: Transform,
}

impl DrawCommandAdder {
    fn new(engine: &RenderingEngine, world: &World) -> Self {
        let (camera, camera_transform) = get_camera_and_camera_transform(world);
        let camera_bounds = if engine.settings.frustrum_culling {
            let screen_size = (engine.size.width as f32, engine.size.height as f32);

            let (camera, camera_transform) = get_camera_and_camera_transform(world);
            let mut camera_view_region = Rectangle::new(
                camera_transform.translation.x - screen_size.0 / 2.0,
                camera_transform.translation.y - screen_size.1 / 2.0,
                screen_size.0,
                screen_size.1,
            );
            camera_view_region.width *= camera.zoom;
            camera_view_region.height *= camera.zoom;

            Some(camera_view_region)
        } else {
            None
        };

        Self {
            camera,
            camera_transform,
            camera_bounds,
        }
    }

    fn add_draw_commands<'a, D>(
        &self,
        draw_queue: &mut Vec<DrawCommand>,
        world: &'a World,
        asset_store: &mut AssetStore,
    ) where
        D: hecs::Component + ToDrawable + 'a,
    {
        let camera_transform = self.camera_transform.clone();
        draw_queue.extend(
            world
                .query::<(&D, &Transform)>()
                .into_iter()
                .filter(|(_entity, (to_drawable, transform))| {
                    if let Some(camera_bounds) = self.camera_bounds {
                        if let Some(drawable_bounds) =
                            to_drawable.get_visible_bounds(transform, asset_store)
                        {
                            return camera_bounds.intersects_with(&drawable_bounds);
                        }
                    }

                    true
                })
                .map(|(_entity, (to_drawable, transform))| {
                    let drawable = to_drawable.to_drawable();

                    // TODO: Add camera transform offset to drawable transform
                    let transform = *transform - camera_transform.clone();
                    DrawCommand {
                        drawable,
                        transform,
                        z_index: to_drawable.z_index(),
                    }
                }),
        );
    }
}

// #[inline]
// pub(crate) fn create_render_texture(
//     w: usize,
//     h: usize,
//     key: TextureKey,
//     ctx: &mut Context,
//     asset_store: &mut AssetStore,
// ) -> Result<TextureKey, EmeraldError> {
//     let color_img = miniquad::Texture::new_render_texture(
//         ctx,
//         TextureParams {
//             width: w as _,
//             height: h as _,
//             format: TextureFormat::RGBA8,
//             wrap: TextureWrap::Clamp,
//             filter: FilterMode::Nearest,
//         },
//     );

//     let texture = crate::rendering::Texture::from_texture(ctx, key.clone(), color_img)?;
//     asset_store.insert_texture(key.clone(), texture);

//     Ok(key)
// }

// fn current_window_resolution(ctx: &mut Context) -> (usize, usize) {
//     let (w, h) = ctx.screen_size();
//     let dpi_scale = ctx.dpi_scale();

//     ((w * dpi_scale) as usize, (h * dpi_scale) as usize)
// }
