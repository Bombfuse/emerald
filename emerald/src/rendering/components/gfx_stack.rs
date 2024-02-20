use std::collections::HashMap;

use anymap::{
    any::{Any, UncheckedAnyExt},
    AnyMap,
};
use hecs::Entity;
use serde::Deserialize;

use crate::{
    ent::{
        ent_aseprite_loader::{self, load_aseprite, EntAsepriteSchema},
        ent_color_rect_loader::{load_color_rect, EntColorRectSchema},
        ent_sprite_loader::{load_sprite, EntSpriteSchema},
    },
    rendering_engine::{DrawableType, ToDrawable},
    Aseprite, AssetLoader, Emerald, EmeraldError, Rectangle, Sprite, World,
};

pub struct GraphicsStack {
    /// Map of component label to z index
    pub(crate) components: HashMap<String, Box<dyn Any + Send + Sync>>,

    /// Lets us know how to downcast the component
    pub(crate) drawable_types: HashMap<String, DrawableType>,

    pub z_index: f32,

    pub visible: bool,
}
impl GraphicsStack {
    pub fn new() -> Self {
        Self {
            z_index: 0.0,
            components: HashMap::new(),
            drawable_types: HashMap::new(),
            visible: true,
        }
    }

    pub fn add_component<T: ToDrawable + Send + Sync + 'static>(
        &mut self,
        label: &str,
        component: T,
    ) {
        self.drawable_types
            .insert(label.to_string(), component.get_type());
        self.components
            .insert(label.to_string(), Box::new(component));
    }

    pub fn set_z_index(&mut self, label: &str, new_z: f32) {
        self.components.get_mut(label).map(|component| unsafe {
            component
                .downcast_mut_unchecked::<Box<dyn ToDrawable>>()
                .set_z_index(new_z);
        });
    }

    pub fn remove_component(&mut self, label: &str) {
        self.components.remove(label);
        self.drawable_types.remove(label);
    }
}

impl ToDrawable for GraphicsStack {
    fn get_visible_bounds(
        &self,
        transform: &crate::Transform,
        asset_store: &mut crate::AssetEngine,
    ) -> Option<crate::Rectangle> {
        // TODO: get largest visible bounds of all components
        Some(Rectangle::new(0.0, 0.0, 1000.0, 1000.0))
    }

    fn get_type(&self) -> DrawableType {
        DrawableType::GfxStack
    }

    fn z_index(&self) -> f32 {
        self.z_index
    }

    fn set_z_index(&mut self, new_z_index: f32) {
        self.z_index = new_z_index;
    }
}

#[derive(Deserialize)]
struct GraphicsStackLayerSchema {
    #[serde(default)]
    aseprite: Option<EntAsepriteSchema>,
    #[serde(default)]
    sprite: Option<EntSpriteSchema>,
    #[serde(default)]
    color_rect: Option<EntColorRectSchema>,
    #[serde(default)]
    z_index: f32,
}

#[derive(Deserialize)]
struct GraphicsStackSchema {
    #[serde(default)]
    layers: HashMap<String, GraphicsStackLayerSchema>,
}

pub(crate) fn load_ent_gfx_stack<'a>(
    loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: toml::Value,
) -> Result<(), EmeraldError> {
    let schema = toml.try_into::<GraphicsStackSchema>()?;
    let mut gfx_stack = GraphicsStack::new();
    for (label, layer) in schema.layers {
        if let Some(schema) = layer.aseprite {
            let aseprite = load_aseprite(loader, schema)?;
            gfx_stack.add_component(&label, aseprite);
        };
        if let Some(schema) = layer.sprite {
            let sprite = load_sprite(loader, schema)?;
            gfx_stack.add_component(&label, sprite);
        };
        if let Some(schema) = layer.color_rect {
            let color_rect = load_color_rect(loader, schema)?;
            gfx_stack.add_component(&label, color_rect);
        };
    }

    world.insert_one(entity, gfx_stack).ok();
    Ok(())
}
