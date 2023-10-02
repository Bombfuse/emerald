use std::{
    any::TypeId,
    collections::{HashMap, VecDeque},
    hash::Hash,
    ops::Range,
};

use fontdue::layout::{GlyphRasterConfig, Layout, LayoutSettings, TextStyle};
use hecs::Entity;
use rapier2d::{na::Vector2, prelude::RigidBodyHandle};

use crate::{
    asset_key::{AssetId, AssetKey},
    autotilemap::AutoTilemap,
    font::{CharacterInfo, Font, FontImage, FontKey},
    render_settings::RenderSettings,
    tilemap::Tilemap,
    AssetEngine, Color, EmeraldError, Rectangle, Scale, Transform, Translation, UIButton, World,
    WHITE,
};

use super::components::{get_bounding_box_of_triangle, Camera, ColorRect, ColorTri, Label, Sprite};

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

#[derive(Clone, Copy, PartialEq)]
pub struct ScreenSize {
    pub width: u32,
    pub height: u32,
}

pub struct DrawTexturedQuadCommand<'a> {
    /// The area to render of the texture
    pub texture_target_area: Rectangle,
    pub asset_engine: &'a mut AssetEngine,
    pub texture_asset_id: AssetId,
    pub offset: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: f32,
    pub centered: bool,
    pub color: Color,
    pub transform: &'a Transform,
    pub current_render_target_size: ScreenSize,

    /// If the texture should snap to the nearest pixel
    pub pixel_snap: bool,

    /// If the texture should be culled if outside of view
    pub frustrum_culling: bool,
}

pub struct DrawTexturedTriCommand<'a> {
    /// The area to render of the texture
    pub texture_target_area: Rectangle,
    pub asset_engine: &'a mut AssetEngine,
    pub texture_asset_id: AssetId,
    pub offset: Vector2<f32>,
    pub scale: Vector2<f32>,
    pub rotation: f32,
    pub centered: bool,
    pub color: Color,
    pub transform: &'a Transform,
    pub current_render_target_size: ScreenSize,
}

pub trait RenderingEngine {
    fn initialize(&mut self, asset_engine: &mut AssetEngine);
    fn draw_textured_quad(&mut self, command: DrawTexturedQuadCommand) -> Result<(), EmeraldError>;
    fn draw_textured_tri(&mut self, command: DrawTexturedTriCommand) -> Result<(), EmeraldError>;
    fn current_render_target_size(&self) -> ScreenSize;
    fn screen_size(&self) -> ScreenSize;

    fn update_font_texture(
        &mut self,
        asset_store: &mut AssetEngine,
        key: &FontKey,
    ) -> Result<(), EmeraldError>;
    /// Resize the game window to the new size.
    fn resize_window(&mut self, new_size: ScreenSize);

    /// The window has been resized, the rendering engine should handle this.
    /// May be called every frame with the same value, engine should do a check to make sure its different
    /// than its current size.
    fn handle_window_resize(&mut self, screen_size: ScreenSize);

    /// Gets a copy of the texture key for the given label if it exists
    fn get_texture_key(&self, asset_engine: &mut AssetEngine, label: &str) -> Option<AssetKey>;

    #[inline]
    fn draw_world(
        &mut self,
        world: &mut World,
        asset_store: &mut AssetEngine,
    ) -> Result<(), EmeraldError> {
        self.draw_world_with_transform(world, Transform::default(), asset_store)
    }

