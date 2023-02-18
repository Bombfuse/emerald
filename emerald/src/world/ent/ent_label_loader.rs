use fontdue::layout::{HorizontalAlign, VerticalAlign};
use hecs::Entity;
use rapier2d::na::Vector2;
use serde::{Deserialize, Serialize};

use crate::{rendering::components::Label, AssetLoader, EmeraldError, World};

use super::Vec2f32Schema;

#[derive(Deserialize, Serialize)]
pub(crate) struct EntLabelSchema {
    pub font: Option<String>,
    pub font_size: Option<u32>,

    /// A path to the font resource file
    pub resource: Option<String>,

    pub size: u16,
    pub text: Option<String>,
    pub offset: Option<Vec2f32Schema>,
    /// options: "bottom", "middle", "top"
    pub vertical_align: Option<String>,

    /// options: "center", "left", "right"
    pub horizontal_align: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct FontResource {
    /// The path to the font
    pub font: String,

    /// The size of the font
    pub size: u32,
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

    if (schema.font.is_none() || schema.font_size.is_none()) && schema.resource.is_none() {
        return Err(EmeraldError::new(format!("Failure loading entity {:?}: Labels require either a resource OR a (font AND font_size).", entity)));
    }

    let mut font = None;

    if let (Some(font_path), Some(font_size)) = (schema.font, schema.font_size) {
        font = Some(loader.font(font_path, font_size)?);
    } else if let Some(resource_file) = schema.resource {
        let resource_data = loader.string(resource_file)?;
        let resource: FontResource = toml::from_str(&resource_data)?;
        font = Some(loader.font(resource.font, resource.size)?);
    }

    if font.is_none() {
        return Err(EmeraldError::new(format!("Failure loading entity {:?}: Labels require either a resource OR a (font AND font_size).", entity)));
    }

    let font = font.unwrap();
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

#[cfg(test)]
mod tests {
    use super::FontResource;

    #[test]
    fn validate_font_resource() {
        let example_resource = r#"
            font = "Roboto-Light.ttf"
            size = 48
        "#;

        toml::from_str::<FontResource>(example_resource).unwrap();
    }
}
