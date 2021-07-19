use crate::rendering::components::*;
use crate::rendering::*;
use crate::world::*;
use crate::*;

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use glam::{Mat4, Vec2, Vec4};
use miniquad::*;
use std::collections::HashMap;
use std::sync::Arc;

const EMERALD_TEXTURE_PIPELINE_NAME: &str = "emerald_default_texture_pipline";

// The default "screen" pass.
// Renders to a texture the size of the screen when rendering begins.
const EMERALD_DEFAULT_RENDER_TARGET: &str = "emerald_default_render_target";

// When reference is 2 or less, the only reference remaining is by the engine itself. So it is safe to cleanup.
// 1 reference is by the texture_key_map
// 1 reference is held by the texture itself
const MINIMUM_TEXTURE_REFERENCES: usize = 1;

pub(crate) struct RenderingEngine {
    pub(crate) settings: RenderSettings,
    pipelines: HashMap<String, Pipeline>,
    render_pass: RenderPass,
    layout: Layout,
    render_texture_counter: usize,
    last_screen_size: (usize, usize),
    screen_texture_key: TextureKey,
    current_render_texture_key: TextureKey,
    current_resolution: (usize, usize),
}
impl RenderingEngine {
    pub(crate) fn new(ctx: &mut Context, settings: RenderSettings, asset_store: &mut AssetStore) -> Self {
        let mut pipelines = HashMap::new();

        let shader = Shader::new(ctx, VERTEX, FRAGMENT, shaders::meta()).unwrap();
        let mut params = PipelineParams::default();
        params.depth_write = true;
        params.color_blend = Some(BlendState::new(
            Equation::Add,
            BlendFactor::Value(BlendValue::SourceAlpha),
            BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
        ));
        params.alpha_blend = Some(BlendState::new(
            Equation::Add,
            BlendFactor::Zero,
            BlendFactor::One,
        ));

        let texture_pipeline = Pipeline::with_params(
            ctx,
            &[BufferLayout::default()],
            &[VertexAttribute::new("position", VertexFormat::Float2)],
            shader,
            params,
        );

        pipelines.insert(EMERALD_TEXTURE_PIPELINE_NAME.to_string(), texture_pipeline);

        let mut render_texture_counter = 0;
        let key = TextureKey::new(String::from(EMERALD_DEFAULT_RENDER_TARGET));
        let (w, h) = ctx.screen_size();
        let screen_texture_key = create_render_texture(w as usize, h as usize, key, ctx, asset_store).unwrap();
        render_texture_counter += 1;

        let texture = asset_store.get_texture(&screen_texture_key).unwrap();
        let render_pass = RenderPass::new(ctx, texture.inner, None);
        let current_render_texture_key = screen_texture_key.clone();
        let current_resolution = (w as usize, h as usize);

        RenderingEngine {
            settings,
            pipelines,
            layout: Layout::new(CoordinateSystem::PositiveYDown),
            render_texture_counter,
            render_pass,
            last_screen_size: (0, 0),
            screen_texture_key,
            current_render_texture_key,
            current_resolution,
        }
    }

    #[inline]
    pub(crate) fn create_render_texture(&mut self, w: usize, h: usize, ctx: &mut Context, asset_store: &mut AssetStore) -> Result<TextureKey, EmeraldError> {
        self.render_texture_counter += 1;
        let key = TextureKey::new(format!("emd_render_texture_{}", self.render_texture_counter));

        create_render_texture(w, h, key, ctx, asset_store)
    }

    #[inline]
    pub(crate) fn pre_draw(&mut self, ctx: &mut Context, asset_store: &mut AssetStore) -> Result<(), EmeraldError> {
        let (w, h) = ctx.screen_size();
        let (prev_w, prev_h) = self.last_screen_size;

        if w as usize != prev_w || h as usize != prev_h {
            self.update_screen_texture_size(ctx, w as usize, h as usize, asset_store)?;
        }

        Ok(())
    }

    #[inline]
    fn update_screen_texture_size(&mut self, ctx: &mut Context, w: usize, h: usize, asset_store: &mut AssetStore) -> Result<TextureKey, EmeraldError> {
        let key = TextureKey::new(String::from(EMERALD_DEFAULT_RENDER_TARGET));
        let screen_texture_key = create_render_texture(w as usize, h as usize, key, ctx, asset_store)?;

        Ok(screen_texture_key)
    }

