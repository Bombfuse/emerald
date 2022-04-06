use crate::rendering::*;
use crate::transform::Transform;
use crate::world::*;
use crate::*;
use crate::{rendering::components::*, transform::Translation};

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use glam::{Mat4, Vec2, Vec4};
use miniquad::*;
use std::collections::{HashMap, VecDeque};

const EMERALD_TEXTURE_PIPELINE_NAME: &str = "emerald_default_texture_pipline";

// The default "screen" pass.
// Renders to a texture the size of the screen when rendering begins.
const EMERALD_DEFAULT_RENDER_TARGET: &str = "emerald_default_render_target";

pub(crate) struct RenderingEngine {
    pub(crate) settings: RenderSettings,
    pipelines: HashMap<String, Pipeline>,
    layout: Layout,
    render_texture_counter: usize,
    last_screen_size: (usize, usize),
    screen_texture_key: TextureKey,
    render_passes: HashMap<TextureKey, RenderPass>,
    current_render_texture_key: TextureKey,
    current_resolution: (usize, usize),

    draw_queue: VecDeque<DrawCommand>,
}
impl RenderingEngine {
    pub(crate) fn new(
        ctx: &mut Context,
        settings: RenderSettings,
        asset_store: &mut AssetStore,
    ) -> Self {
        let mut pipelines = HashMap::new();

        let shader = Shader::new(ctx, VERTEX, FRAGMENT, shaders::meta()).unwrap();
        let params = PipelineParams {
            depth_write: true,
            color_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Value(BlendValue::SourceAlpha),
                BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
            )),
            alpha_blend: Some(BlendState::new(
                Equation::Add,
                BlendFactor::Zero,
                BlendFactor::One,
            )),
            ..Default::default()
        };

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
        let screen_texture_key =
            create_render_texture(w as usize, h as usize, key, ctx, asset_store).unwrap();
        render_texture_counter += 1;

        let texture = asset_store.get_texture(&screen_texture_key).unwrap();
        let current_render_texture_key = screen_texture_key.clone();
        let mut render_passes = HashMap::new();
        render_passes.insert(
            screen_texture_key.clone(),
            RenderPass::new(ctx, texture.inner, None),
        );
        let current_resolution = (w as usize, h as usize);

        RenderingEngine {
            settings,
            pipelines,
            layout: Layout::new(CoordinateSystem::PositiveYDown),
            render_texture_counter,
            render_passes,
            last_screen_size: current_resolution,
            screen_texture_key,
            current_render_texture_key,
            current_resolution,
            draw_queue: VecDeque::new(),
        }
    }

    #[inline]
    pub(crate) fn create_render_texture(
        &mut self,
        w: usize,
        h: usize,
        ctx: &mut Context,
        asset_store: &mut AssetStore,
    ) -> Result<TextureKey, EmeraldError> {
        self.render_texture_counter += 1;
        let key = TextureKey::new(format!(
            "emd_render_texture_{}",
            self.render_texture_counter
        ));

        create_render_texture(w, h, key, ctx, asset_store)
    }

    #[inline]
    pub(crate) fn pre_draw(
        &mut self,
        ctx: &mut Context,
        asset_store: &mut AssetStore,
    ) -> Result<(), EmeraldError> {
        let (w, h) = ctx.screen_size();
        let (prev_w, prev_h) = self.last_screen_size;

        if w as usize != prev_w || h as usize != prev_h {
            self.update_screen_texture_size(ctx, w as usize, h as usize, asset_store)?;
        }

        Ok(())
    }

    #[inline]
    fn update_screen_texture_size(
        &mut self,
        ctx: &mut Context,
        w: usize,
        h: usize,
        asset_store: &mut AssetStore,
    ) -> Result<TextureKey, EmeraldError> {
        let key = TextureKey::new(String::from(EMERALD_DEFAULT_RENDER_TARGET));

        if let Some(render_pass) = self.render_passes.get_mut(&key) {
            render_pass.delete(ctx);
            self.render_passes.remove(&key);
        }

        let screen_texture_key =
            create_render_texture(w as usize, h as usize, key, ctx, asset_store)?;

        Ok(screen_texture_key)
    }

    #[inline]
    pub(crate) fn post_draw(&mut self, ctx: &mut Context, _asset_store: &mut AssetStore) {
        let (w, h) = ctx.screen_size();
        self.last_screen_size = (w as usize, h as usize);
    }

    #[inline]
    pub fn draw_world(&mut self, world: &mut World) -> Result<(), EmeraldError> {
        let screen_size = (
            self.current_resolution.0 as f32,
            self.current_resolution.1 as f32,
        );
        let (camera, camera_transform) = get_camera_and_camera_transform(world);
        let mut draw_queue = Vec::new();

        #[cfg(feature = "aseprite")]
        for (_id, (aseprite, transform)) in
            world.inner.query::<(&mut Aseprite, &Transform)>().iter()
        {
            aseprite.update();

            if is_sprite_in_view(
                &self.settings,
                &aseprite.sprite,
                transform,
                &camera,
                &camera_transform,
                &screen_size,
            ) {
                let drawable = Drawable::Aseprite {
                    sprite: aseprite.sprite.clone(),
                    offset: aseprite.offset,
                    scale: aseprite.scale,
                    centered: aseprite.centered,
                    color: aseprite.color,
                    rotation: aseprite.rotation,
                    z_index: aseprite.z_index,
                    visible: aseprite.visible,
                };

                draw_queue.push(DrawCommand {
                    drawable,
                    transform: *transform,
                    z_index: aseprite.z_index,
                });
            }
        }

        for (_id, (sprite, transform)) in world.inner.query::<(&Sprite, &Transform)>().iter() {
            if is_sprite_in_view(
                &self.settings,
                sprite,
                transform,
                &camera,
                &camera_transform,
                &screen_size,
            ) {
                let drawable = Drawable::Sprite {
                    sprite: sprite.clone(),
                };

                draw_queue.push(DrawCommand {
                    drawable,
                    transform: *transform,
                    z_index: sprite.z_index,
                });
            }
        }

        for (_id, (ui_button, transform)) in world.inner.query::<(&UIButton, &Transform)>().iter() {
            let sprite = if ui_button.is_pressed() {
                Sprite::from_texture(ui_button.pressed_texture.clone())
            } else {
                Sprite::from_texture(ui_button.unpressed_texture.clone())
            };

            if is_sprite_in_view(
                &self.settings,
                &sprite,
                transform,
                &camera,
                &camera_transform,
                &screen_size,
            ) {
                let drawable = Drawable::Sprite {
                    sprite: sprite.clone(),
                };

                draw_queue.push(DrawCommand {
                    drawable,
                    transform: *transform,
                    z_index: ui_button.z_index,
                });
            }
        }

        for (_id, (color_rect, transform)) in world.inner.query::<(&ColorRect, &Transform)>().iter()
        {
            if is_color_rect_in_view(
                &self.settings,
                &color_rect,
                &transform,
                &camera,
                &camera_transform,
                &screen_size,
            ) {
                let drawable = Drawable::ColorRect {
                    color_rect: *color_rect,
                };

                draw_queue.push(DrawCommand {
                    drawable,
                    transform: *transform,
                    z_index: color_rect.z_index,
                });
            }
        }

        for (_, (label, transform)) in world.query::<(&Label, &Transform)>().iter() {
            let drawable = Drawable::Label {
                label: label.clone(),
            };

            draw_queue.push(DrawCommand {
                drawable,
                transform: *transform,
                z_index: label.z_index,
            })
        }

        draw_queue.sort_by(|a, b| a.z_index.partial_cmp(&b.z_index).unwrap());

        for mut draw_command in draw_queue {
            let translation = {
                let mut translation =
                    draw_command.transform.translation - camera_transform.translation;

                if camera.centered {
                    translation =
                        translation + Translation::new(screen_size.0 / 2.0, screen_size.1 / 2.0);
                }

                translation.x += camera.offset.x;
                translation.y += camera.offset.y;

                translation
            };

            draw_command.transform.translation = translation;
            self.push_draw_command(draw_command)?;
        }

        Ok(())
    }

    #[inline]
    #[cfg(feature = "physics")]
    pub fn draw_colliders(
        &mut self,
        world: &mut World,
        collider_color: Color,
    ) -> Result<(), EmeraldError> {
        let screen_size = (
            self.current_resolution.0 as f32,
            self.current_resolution.1 as f32,
        );
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
                            Translation::new(aabb.center().coords.x, aabb.center().coords.y);
                        color_rect.width = aabb.half_extents().x as u32 * 2;
                        color_rect.height = aabb.half_extents().y as u32 * 2;

                        let translation = {
                            let mut translation = body_translation - camera_transform.translation;

                            if camera.centered {
                                translation = translation
                                    + Translation::new(screen_size.0 / 2.0, screen_size.1 / 2.0);
                            }

                            translation
                        };

                        self.push_draw_command(DrawCommand {
                            drawable: Drawable::ColorRect { color_rect },
                            transform: Transform::from_translation(translation),
                            z_index: 0.0,
                        })?;
                    }
                }
            }
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn begin(
        &mut self,
        ctx: &mut Context,
        asset_store: &mut AssetStore,
    ) -> Result<(), EmeraldError> {
        self.current_render_texture_key = self.screen_texture_key.clone();

        if let Some(texture) = asset_store.get_texture(&self.current_render_texture_key) {
            self.current_resolution = (texture.width as usize, texture.height as usize);
        } else {
            return Err(EmeraldError::new(
                "Unable to retrieve default rendering texture",
            ));
        }

        self.begin_texture_pass(ctx, asset_store, self.current_render_texture_key.clone())?;

        Ok(())
    }

    #[inline]
    pub(crate) fn begin_texture(
        &mut self,
        ctx: &mut Context,
        texture_key: TextureKey,
        asset_store: &mut AssetStore,
    ) -> Result<(), EmeraldError> {
        self.current_render_texture_key = texture_key.clone();

        if let Some(texture) = asset_store.get_texture(&self.current_render_texture_key) {
            self.current_resolution = (texture.width as usize, texture.height as usize);
        } else {
            return Err(EmeraldError::new(format!(
                "Unable to retrieve texture for {:?}",
                texture_key
            )));
        }

        self.begin_texture_pass(ctx, asset_store, texture_key)?;

        Ok(())
    }

    /// This will begin a rendering pass that will render to a WxH size texture
    /// Call `render_to_texture` to retrieve the texture key for this pass.
    #[inline]
    fn begin_texture_pass(
        &mut self,
        ctx: &mut Context,
        asset_store: &mut AssetStore,
        texture_key: TextureKey,
    ) -> Result<(), EmeraldError> {
        if let Some(texture) = asset_store.get_texture(&texture_key) {
            if !self.render_passes.contains_key(&texture_key) {
                self.render_passes.insert(
                    texture_key.clone(),
                    RenderPass::new(ctx, texture.inner, None),
                );
            }
        } else {
            return Err(EmeraldError::new(format!(
                "Unable to retrieve texture for {:?}",
                texture_key
            )));
        }

        if let Some(render_pass) = self.render_passes.get(&texture_key) {
            ctx.begin_pass(
                *render_pass,
                PassAction::Clear {
                    color: Some(self.settings.background_color.to_percentage()),
                    depth: None,
                    stencil: None,
                },
            );

            return Ok(());
        }

        Err(EmeraldError::new(format!(
            "Unable to retrieve render pass for {:?}",
            texture_key
        )))
    }

    #[inline]
    pub(crate) fn render(
        &mut self,
        ctx: &mut Context,
        asset_store: &mut AssetStore,
    ) -> Result<(), EmeraldError> {
        self.consume_draw_queue(ctx, asset_store)?;

        let texture_key = self.render_texture(ctx, asset_store)?;

        ctx.begin_default_pass(PassAction::Clear {
            color: Some(self.settings.background_color.to_percentage()),
            depth: None,
            stencil: None,
        });
        let sprite = Sprite::from_texture(texture_key);
        let (w, h) = ctx.screen_size();
        let translation = Translation::new(w as f32 / 2.0, h as f32 / 2.0);
        self.draw_sprite(ctx, asset_store, &sprite, &translation);
        ctx.end_render_pass();

        Ok(())
    }

    #[inline]
    pub(crate) fn render_texture(
        &mut self,
        ctx: &mut Context,
        asset_store: &mut AssetStore,
    ) -> Result<TextureKey, EmeraldError> {
        self.consume_draw_queue(ctx, asset_store)?;
        ctx.end_render_pass();

        Ok(self.current_render_texture_key.clone())
    }

    #[inline]
    pub fn push_draw_command(&mut self, draw_command: DrawCommand) -> Result<(), EmeraldError> {
        self.draw_queue.push_front(draw_command);

        Ok(())
    }

    #[inline]
    fn consume_draw_queue(
        &mut self,
        ctx: &mut Context,
        asset_store: &mut AssetStore,
    ) -> Result<(), EmeraldError> {
        ctx.apply_pipeline(self.pipelines.get(EMERALD_TEXTURE_PIPELINE_NAME).unwrap());

        while let Some(draw_command) = self.draw_queue.pop_back() {
            let translation = draw_command.transform.translation;

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
                    ctx,
                    asset_store,
                    &sprite,
                    rotation,
                    &offset,
                    centered,
                    visible,
                    &scale,
                    &color,
                    z_index,
                    &translation,
                ),
                Drawable::Sprite { sprite } => {
                    self.draw_sprite(ctx, asset_store, &sprite, &translation)
                }
                Drawable::ColorRect { color_rect } => {
                    self.draw_color_rect(ctx, asset_store, &color_rect, &translation)
                }
                Drawable::Label { label } => {
                    self.draw_label(ctx, asset_store, &label, &translation)?
                }
            }
        }
        Ok(())
    }

    pub(crate) fn draw_label(
        &mut self,
        mut ctx: &mut Context,
        mut asset_store: &mut AssetStore,
        label: &Label,
        position: &Translation,
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
            self.layout.append(
                &[font],
                &TextStyle::new(&label.text, label.font_size as f32, 0),
            );
        }

        let mut font_texture_width = 0;
        let mut font_texture_height = 0;
        let mut font_texture_key: Option<TextureKey> = None;

        if let Some(font) = asset_store.get_font_mut(&label.font_key) {
            font_texture_key = Some(font.font_texture_key.clone());
        }

        if let Some(font_texture_key) = font_texture_key.as_ref() {
            if let Some(texture) = asset_store.get_texture(font_texture_key) {
                font_texture_width = texture.width;
                font_texture_height = texture.height;
            }
        }

        let mut draw_calls: Vec<(
            f32,         // z_index
            Vec2,        // real_scale
            Vec2,        // real_position
            Rectangle,   // target
            Color,       // color
            bool,        // centered
            bool,        // Visible
            Option<f32>, // max_width
        )> = Vec::new();

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
                cache_glyph(
                    &mut ctx,
                    &mut asset_store,
                    &label.font_key,
                    glyph_key,
                    label.font_size,
                )?;
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
                            label.max_width,
                        ));
                    }
                }

                remaining_char_count -= 1;
            }
        }

        if let Some(font_texture_key) = font_texture_key {
            for draw_call in draw_calls {
                let (
                    z_index,
                    real_scale,
                    mut real_position,
                    target,
                    mut color,
                    centered,
                    visible,
                    max_width,
                ) = draw_call;

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
                    self.current_resolution,
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
        translation: &Translation,
    ) {
        let (width, height) = (color_rect.width, color_rect.height);
        let mut offset = color_rect.offset;

        if color_rect.centered {
            offset.x -= (color_rect.width / 2) as f32;
            offset.y -= (color_rect.height / 2) as f32;
        }

        let real_scale = Vec2::new(width as f32, height as f32);
        let real_position = Vec2::new(translation.x + offset.x, translation.y + offset.y);

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
            self.current_resolution,
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
        position: &Translation,
    ) {
        if !visible {
            return;
        }

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

        let mut offset = *offset;
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
            *color,
            self.current_resolution,
        )
    }

    #[inline]
    pub(crate) fn draw_sprite(
        &mut self,
        mut ctx: &mut Context,
        mut asset_store: &mut AssetStore,
        sprite: &Sprite,
        position: &Translation,
    ) {
        if !sprite.visible {
            return;
        }

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

        let mut offset = sprite.offset;
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
            sprite.color,
            self.current_resolution,
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

    let mut uniforms = Uniforms {
        projection,
        model: crate::rendering::param_to_instance_transform(rotation, scale, offset, position),
        ..Default::default()
    };

    let color = color.to_percentage();
    uniforms.source = Vec4::new(source.x, source.y, source.width, source.height);
    uniforms.color = Vec4::new(color.0, color.1, color.2, color.3);

    if let Some(texture) = asset_store.get_texture(texture_key) {
        texture.inner.set_filter(&mut ctx, texture.filter);
        ctx.apply_bindings(&texture.bindings);
        ctx.apply_uniforms(&uniforms);
        ctx.draw(0, 6, 1);
    }
}