    #[inline]
    fn draw_world_with_transform(
        &mut self,
        world: &mut World,
        transform: Transform,
        asset_store: &mut AssetEngine,
    ) -> Result<(), EmeraldError> {
        let (camera, camera_transform) = get_camera_and_camera_transform(world);
        let camera_transform = camera_transform - transform;
        let cmd_adder = DrawCommandAdder::new(world);
        let mut draw_queue = Vec::new();

        // cmd_adder.add_draw_commands::<Aseprite>(&mut draw_queue, world, asset_store);
        cmd_adder.add_draw_commands::<AutoTilemap>(&mut draw_queue, world, asset_store);
        cmd_adder.add_draw_commands::<Tilemap>(&mut draw_queue, world, asset_store);
        cmd_adder.add_draw_commands::<Sprite>(&mut draw_queue, world, asset_store);
        cmd_adder.add_draw_commands::<UIButton>(&mut draw_queue, world, asset_store);
        cmd_adder.add_draw_commands::<ColorRect>(&mut draw_queue, world, asset_store);
        cmd_adder.add_draw_commands::<Label>(&mut draw_queue, world, asset_store);
        draw_queue.sort_by(|a, b| a.z_index.partial_cmp(&b.z_index).unwrap());
        for draw_command in draw_queue {
            self.draw(asset_store, world, draw_command, &camera, &camera_transform)?;
        }

        Ok(())
    }

    #[inline]
    fn draw(
        &mut self,
        asset_engine: &mut AssetEngine,
        world: &mut World,
        draw_command: DrawCommand,
        camera: &Camera,
        camera_transform: &Transform,
    ) -> Result<(), EmeraldError> {
        let transform = {
            let entity_transform = world.get::<&mut Transform>(draw_command.entity)?.clone();
            let mut transform = entity_transform - camera_transform.clone();

            transform.translation += Translation::from(camera.offset);
            transform
        };

        match draw_command.drawable_type {
            DrawableType::Aseprite => {
                // let aseprite = world.get::<&Aseprite>(draw_command.entity)?;
                // self.draw_aseprite(asset_engine, &aseprite, &transform)?;
            }
            DrawableType::Sprite => {
                let sprite = world.get::<&Sprite>(draw_command.entity)?;
                self.draw_sprite(asset_engine, &sprite, &transform)?;
            }
            DrawableType::Tilemap => {
                let tilemap = world.get::<&Tilemap>(draw_command.entity)?;
                self.draw_tilemap(asset_engine, &tilemap, &transform)?;
            }
            DrawableType::Autotilemap => {
                let autotilemap = world.get::<&AutoTilemap>(draw_command.entity)?;
                self.draw_tilemap(asset_engine, &autotilemap.tilemap, &transform)?;
            }
            DrawableType::ColorRect => {
                let color_rect = world.get::<&ColorRect>(draw_command.entity)?;
                self.draw_color_rect(asset_engine, &color_rect, &transform)?;
            }
            DrawableType::UIButton => {
                let ui_button = world.get::<&UIButton>(draw_command.entity)?;
                self.draw_ui_button(asset_engine, &ui_button, &transform)?;
            }
            DrawableType::ColorTri => {
                let color_tri = world.get::<&ColorTri>(draw_command.entity)?;
                self.draw_color_tri(asset_engine, &color_tri, &transform)?;
            }
            DrawableType::Label => {
                let label = world.get::<&Label>(draw_command.entity)?;
                self.draw_label(asset_engine, &label, &transform)?;
            }
        }

        Ok(())
    }

    fn draw_ui_button(
        &mut self,
        asset_engine: &mut AssetEngine,
        ui_button: &UIButton,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        if !ui_button.visible {
            return Ok(());
        }

        // let texture = ui_button.current_texture();
        // let cmd = DrawTexturedQuadCommand { asset_engine };
        // self.draw_textured_quad(cmd)
        todo!()
    }

    // fn draw_aseprite(
    //     &mut self,
    //     asset_engine: &mut AssetEngine,
    //     aseprite: &Aseprite,
    //     transform: &Transform,
    // ) -> Result<(), EmeraldError> {
    //     if !aseprite.visible {
    //         return Ok(());
    //     }

    //     let sprite = aseprite.get_sprite();
    //     draw_textured_quad(cmd)
    // }