    #[inline]
    pub(crate) fn post_draw(&mut self, ctx: &mut Context, asset_store: &mut AssetStore) {
        let mut to_remove = Vec::new();
        let default_texture_name = String::from(EMERALD_DEFAULT_TEXTURE_NAME);

        for (key, _) in &asset_store.texture_key_map {
            if key.get_name() == default_texture_name {
                continue;
            }

            let i = Arc::strong_count(&key.0);

            if i <= MINIMUM_TEXTURE_REFERENCES {
                to_remove.push(key.clone());
            }
        }

        for key in to_remove {
            asset_store.remove_texture(ctx, key);
        }

        let (w, h) = ctx.screen_size();
        self.last_screen_size = (w as usize, h as usize);
    }

    #[inline]
    pub fn draw_world(
        &mut self,
        mut ctx: &mut Context,
        asset_store: &mut AssetStore,
        world: &mut EmeraldWorld,
    ) -> Result<(), EmeraldError> {
        let screen_size = (self.current_resolution.0 as f32, self.current_resolution.1 as f32);
        let (camera, camera_position) = get_camera_and_camera_position(world);
        let mut draw_queue = Vec::new();

        for (_id, (aseprite, position)) in world.inner.query::<(&mut Aseprite, &Position)>().iter()
        {
            aseprite.update();

            if is_in_view(&aseprite.sprite, &position, &camera, &screen_size) {
                let drawable = Drawable::Aseprite {
                    sprite: aseprite.sprite.clone(),
                    offset: aseprite.offset.clone(),
                    scale: aseprite.scale.clone(),
                    centered: aseprite.centered,
                    color: aseprite.color.clone(),
                    rotation: aseprite.rotation,
                    z_index: aseprite.z_index,
                    visible: aseprite.visible,
                };

                draw_queue.push(DrawCommand {
                    drawable,
                    position: position.clone(),
                    z_index: aseprite.z_index,
                });
            }
        }

        for (_id, (sprite, position)) in world.inner.query::<(&Sprite, &Position)>().iter() {
            if is_in_view(&sprite, &position, &camera, &screen_size) {
                let drawable = Drawable::Sprite {
                    sprite: sprite.clone(),
                };

                draw_queue.push(DrawCommand {
                    drawable,
                    position: position.clone(),
                    z_index: sprite.z_index,
                });
            }
        }

        for (_id, (color_rect, position)) in world.inner.query::<(&ColorRect, &Position)>().iter() {
            let drawable = Drawable::ColorRect {
                color_rect: color_rect.clone(),
            };

            draw_queue.push(DrawCommand {
                drawable,
                position: position.clone(),
                z_index: color_rect.z_index,
            });
        }

        for (_, (label, position)) in world.query::<(&Label, &Position)>().iter() {
            let drawable = Drawable::Label {
                label: label.clone(),
            };

            draw_queue.push(DrawCommand {
                drawable,
                position: position.clone(),
                z_index: label.z_index,
            })
        }

        draw_queue.sort_by(|a, b| a.z_index.partial_cmp(&b.z_index).unwrap());

        for draw_command in draw_queue {
            let position = {
                let mut pos = draw_command.position.clone() - camera_position;

                if camera.centered {
                    pos = pos + Position::new(screen_size.0 / 2.0, screen_size.1 / 2.0);
                }

                pos.x += camera.offset.x;
                pos.y += camera.offset.y;

                pos
            };

            match draw_command.drawable {
                Drawable::Aseprite {
                    sprite,
                    rotation,
                    offset,
                    centered,
                    visible,
                    scale,
                    color,
                    z_index,
                } => self.draw_aseprite(
                    &mut ctx, asset_store, &sprite, rotation, &offset, centered, visible, &scale, &color,
                    z_index, &position,
                ),
                Drawable::Sprite { sprite } => self.draw_sprite(&mut ctx, asset_store, &sprite, &position),
                Drawable::ColorRect { color_rect } => {
                    self.draw_color_rect(&mut ctx, asset_store, &color_rect, &position)
                }
                Drawable::Label { label } => self.draw_label(&mut ctx, asset_store, &label, &position)?,
            }
        }

        Ok(())
    }