#[inline]
fn get_camera_view_region(
    camera: &Camera,
    camera_transform: &Transform,
    screen_size: &(f32, f32),
) -> Rectangle {
    let (screen_width, screen_height) = *screen_size;
    let mut region = Rectangle::new(
        camera_transform.translation.x - screen_width / 2.0,
        camera_transform.translation.y - screen_height / 2.0,
        screen_width,
        screen_height,
    );
    // When zoom is higher, then the area that we can display is smaller.
    region.width /= camera.zoom;
    region.height /= camera.zoom;
    region
}

#[inline]
fn is_rect_in_view_region(
    rect: Rectangle,
    camera: &Camera,
    camera_transform: &Transform,
    screen_size: &(f32, f32),
) -> bool {
    get_camera_view_region(camera, camera_transform, screen_size).intersects_with(&rect)
}

#[inline]
fn is_color_rect_in_view(
    settings: &RenderSettings,
    color_rect: &ColorRect,
    color_rect_transform: &Transform,
    camera: &Camera,
    camera_transform: &Transform,
    screen_size: &(f32, f32),
) -> bool {
    // No need to do culling checks
    if !settings.frustrum_culling {
        return true;
    }

    // Build a rectangle representing the visual size of the sprite
    let mut color_rect_visible_bounds = Rectangle::new(
        color_rect_transform.translation.x,
        color_rect_transform.translation.y,
        color_rect.width as f32,
        color_rect.height as f32,
    );
    if color_rect.centered {
        color_rect_visible_bounds.x -= color_rect.width as f32 / 2.0;
        color_rect_visible_bounds.y -= color_rect.height as f32 / 2.0;
    }

    is_rect_in_view_region(
        color_rect_visible_bounds,
        camera,
        camera_transform,
        screen_size,
    )
}