    fn draw_tilemap(
        &mut self,
        asset_engine: &mut AssetEngine,
        tilemap: &Tilemap,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        if !tilemap.visible {
            return Ok(());
        }
        // let tileset_width;
        // let tileset_height;

        // if let Some(texture) =
        //     asset_engine.get_asset::<Texture>(&tilemap.tilesheet.asset_key.asset_id)
        // {
        //     tileset_width = texture.size.width as usize / tilemap.tile_size.x;
        //     tileset_height = texture.size.height as usize / tilemap.tile_size.y;
        // } else {
        //     return Err(EmeraldError::new(format!(
        //         "Tilemap Texture {:?} not found.",
        //         &tilemap.tilesheet
        //     )));
        // }

        // let tile_width = tilemap.tile_size.x as f32;
        // let tile_height = tilemap.tile_size.y as f32;

        // let mut x = 0;
        // let mut y = 0;

        // for tile in &tilemap.tiles {
        //     if let Some(tile_id) = &tile {
        //         let tile_x = tile_id % tileset_width;
        //         let tile_y = tileset_height - 1 - tile_id / tileset_width;
        //         let target = Rectangle::new(
        //             tile_x as f32 * tile_width,
        //             tile_y as f32 * tile_height,
        //             tile_width,
        //             tile_height,
        //         );
        //         let translation = transform.translation.clone()
        //             + Translation::new(tile_width * x as f32, tile_height * y as f32);

        //         let offset = Vector2::new(0.0, 0.0);
        //         let scale = Vector2::new(1.0, 1.0);
        //         let rotation = 0.0;
        //         let centered = true;
        //         let color = crate::colors::WHITE;
        //         let transform = Transform::from_translation(translation);
        //         let active_size = self.active_size;

        //         draw_textured_quad(
        //             asset_engine,
        //             tilemap.tilesheet.asset_key.asset_id,
        //             tilemap.tilesheet.bind_group_key.asset_id,
        //             target,
        //             offset,
        //             scale,
        //             rotation,
        //             centered,
        //             color,
        //             &transform,
        //             active_size,
        //             &mut self.vertices,
        //             &mut self.indices,
        //             &mut self.draw_queue,
        //             &self.settings,
        //         )?;
        //     }

        //     x += 1;
        //     if x >= tilemap.width {
        //         x = 0;
        //         y += 1;
        //     }
        // }

        Ok(())
    }

    fn draw_sprite(
        &mut self,
        asset_engine: &mut AssetEngine,
        sprite: &Sprite,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        if !sprite.visible {
            return Ok(());
        }

        self.draw_textured_quad(DrawTexturedQuadCommand {
            texture_target_area: sprite.target.clone(),
            asset_engine,
            texture_asset_id: sprite.texture_key.asset_id(),
            offset: sprite.offset.clone(),
            scale: sprite.scale.clone(),
            rotation: sprite.rotation,
            centered: sprite.centered,
            color: sprite.color.clone(),
            transform,
            current_render_target_size: self.current_render_target_size(),
            pixel_snap: true,
            frustrum_culling: true,
        })
    }

    fn draw_color_rect(
        &mut self,
        asset_engine: &mut AssetEngine,
        color_rect: &ColorRect,
        transform: &Transform,
    ) -> Result<(), EmeraldError>;

    fn draw_color_tri(
        &mut self,
        asset_engine: &mut AssetEngine,
        color_tri: &ColorTri,
        transform: &Transform,
    ) -> Result<(), EmeraldError>;

    /// Begin a render pass
    fn begin(&mut self, _asset_store: &mut AssetEngine) -> Result<(), EmeraldError>;

    /// Begin a render pass to the given texture
    fn begin_texture(
        &mut self,
        texture_key: &AssetKey,
        asset_engine: &mut AssetEngine,
    ) -> Result<(), EmeraldError>;

    /// Render the texture draw pass
    fn render_texture(&mut self, asset_store: &mut AssetEngine) -> Result<(), EmeraldError>;

    /// Load a texture
    fn load_texture(
        &mut self,
        label: &str,
        asset_engine: &mut AssetEngine,
        data: &[u8],
    ) -> Result<AssetKey, EmeraldError>;