    #[inline]
    #[cfg(feature = "physics")]
    pub fn draw_colliders(
        &mut self,
        mut ctx: &mut Context,
        asset_store: &mut AssetStore,
        world: &mut EmeraldWorld,
        collider_color: Color,
    ) {
        let screen_size = (self.current_resolution.0 as f32, self.current_resolution.1 as f32);
        let mut color_rect = ColorRect::default();
        color_rect.color = collider_color;
        let (camera, camera_position) = get_camera_and_camera_position(world);

        for (_id, body_handle) in world.inner.query::<&RigidBodyHandle>().iter() {
            if let Some(body) = world.physics_engine.bodies.get(*body_handle) {
                for collider_handle in body.colliders() {
                    if let Some(collider) =
                        world.physics_engine.colliders.get(collider_handle.clone())
                    {
                        let aabb = collider.compute_aabb();
                        let pos = Position::new(aabb.center().coords.x, aabb.center().coords.y);
                        color_rect.width = aabb.half_extents().x as u32 * 2;
                        color_rect.height = aabb.half_extents().y as u32 * 2;

                        let position = {
                            let mut pos = pos - camera_position;

                            if camera.centered {
                                pos = pos + Position::new(screen_size.0 / 2.0, screen_size.1 / 2.0);
                            }

                            pos
                        };

                        self.draw_color_rect(&mut ctx, asset_store, &color_rect, &position);
                    }
                }
            }
        }
    }

    #[inline]
    pub(crate) fn begin(&mut self, ctx: &mut Context, asset_store: &mut AssetStore) -> Result<(), EmeraldError> {
        self.current_render_texture_key = self.screen_texture_key.clone();

        if let Some(texture) = asset_store.get_texture(&self.current_render_texture_key) {
            self.current_resolution = (texture.width as usize, texture.height as usize);
        } else {
            return Err(EmeraldError::new("Unable to retrieve default rendering texture"));
        }

        self.begin_texture_pass(ctx, asset_store, self.screen_texture_key.clone())?;

        Ok(())
    }

    #[inline]
    pub(crate) fn begin_texture(&mut self, ctx: &mut Context, texture_key: TextureKey, asset_store: &mut AssetStore) -> Result<(), EmeraldError> {
        self.current_render_texture_key = texture_key.clone();

        if let Some(texture) = asset_store.get_texture(&self.current_render_texture_key) {
            self.current_resolution = (texture.width as usize, texture.height as usize);
        } else {
            return Err(EmeraldError::new(format!("Unable to retrieve texture for {:?}", texture_key)));
        }

        self.begin_texture_pass(ctx, asset_store, texture_key)?;

        Ok(())
    }

    /// This will begin a rendering pass that will render to a WxH size texture
    /// Call `render_to_texture` to retrieve the texture key for this pass.
    #[inline]
    fn begin_texture_pass(&mut self, ctx: &mut Context, asset_store: &mut AssetStore, texture_key: TextureKey) -> Result<(), EmeraldError> {
        if let Some(texture) = asset_store.get_texture(&texture_key) {
            self.render_pass = RenderPass::new(ctx, texture.inner, None);
            ctx.begin_pass(
                self.render_pass,
                PassAction::Clear {
                    color: Some(self.settings.background_color.to_percentage()),
                    depth: None,
                    stencil: None,
                }
            );

            return Ok(())
        }

        Err(EmeraldError::new(format!("Unable to retrieve texture for {:?}", texture_key)))
    }

    #[inline]
    pub(crate) fn render(&mut self, ctx: &mut Context, asset_store: &mut AssetStore) -> Result<(), EmeraldError> {
        let texture_key = self.render_texture(ctx, asset_store)?;

        ctx.begin_default_pass(PassAction::Clear {
            color: Some(self.settings.background_color.to_percentage()),
            depth: None,
            stencil: None,
        });
        let sprite = Sprite::from_texture(texture_key);
        let (w, h) = ctx.screen_size();
        let position = Position::new(w as f32 / 2.0, h as f32 / 2.0);
        self.draw_sprite(ctx, asset_store, &sprite, &position);
        ctx.end_render_pass();

        Ok(())
    }

