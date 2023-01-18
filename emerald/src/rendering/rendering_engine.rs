use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    ops::Range,
};

use fontdue::layout::{Layout, LayoutSettings, TextStyle};
use hecs::Entity;
use rapier2d::{na::Vector2, prelude::RigidBodyHandle};
use wgpu::{util::DeviceExt, TextureView};
use wgpu::{Adapter, BindGroup, BindGroupLayout, Device, Queue, RenderPipeline, Surface};
use winit::dpi::PhysicalSize;

use crate::{
    autotilemap::AutoTilemap,
    font::FontKey,
    render_settings::RenderSettings,
    shaders::{
        self,
        textured_quad::{CameraUniform, Vertex},
    },
    texture::{Texture, TextureKey},
    tilemap::Tilemap,
    AssetStore, Color, EmeraldError, Rectangle, Scale, Transform, Translation, UIButton, World,
};

use super::components::{get_bounding_box_of_triangle, Camera, ColorRect, ColorTri, Label, Sprite};

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum BindGroupLayoutId {
    TextureQuad,
}

/// A set of textured tris that will be drawn.
struct TexturedTriDraw {
    pub key: TextureKey,
    pub vertices_range: Range<u64>,
    pub indices_range: Range<u64>,
    count: usize,
    indices_per_draw: usize,
    vertices_per_draw: usize,
}
impl TexturedTriDraw {
    pub fn new(
        key: TextureKey,
        vertices_start: u64,
        vertices_set_size: u64,
        vertices_per_draw: usize,
        indices_start: u64,
        indices_set_size: u64,
        indices_per_draw: usize,
    ) -> Self {
        Self {
            key,
            vertices_range: vertices_start..vertices_set_size,
            indices_range: indices_start..indices_set_size,
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
            &mut bind_groups,
            &bind_group_layouts,
            asset_store,
            &device,
            &queue,
            1,
            1,
            &[255, 255, 255, 255],
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
    pub fn draw_world_with_transform(
        &mut self,
        world: &mut World,
        transform: Transform,
        asset_store: &mut AssetStore,
    ) -> Result<(), EmeraldError> {
        let (camera, camera_transform) = get_camera_and_camera_transform(world);
        let camera_transform = camera_transform - transform;
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

    #[inline]
    pub fn draw_world(
        &mut self,
        world: &mut World,
        asset_store: &mut AssetStore,
    ) -> Result<(), EmeraldError> {
        self.draw_world_with_transform(world, Transform::default(), asset_store)
    }

    #[inline]
    pub fn draw_colliders(
        &mut self,
        asset_store: &mut AssetStore,
        world: &mut World,
        collider_color: Color,
    ) -> Result<(), EmeraldError> {
        let screen_size = self.active_size;
        let mut color_rect = ColorRect {
            color: collider_color,
            ..Default::default()
        };
        color_rect.color = collider_color;
        let (camera, camera_transform) = get_camera_and_camera_transform(world);

        for (_id, body_handle) in world.inner.query::<&RigidBodyHandle>().iter() {
            if let Some(body) = world.physics_engine.bodies.get(*body_handle) {
                for collider_handle in body.colliders() {
                    if let Some(collider) = world.physics_engine.colliders.get(*collider_handle) {
                        let aabb = collider.compute_aabb();
                        let body_translation =
                            Translation::from(Vector2::from(aabb.center().coords));
                        color_rect.width = aabb.half_extents().x as u32 * 2;
                        color_rect.height = aabb.half_extents().y as u32 * 2;

                        let translation = {
                            let mut translation = body_translation - camera_transform.translation;

                            translation
                        };

                        self.push_draw_command(
                            asset_store,
                            DrawCommand {
                                drawable: Drawable::ColorRect { color_rect },
                                transform: Transform::from_translation(translation),
                                z_index: 0.0,
                            },
                        )?;
                    }
                }
            }
        }

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

    pub fn begin(&mut self, _asset_store: &mut AssetStore) -> Result<(), EmeraldError> {
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
        let vertex_set_size = (std::mem::size_of::<Vertex>() * 4) as u64;
        let indices_set_size: u64 = std::mem::size_of::<u32>() as u64 * 6;

        let mut textured_tri_draws: Vec<TexturedTriDraw> = Vec::new();
        self.vertices.clear();
        self.indices.clear();

        let draw_queue = &mut self.draw_queue;
        while let Some(draw_command) = draw_queue.pop_back() {
            match draw_command.drawable {
                Drawable::Sprite { sprite } => {
                    if !sprite.visible {
                        continue;
                    }

                    draw_textured_quad(
                        asset_store,
                        sprite.texture_key,
                        sprite.target,
                        sprite.offset,
                        sprite.scale,
                        sprite.rotation,
                        sprite.centered,
                        sprite.color,
                        draw_command.transform,
                        self.active_size.clone(),
                        &mut self.vertices,
                        &mut self.indices,
                        &mut textured_tri_draws,
                        &self.settings,
                    )?;
                }
                Drawable::Aseprite {
                    mut sprite,
                    rotation,
                    color,
                    centered,
                    scale,
                    offset,
                    z_index,
                    visible,
                } => {
                    if !visible {
                        continue;
                    }

                    sprite.rotation = rotation;
                    sprite.color = color;
                    sprite.centered = centered;
                    sprite.scale = scale;
                    sprite.offset = offset;
                    sprite.z_index = z_index;
                    sprite.visible = visible;

                    // Aseprites can be broken down into a sprite draw
                    draw_queue.push_back(DrawCommand {
                        drawable: Drawable::Sprite { sprite },
                        ..draw_command
                    });
                    continue;
                }
                Drawable::ColorRect { color_rect } => {
                    if !color_rect.visible {
                        continue;
                    }

                    let mut sprite = Sprite::default();
                    sprite.target = Rectangle::new(0.0, 0.0, 0.0, 0.0);
                    sprite.scale.x = color_rect.width as f32;
                    sprite.scale.y = color_rect.height as f32;
                    sprite.rotation = color_rect.rotation;
                    sprite.centered = color_rect.centered;
                    sprite.color = color_rect.color;
                    draw_queue.push_back(DrawCommand {
                        drawable: Drawable::Sprite { sprite },
                        ..draw_command
                    });
                    continue;
                }

                Drawable::ColorTri { color_tri } => draw_textured_tri(
                    asset_store,
                    TextureKey::default(),
                    color_tri.points,
                    [
                        Vector2::new(0.0, 0.0),
                        Vector2::new(1.0, 1.0),
                        Vector2::new(0.0, 1.0),
                    ],
                    color_tri.color,
                    draw_command.transform,
                    self.active_size.clone(),
                    &mut self.vertices,
                    &mut self.indices,
                    &mut textured_tri_draws,
                    &self.settings,
                )?,
                Drawable::Label { label } => {
                    if !label.visible {
                        continue;
                    }

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

                    let mut to_draw = Vec::new();
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

                            let scale = Vector2::new(label.scale, label.scale);
                            let offset = label.offset;
                            let rotation = 0.0;
                            if label.centered {
                                if let Some(width) = &label.max_width {
                                    transform.translation.x -= width / 2.0;
                                }
                            }

                            if remaining_char_count < 0 || target.is_zero_sized() {
                                continue;
                            }

                            to_draw.push((
                                font.font_texture_key.clone(),
                                target,
                                offset,
                                scale,
                                rotation,
                                transform,
                            ));

                            remaining_char_count -= 1;
                        } else {
                            return Err(EmeraldError::new(format!(
                                "Font not found: {:?}",
                                label.font_key
                            )));
                        }
                    }
                    for (texture_key, target, offset, scale, rotation, transform) in to_draw {
                        draw_textured_quad(
                            asset_store,
                            texture_key,
                            target,
                            offset,
                            scale,
                            rotation,
                            false,
                            label.color,
                            transform,
                            self.active_size.clone(),
                            &mut self.vertices,
                            &mut self.indices,
                            &mut textured_tri_draws,
                            &self.settings,
                        )?;
                    }
                }
                Drawable::Tilemap {
                    texture_key,
                    tiles,
                    tile_size,
                    width,
                    visible,
                    height,
                    ..
                } => {
                    if !visible {
                        continue;
                    }
                    let tileset_width;
                    let tileset_height;

                    if let Some(texture) = asset_store.get_texture(&texture_key) {
                        tileset_width = texture.size.width as usize / tile_size.x;
                        tileset_height = texture.size.height as usize / tile_size.y;
                    } else {
                        return Err(EmeraldError::new(format!(
                            "Tilemap Texture {:?} not found.",
                            texture_key
                        )));
                    }

                    let tile_width = tile_size.x as f32;
                    let tile_height = tile_size.y as f32;

                    let mut x = 0;
                    let mut y = 0;
                    for tile in tiles {
                        if tile >= 0 {
                            let tile_id = tile as usize;

                            let tile_x = tile_id % tileset_width;
                            let tile_y = tileset_height - 1 - tile_id / tileset_width;
                            let target = Rectangle::new(
                                tile_x as f32 * tile_width,
                                tile_y as f32 * tile_height,
                                tile_width,
                                tile_height,
                            );
                            let translation = draw_command.transform.translation.clone()
                                + Translation::new(tile_width * x as f32, tile_height * y as f32);

                            let offset = Vector2::new(0.0, 0.0);
                            let scale = Vector2::new(1.0, 1.0);
                            let rotation = 0.0;
                            let centered = true;
                            let color = crate::colors::WHITE;
                            let transform = Transform::from_translation(translation);
                            let active_size = self.active_size;

                            draw_textured_quad(
                                asset_store,
                                texture_key.clone(),
                                target,
                                offset,
                                scale,
                                rotation,
                                centered,
                                color,
                                transform,
                                active_size,
                                &mut self.vertices,
                                &mut self.indices,
                                &mut textured_tri_draws,
                                &self.settings,
                            )?;
                        }

                        x += 1;
                        if x >= width {
                            x = 0;
                            y += 1;
                        }
                    }
                }
            }
        }
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

        for draw_call in textured_tri_draws {
            let indices_count = draw_call.count() as u32 * draw_call.indices_per_draw as u32;
            if let Some(texture_bind_group) = self.bind_groups.get(&draw_call.key.0) {
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
                    "Unable to find texture bind group for {:?}",
                    &draw_call.key.0
                )));
            }
        }
        Ok(())
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

const VERTEX_SIZE: u64 = std::mem::size_of::<Vertex>() as u64;
const INDEX_SIZE: u64 = std::mem::size_of::<u32>() as u64;

const TEXTURED_QUAD_VERTICES_PER_DRAW: usize = 4;
const TEXTURED_QUAD_INDICES_PER_DRAW: usize = 6;
const TEXTURED_QUAD_VERTEX_SET_SIZE: u64 = VERTEX_SIZE * TEXTURED_QUAD_VERTICES_PER_DRAW as u64; // 4 vertices, 1 for each corner of the quad
const TEXTURED_QUAD_INDICES_SET_SIZE: u64 = INDEX_SIZE * TEXTURED_QUAD_INDICES_PER_DRAW as u64; // 6 indices to draw a quad using 4 vertices

fn draw_textured_quad(
    asset_store: &mut AssetStore,
    texture_key: TextureKey,
    mut target: Rectangle,
    offset: Vector2<f32>,
    scale: Vector2<f32>,
    rotation: f32,
    centered: bool,
    color: Color,
    transform: Transform,
    active_size: PhysicalSize<u32>,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    textured_tri_draws: &mut Vec<TexturedTriDraw>,
    settings: &RenderSettings,
) -> Result<(), EmeraldError> {
    let texture_size;
    if let Some(texture) = asset_store.get_texture(&texture_key) {
        texture_size = (texture.size.width as f32, texture.size.height as f32);

        // Zeroed target means display entire texture
        if target.is_zero_sized() {
            target.width = texture_size.0;
            target.height = texture_size.1;
        }
    } else {
        return Err(EmeraldError::new(format!(
            "Unable to find texture {:?}",
            texture_key
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
            .last()
            .filter(|draw| draw.key.0 == texture_key.0)
            .is_some();

    let mut index_start: u32 = 0;
    if same_texture {
        if let Some(draw) = textured_tri_draws.last_mut() {
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
        if let Some(draw) = textured_tri_draws.last() {
            indices_start = draw.indices_range.end;
            vertices_start = draw.vertices_range.end;
        }

        textured_tri_draws.push(TexturedTriDraw::new(
            texture_key,
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
    asset_store: &mut AssetStore,
    texture_key: TextureKey,
    mut points: [Vector2<f32>; 3],
    mut target: [Vector2<f32>; 3],
    color: Color,
    transform: Transform,
    active_size: PhysicalSize<u32>,
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    textured_tri_draws: &mut Vec<TexturedTriDraw>,
    settings: &RenderSettings,
) -> Result<(), EmeraldError> {
    let texture_size;
    if let Some(texture) = asset_store.get_texture(&texture_key) {
        texture_size = (texture.size.width as f32, texture.size.height as f32);
    } else {
        return Err(EmeraldError::new(format!(
            "Unable to find texture {:?}",
            texture_key
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
    let mut same_texture = len > 0
        && textured_tri_draws
            .last()
            .filter(|draw| draw.key.0 == texture_key.0)
            .is_some();

    let mut index_start: u32 = 0;
    if same_texture {
        if let Some(draw) = textured_tri_draws.last_mut() {
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
        if let Some(draw) = textured_tri_draws.last() {
            indices_start = draw.indices_range.end;
            vertices_start = draw.vertices_range.end;
        }

        textured_tri_draws.push(TexturedTriDraw::new(
            texture_key,
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

#[inline]
fn get_camera_and_camera_transform(world: &World) -> (Camera, Transform) {
    let mut cam = Camera::default();
    let mut cam_transform = Transform::default();
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
    ColorTri {
        color_tri: ColorTri,
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
        let mut sprite = self.get_sprite().clone();
        sprite.offset = self.offset.clone();

        sprite.get_visible_bounds(transform, asset_store)
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
                bounds.width = texture.size.width as f32;
                bounds.height = texture.size.height as f32;
            }
        }

        // Set the visibility rect at the position of the sprite
        bounds.x = transform.translation.x + self.offset.x;
        bounds.y = transform.translation.y + self.offset.y;

        bounds.width *= self.scale.x;
        bounds.height *= self.scale.y;

        if self.centered {
            bounds.x -= bounds.width as f32 / 2.0;
            bounds.y -= bounds.height as f32 / 2.0;
        }

        // Take the sprite's scale factor into account

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
            let screen_size = (
                engine.active_size.width as f32,
                engine.active_size.height as f32,
            );

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

                    let transform = *transform;
                    DrawCommand {
                        drawable,
                        transform,
                        z_index: to_drawable.z_index(),
                    }
                }),
        );
    }
}