    /// Load a texture with the specified data
    fn load_texture_ext(
        &mut self,
        label: &str,
        asset_store: &mut AssetEngine,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> Result<AssetKey, EmeraldError>;

    /// Render to the screen surface
    fn render(&mut self, asset_store: &mut AssetEngine) -> Result<(), EmeraldError>;

    fn create_render_texture(
        &mut self,
        width: u32,
        height: u32,
        asset_store: &mut AssetEngine,
    ) -> Result<AssetKey, EmeraldError>;

    fn layout(&self) -> &Layout;
    fn layout_mut(&mut self) -> &mut Layout;

    #[inline]
    fn draw_label(
        &mut self,
        asset_engine: &mut AssetEngine,
        label: &Label,
        transform: &Transform,
    ) -> Result<(), EmeraldError> {
        if !label.visible {
            return Ok(());
        }

        self.layout_mut().reset(&Default::default());

        let mut to_cache = Vec::new();
        asset_engine
            .get_asset::<Font>(&label.font_key.asset_key.asset_id)
            .map(|font| {
                self.layout_mut().append(
                    &[&font.inner],
                    &TextStyle::new(&label.text, label.font_size as f32, 0),
                );
                for glyph in self.layout().glyphs() {
                    if !font.characters.contains_key(&glyph.key) {
                        to_cache.push(glyph.key);
                    }
                }
            });

        for glyph_key in to_cache {
            self.cache_glyph(asset_engine, &label.font_key, glyph_key, label.font_size)?;
        }

        self.layout_mut().reset(&LayoutSettings {
            max_width: label.max_width,
            max_height: label.max_height,
            wrap_style: label.wrap_style,
            horizontal_align: label.horizontal_align,
            vertical_align: label.vertical_align,
            ..LayoutSettings::default()
        });

        if let Some(font) = asset_engine.get_asset::<Font>(&label.font_key.asset_key.asset_id) {
            self.layout_mut().append(
                &[&font.inner],
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
        for glyph in self.layout().glyphs() {
            let glyph_key = glyph.key;
            let x = glyph.x;
            let y = glyph.y;

            if let Some(font) =
                asset_engine.get_asset_mut::<Font>(&label.font_key.asset_key.asset_id)
            {
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

                let mut transform = transform.clone();
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
                    font.font_texture_key.asset_id,
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
        for (texture_asset_id, target, offset, scale, rotation, transform) in to_draw {
            self.draw_textured_quad(DrawTexturedQuadCommand {
                texture_target_area: target,
                asset_engine,
                texture_asset_id,
                offset,
                scale,
                rotation,
                centered: false,
                color: WHITE,
                transform: &transform,
                current_render_target_size: self.current_render_target_size(),
                pixel_snap: true,
                frustrum_culling: true,
            })?;
        }

        Ok(())
    }

    fn cache_glyph(
        &mut self,
        asset_engine: &mut AssetEngine,
        font_key: &FontKey,
        glyph_key: GlyphRasterConfig,
        size: u16,
    ) -> Result<(), EmeraldError> {
        // let mut recache_characters = None;
        let mut update_font_texture = false;

        let mut optional_metrics = None;
        let mut optional_bitmap = None;
        let mut recache_characters = None;

        let mut to_update = Vec::new();

        if let Some(font) = asset_engine.get_asset::<Font>(&font_key.asset_key.asset_id) {
            let (metrics, bitmap) = font.inner.rasterize_config(glyph_key);
            optional_metrics = Some(metrics);
            optional_bitmap = Some(bitmap);
        } else {
            return Err(EmeraldError::new(format!(
                "Unable to get Fontdue Font while caching font glyph: {:?}",
                font_key
            )));
        }

        if let (Some(metrics), Some(bitmap)) = (optional_metrics, optional_bitmap) {
            if let Some(font) = asset_engine.get_asset_mut::<Font>(&font_key.asset_key.asset_id) {
                if metrics.advance_height != 0.0 {
                    return Err(EmeraldError::new("Vertical fonts are not supported"));
                }

                let (width, height) = (metrics.width, metrics.height);
                let advance = metrics.advance_width;
                let (offset_x, offset_y) = (metrics.xmin, metrics.ymin);

                let x = if font.cursor_x + (width as u16) < font.font_image.width {
                    if height as u16 > font.max_line_height {
                        font.max_line_height = height as u16;
                    }
                    let res = font.cursor_x;
                    font.cursor_x += width as u16 + Font::GAP;
                    res
                } else {
                    font.cursor_y += font.max_line_height + Font::GAP;
                    font.cursor_x = width as u16 + Font::GAP;
                    font.max_line_height = height as u16;
                    Font::GAP
                };

                let y = font.cursor_y;

                let character_info = CharacterInfo {
                    glyph_x: x as _,
                    glyph_y: y as _,
                    glyph_w: width as _,
                    glyph_h: height as _,

                    _advance: advance,
                    offset_x,
                    _offset_y: offset_y,
                };

                font.characters.insert(glyph_key, character_info);

                // texture bounds exceeded
                if font.cursor_y + height as u16 > font.font_image.height {
                    // reset glyph asset_store state
                    let characters = font.characters.drain().collect::<Vec<_>>();
                    font.cursor_x = 0;
                    font.cursor_y = 0;
                    font.max_line_height = 0;

                    // increase font texture size
                    font.font_image = FontImage::gen_image_color(
                        font.font_image.width * 2,
                        font.font_image.height * 2,
                        Color::new(0, 0, 0, 0),
                    );

                    to_update.push((font.font_image.bytes.clone(), font_key.path.clone()));
                    recache_characters = Some(characters);
                } else {
                    for j in 0..height {
                        for i in 0..width {
                            let coverage = bitmap[j * width + i];
                            font.font_image.set_pixel(
                                x as u32 + i as u32,
                                y as u32 + j as u32,
                                Color::new(255, 255, 255, coverage),
                            );
                        }
                    }

                    update_font_texture = true;
                }
            } else {
                return Err(EmeraldError::new(format!(
                    "Unable to get Font while caching font glyph: {:?}",
                    font_key
                )));
            }
        } else {
            return Err(EmeraldError::new(format!(
                "Unable to get Metrics while caching font glyph: {:?}",
                font_key
            )));
        }

        for (bytes, label) in to_update {
            self.load_texture(label.as_str(), asset_engine, bytes.as_slice())?;
        }

        if update_font_texture {
            self.update_font_texture(asset_engine, font_key)?;
        }

        if let Some(characters) = recache_characters {
            // recache all previously asset_stored symbols
            for (glyph_key, _) in characters {
                self.cache_glyph(asset_engine, font_key, glyph_key, size)?;
            }
        }

        Ok(())
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

trait ToDrawable {
    /// Returns a rectangle representing the visual size of this drawable, if a
    /// culling check should be performed. `None` can be returned to skip the
    /// culling check.
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        asset_store: &mut AssetEngine,
    ) -> Option<Rectangle>;

    fn get_type(&self) -> DrawableType;

    fn z_index(&self) -> f32;
}

impl ToDrawable for Tilemap {
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        _asset_store: &mut AssetEngine,
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

    fn get_type(&self) -> DrawableType {
        DrawableType::Tilemap
    }

    fn z_index(&self) -> f32 {
        self.z_index
    }
}
impl ToDrawable for AutoTilemap {
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        asset_store: &mut AssetEngine,
    ) -> Option<Rectangle> {
        self.tilemap.get_visible_bounds(transform, asset_store)
    }

    fn get_type(&self) -> DrawableType {
        DrawableType::Autotilemap
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
        asset_store: &mut AssetEngine,
    ) -> Option<Rectangle> {
        let mut sprite = self.get_sprite().clone();
        sprite.offset = self.offset.clone();

        sprite.get_visible_bounds(transform, asset_store)
    }

    // fn to_drawable(&self, _ctx: &DrawableContext) -> Drawable {
    //     Drawable::Aseprite {
    //         sprite: self.get_sprite().clone(),
    //         offset: self.offset,
    //         scale: self.scale,
    //         centered: self.centered,
    //         color: self.color,
    //         rotation: self.rotation,
    //         z_index: self.z_index,
    //         visible: self.visible,
    //     }
    // }

    fn z_index(&self) -> f32 {
        self.z_index
    }

    fn get_type(&self) -> DrawableType {
        DrawableType::Aseprite
    }
}

impl ToDrawable for Sprite {
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        asset_store: &mut AssetEngine,
    ) -> Option<Rectangle> {
        todo!()
        // let mut bounds = self.target.clone();

        // if bounds.is_zero_sized() {
        //     if let Some(texture) =
        //         asset_store.get_asset::<Texture>(&self.texture_key.asset_key.asset_id)
        //     {
        //         bounds.width = texture.size.width as f32;
        //         bounds.height = texture.size.height as f32;
        //     }
        // }

        // // Set the visibility rect at the position of the sprite
        // bounds.x = transform.translation.x + self.offset.x;
        // bounds.y = transform.translation.y + self.offset.y;

        // // Take the sprite's scale factor into account
        // bounds.width *= self.scale.x;
        // bounds.height *= self.scale.y;

        // if self.centered {
        //     bounds.x -= bounds.width as f32 / 2.0;
        //     bounds.y -= bounds.height as f32 / 2.0;
        // }

        // Some(bounds)
    }

    fn z_index(&self) -> f32 {
        self.z_index
    }

    fn get_type(&self) -> DrawableType {
        DrawableType::Sprite
    }
}

impl ToDrawable for UIButton {
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        asset_store: &mut AssetEngine,
    ) -> Option<Rectangle> {
        let sprite = Sprite::from_texture(self.current_texture().clone());
        sprite.get_visible_bounds(transform, asset_store)
    }

    // fn to_drawable(&self, _ctx: &DrawableContext) -> Drawable {
    //     let mut sprite = Sprite::from_texture(self.current_texture().clone());
    //     sprite.visible = self.visible;

    //     Drawable::Sprite { sprite }
    // }

    fn z_index(&self) -> f32 {
        self.z_index
    }

    fn get_type(&self) -> DrawableType {
        DrawableType::UIButton
    }
}

impl ToDrawable for ColorRect {
    fn get_visible_bounds(
        &self,
        transform: &Transform,
        _asset_store: &mut AssetEngine,
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

    fn z_index(&self) -> f32 {
        self.z_index
    }

    fn get_type(&self) -> DrawableType {
        DrawableType::ColorRect
    }
}

impl ToDrawable for Label {
    fn get_visible_bounds(
        &self,
        _transform: &Transform,
        _asset_store: &mut AssetEngine,
    ) -> Option<Rectangle> {
        None
    }

    fn z_index(&self) -> f32 {
        self.z_index
    }

    fn get_type(&self) -> DrawableType {
        DrawableType::Label
    }
}

pub enum DrawableType {
    Aseprite,
    Sprite,
    Tilemap,
    Autotilemap,
    ColorRect,
    UIButton,
    ColorTri,
    Label,
}

pub struct DrawCommand {
    pub drawable_type: DrawableType,
    pub entity: Entity,
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
    fn new(world: &World) -> Self {
        let (camera, camera_transform) = get_camera_and_camera_transform(world);
        // let camera_bounds = if engine.settings.frustrum_culling {
        //     let screen_size = (
        //         engine.active_size.width as f32,
        //         engine.active_size.height as f32,
        //     );

        //     let mut camera_view_region = Rectangle::new(
        //         camera_transform.translation.x - screen_size.0 / 2.0,
        //         camera_transform.translation.y - screen_size.1 / 2.0,
        //         screen_size.0,
        //         screen_size.1,
        //     );
        //     camera_view_region.width *= camera.zoom;
        //     camera_view_region.height *= camera.zoom;

        //     Some(camera_view_region)
        // } else {
        //     None
        // };
        let camera_bounds = None;
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
        asset_store: &mut AssetEngine,
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
                .map(|(entity, (to_drawable, transform))| DrawCommand {
                    drawable_type: to_drawable.get_type(),
                    entity,
                    z_index: to_drawable.z_index(),
                }),
        );
    }
}