    #[inline]
    pub(crate) fn render_texture(&mut self, ctx: &mut Context, _asset_store: &mut AssetStore) -> Result<TextureKey, EmeraldError> {
        ctx.end_render_pass();
        
        Ok(self.current_render_texture_key.clone())
    }

    pub(crate) fn draw_label(
        &mut self,
        mut ctx: &mut Context,
        mut asset_store: &mut AssetStore,
        label: &Label,
        position: &Position,
    ) -> Result<(), EmeraldError> {
        self.layout.reset(&LayoutSettings {
            max_width: label.max_width,
            max_height: label.max_height,
            wrap_style: label.wrap_style,
            horizontal_align: label.horizontal_align,
            vertical_align: label.vertical_align,
            ..LayoutSettings::default()
        });

        if let Some(font) = asset_store.get_fontdue_font(&label.font_key) {
            self.layout.append(&[font], &TextStyle::new(&label.text, label.font_size as f32, 0));
        }

        let mut font_texture_width = 0;
        let mut font_texture_height = 0;
        let mut font_texture_key: Option<TextureKey> = None;

        if let Some(font) = asset_store.get_font_mut(&label.font_key) {
            font_texture_key = Some(font.font_texture_key.clone());
        }

        if let Some(font_texture_key) = font_texture_key.as_ref() {
            if let Some(texture) = asset_store.get_texture(&font_texture_key) {
                font_texture_width = texture.width;
                font_texture_height = texture.height;
            }
        }

        let mut draw_calls: Vec<(
            f32,       // z_index
            Vec2,      // real_scale
            Vec2,      // real_position
            Rectangle, // target
            Color,     // color
            bool,      // centered
            bool,      // Visible
            Option<f32>, // max_width
        )> = Vec::new();

        ctx.apply_pipeline(&self.pipelines.get(EMERALD_TEXTURE_PIPELINE_NAME).unwrap());

        let mut remaining_char_count = label.visible_characters;
        if label.visible_characters < 0 {
            remaining_char_count = label.text.len() as i64;
        }

        for glyph in self.layout.glyphs() {
            let glyph_key = glyph.key;
            let x = glyph.x;
            let y = glyph.y;

            let mut need_to_cache_glyph = false;
            if let Some(font) = asset_store.get_font(&label.font_key) {
                need_to_cache_glyph = !font.characters.contains_key(&glyph_key);
            }
            
            if need_to_cache_glyph {
                cache_glyph(&mut ctx, &mut asset_store, &label.font_key, glyph_key.clone(), label.font_size)?;
            }

            if let Some(font) = asset_store.get_font_mut(&label.font_key) {
                let font_data = &font.characters[&glyph_key];
                {
                    let left_coord = (font_data.offset_x as f32 + x) * label.scale;
                    let top_coord = y * label.scale;

                    let target = Rectangle::new(
                        (font_data.glyph_x as f32) / font_texture_width as f32,
                        (font_data.glyph_y as f32) / font_texture_height as f32,
                        (font_data.glyph_w as f32) / font_texture_width as f32,
                        (font_data.glyph_h as f32) / font_texture_height as f32,
                    );

                    let real_scale = Vec2::new(
                        label.scale * target.width * font_texture_width as f32,
                        label.scale * target.height * font_texture_height as f32 * -1.0,
                    );
                    let real_position = Vec2::new(
                        position.x + label.offset.x + left_coord,
                        position.y + label.offset.y - top_coord,
                    );

                    if remaining_char_count > 0 {
                        draw_calls.push((
                            label.z_index,
                            real_scale,
                            real_position,
                            target,
                            label.color,
                            label.centered,
                            label.visible,
                            label.max_width.clone(),
                        ));
                    }
                }

                remaining_char_count -= 1;
            }
        }

        if let Some(font_texture_key) = font_texture_key {
            for draw_call in draw_calls {
                let (z_index, real_scale, mut real_position, target, mut color, centered, visible, max_width) = draw_call;
    
                if centered {
                    if let Some(max_width) = max_width {
                        real_position.x -= max_width / 2.0;
                    }
                }

                if !visible {
                    color.a = 0;
                }
    
                draw_texture(
                    &self.settings,
                    &mut ctx,
                    &mut asset_store,
                    &font_texture_key,
                    z_index,
                    real_scale,
                    0.0,
                    Vec2::new(0.0, 0.0),
                    real_position,
                    target,
                    color,
                    self.current_resolution
                );
            }
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn draw_color_rect(
        &mut self,
        mut ctx: &mut Context,
        mut asset_store: &mut AssetStore,
        color_rect: &ColorRect,
        position: &Position,
    ) {
        ctx.apply_pipeline(&self.pipelines.get(EMERALD_TEXTURE_PIPELINE_NAME).unwrap());

        let (width, height) = (color_rect.width, color_rect.height);
        let mut offset = color_rect.offset.clone();

        if color_rect.centered {
            offset.x -= (color_rect.width / 2) as f32;
            offset.y -= (color_rect.height / 2) as f32;
        }

        let real_scale = Vec2::new(width as f32, height as f32);
        let real_position = Vec2::new(position.x + offset.x, position.y + offset.y);

        draw_texture(
            &self.settings,
            &mut ctx,
            &mut asset_store,
            &TextureKey::default(),
            color_rect.z_index,
            real_scale,
            color_rect.rotation,
            Vec2::new(0.0, 0.0),
            real_position,
            Rectangle::new(0.0, 0.0, 1.0, 1.0),
            color_rect.color,
            self.current_resolution
        )
    }

    #[inline]
    pub(crate) fn draw_aseprite(
        &mut self,
        mut ctx: &mut Context,
        mut asset_store: &mut AssetStore,
        sprite: &Sprite,
        rotation: f32,
        offset: &Vector2<f32>,
        centered: bool,
        visible: bool,
        scale: &Vector2<f32>,
        color: &Color,
        z_index: f32,
        position: &Position,
    ) {
        if !visible {
            return;
        }

        ctx.apply_pipeline(&self.pipelines.get(EMERALD_TEXTURE_PIPELINE_NAME).unwrap());
        let texture = asset_store.get_texture(&sprite.texture_key).unwrap();
        let mut target = Rectangle::new(
            sprite.target.x / texture.width as f32,
            sprite.target.y / texture.height as f32,
            sprite.target.width / texture.width as f32,
            sprite.target.height / texture.height as f32,
        );

        if sprite.target.is_zero_sized() {
            target = Rectangle::new(0.0, 0.0, 1.0, 1.0);
        }

        let mut offset = offset.clone();
        if centered {
            if sprite.target.is_zero_sized() {
                offset.x -= scale.x * texture.width as f32 / 2.0;
                offset.y -= scale.y * texture.height as f32 / 2.0;
            } else {
                offset.x -= scale.x * sprite.target.width / 2.0;
                offset.y -= scale.y * sprite.target.height / 2.0;
            }
        }

        let real_scale = Vec2::new(
            scale.x * target.width * (f32::from(texture.width)),
            scale.y * target.height * (f32::from(texture.height)),
        );
        let real_position = Vec2::new(position.x + offset.x, position.y + offset.y);

        draw_texture(
            &self.settings,
            &mut ctx,
            &mut asset_store,
            &sprite.texture_key,
            z_index,
            real_scale,
            rotation,
            Vec2::new(0.0, 0.0),
            real_position,
            target,
            color.clone(),
            self.current_resolution
        )
    }

    #[inline]
    pub(crate) fn draw_sprite(
        &mut self,
        mut ctx: &mut Context,
        mut asset_store: &mut AssetStore,
        sprite: &Sprite,
        position: &Position,
    ) {
        if !sprite.visible {
            return;
        }

        ctx.apply_pipeline(&self.pipelines.get(EMERALD_TEXTURE_PIPELINE_NAME).unwrap());
        let texture = asset_store.get_texture(&sprite.texture_key).unwrap();
        let mut target = Rectangle::new(
            sprite.target.x / texture.width as f32,
            sprite.target.y / texture.height as f32,
            sprite.target.width / texture.width as f32,
            sprite.target.height / texture.height as f32,
        );

        if sprite.target.is_zero_sized() {
            target = Rectangle::new(0.0, 0.0, 1.0, 1.0);
        }

        let mut offset = sprite.offset.clone();
        if sprite.centered {
            if sprite.target.is_zero_sized() {
                offset.x -= sprite.scale.x * texture.width as f32 / 2.0;
                offset.y -= sprite.scale.y * texture.height as f32 / 2.0;
            } else {
                offset.x -= sprite.scale.x * sprite.target.width / 2.0;
                offset.y -= sprite.scale.y * sprite.target.height / 2.0;
            }
        }

        let real_scale = Vec2::new(
            sprite.scale.x * target.width * (f32::from(texture.width)),
            sprite.scale.y * target.height * (f32::from(texture.height)),
        );
        let real_position = Vec2::new(position.x + offset.x, position.y + offset.y);

        draw_texture(
            &self.settings,
            &mut ctx,
            &mut asset_store,
            &sprite.texture_key,
            sprite.z_index,
            real_scale,
            sprite.rotation,
            Vec2::new(0.0, 0.0),
            real_position,
            target,
            sprite.color.clone(),
            self.current_resolution
        )
    }
}

#[inline]
fn draw_texture(
    _settings: &RenderSettings,
    mut ctx: &mut Context,
    asset_store: &mut AssetStore,
    texture_key: &TextureKey,
    _z_index: f32,
    scale: Vec2,
    rotation: f32,
    offset: Vec2,
    mut position: Vec2,
    source: Rectangle,
    color: Color,
    resolution: (usize, usize),
) {
    position.x = position.x.floor() + 0.375;
    position.y = position.y.floor() + 0.375;

    let projection = Mat4::orthographic_rh_gl(
        0.0,
        resolution.0 as f32,
        0.0,
        resolution.1 as f32,
        -1.0,
        1.0,
    );

    let mut uniforms = Uniforms::default();
    uniforms.projection = projection;
    uniforms.model =
        crate::rendering::param_to_instance_transform(rotation, scale, offset, position);

    let color = color.to_percentage();
    uniforms.source = Vec4::new(source.x, source.y, source.width, source.height);
    uniforms.color = Vec4::new(color.0, color.1, color.2, color.3);

    if let Some(texture) = asset_store.get_texture(&texture_key) {
        texture.inner.set_filter(&mut ctx, texture.filter);
        ctx.apply_bindings(&texture.bindings);
        ctx.apply_uniforms(&uniforms);
        ctx.draw(0, 6, 1);
    }
}

#[inline]
fn is_in_view(
    _sprite: &Sprite,
    _pos: &Position,
    _camera: &Camera,
    _screen_size: &(f32, f32),
) -> bool {
    true
}

#[inline]
fn get_camera_and_camera_position(world: &EmeraldWorld) -> (Camera, Position) {
    let mut cam = Camera::default();
    let mut cam_position = Position::new(0.0, 0.0);
    let mut entity_holding_camera: Option<Entity> = None;

    for (id, camera) in world.query::<&Camera>().iter() {
        if camera.is_active {
            cam = camera.clone();
            entity_holding_camera = Some(id);
        }
    }

    if let Some(entity) = entity_holding_camera {
        if let Ok(position) = world.get_mut::<Position>(entity) {
            cam_position = position.clone();
        }
    }

    (cam, cam_position)
}

#[derive(Clone)]
enum Drawable {
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
}

#[derive(Clone)]
struct DrawCommand {
    pub drawable: Drawable,
    pub position: Position,
    pub z_index: f32,
}

#[inline]
pub(crate) fn create_render_texture(w: usize, h: usize, key: TextureKey, ctx: &mut Context, asset_store: &mut AssetStore) -> Result<TextureKey, EmeraldError> {
    let color_img = miniquad::Texture::new_render_texture(
        ctx,
        TextureParams {
            width: w as _,
            height: h as _,
            format: TextureFormat::RGBA8,
            wrap: TextureWrap::Clamp,
            filter: FilterMode::Nearest,
        },
    );

    let texture = crate::rendering::Texture::from_texture(ctx, key.clone(), color_img)?;
    asset_store.insert_texture(ctx, key.clone(), texture);

    Ok(key)
}