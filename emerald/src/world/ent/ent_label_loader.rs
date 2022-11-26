use fontdue::layout::{HorizontalAlign, VerticalAlign};
use hecs::Entity;
use rapier2d::na::Vector2;
use serde::{Deserialize, Serialize};

use crate::{rendering::components::Label, AssetLoader, EmeraldError, World};

use super::Vec2f32Schema;

#[derive(Deserialize, Serialize)]
pub(crate) struct EntLabelSchema {
    pub font: String,
    pub font_size: u32,
    pub size: u16,
    pub text: Option<String>,
    pub offset: Option<Vec2f32Schema>,
    /// options: "bottom", "middle", "top"
    pub vertical_align: Option<String>,

    /// options: "center", "left", "right"
    pub horizontal_align: Option<String>,
}

pub(crate) fn load_ent_label<'a>(
    loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: &toml::Value,
) -> Result<(), EmeraldError> {
    if !toml.is_table() {
        return Err(EmeraldError::new(
            "Cannot load label from a non-table toml value.",
        ));
    }

    let schema: EntLabelSchema = toml::from_str(&toml.to_string())?;
    let font = loader.font(schema.font, schema.font_size)?;
    let text = schema.text.unwrap_or("".into());
    let mut label = Label::new(text, font, schema.size);

    if let Some(offset) = schema.offset {
        label.offset = Vector2::new(offset.x, offset.y);
    }

    if let Some(vertical_align) = schema.vertical_align {
        match vertical_align.as_str() {
            "bottom" => {
                label.vertical_align = VerticalAlign::Bottom;
            }
            "middle" => {
                label.vertical_align = VerticalAlign::Middle;
            }
            "top" => {
                label.vertical_align = VerticalAlign::Top;
            }
            _ => {
                return Err(EmeraldError::new(format!(
                    "{:?} is not a valid vertical align value",
                    vertical_align
                )))
            }
        }
    }

    if let Some(horizontal_align) = schema.horizontal_align {
        match horizontal_align.as_str() {
            "center" => {
                label.horizontal_align = HorizontalAlign::Center;
            }
            "left" => {
                label.horizontal_align = HorizontalAlign::Left;
            }
            "right" => {
                label.horizontal_align = HorizontalAlign::Right;
            }
            _ => {
                return Err(EmeraldError::new(format!(
                    "{:?} is not a valid horizontal align value",
                    horizontal_align
                )))
            }
        }
    }

    world.insert_one(entity, label)?;

    Ok(())
}