#[inline]
fn is_sprite_in_view(
    settings: &RenderSettings,
    sprite: &Sprite,
    sprite_transform: &Transform,
    camera: &Camera,
    camera_transform: &Transform,
    screen_size: &(f32, f32),
) -> bool {
    // No need to do culling checks
    if !settings.frustrum_culling {
        return true;
    }

    // Build a rectangle representing the visual size of the sprite
    let mut sprite_visible_bounds = sprite.target.clone();
    // Set the visibility rect at the position of the sprite
    sprite_visible_bounds.x = sprite_transform.translation.x;
    sprite_visible_bounds.y = sprite_transform.translation.y;

    if sprite.centered {
        sprite_visible_bounds.x -= sprite.target.width as f32 / 2.0;
        sprite_visible_bounds.y -= sprite.target.height as f32 / 2.0;
    }

    // Take the sprite's scale factor into account
    sprite_visible_bounds.width *= sprite.scale.x;
    sprite_visible_bounds.height *= sprite.scale.y;

    is_rect_in_view_region(sprite_visible_bounds, camera, camera_transform, screen_size)
}

#[inline]
fn get_camera_and_camera_transform(world: &World) -> (Camera, Transform) {
    world
        .query::<(&Camera, Option<&Transform>)>()
        .iter()
        .find_map(|(_entity, (camera, transform))| {
            if camera.is_active {
                Some((*camera, transform.copied().unwrap_or_default()))
            } else {
                None
            }
        })
        .unwrap_or_default()
}

#[derive(Clone)]
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
}

#[derive(Clone)]
pub(crate) struct DrawCommand {
    pub drawable: Drawable,
    pub transform: Transform,
    pub z_index: f32,
}

#[inline]
pub(crate) fn create_render_texture(
    w: usize,
    h: usize,
    key: TextureKey,
    ctx: &mut Context,
    asset_store: &mut AssetStore,
) -> Result<TextureKey, EmeraldError> {
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
    asset_store.insert_texture(key.clone(), texture);

    Ok(key)
}
