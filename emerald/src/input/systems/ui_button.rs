use std::collections::HashMap;

use crate::{
    screen_translation_to_world_translation,
    texture::Texture,
    transform::{Transform, Translation},
    Emerald, TouchState, UIButton, World,
};

/// Updates the status of UI Buttons.
/// Presses the button if the user has pressed it, etc...
pub fn ui_button_system(emd: &mut Emerald<'_>, world: &mut World) {
    let mouse = emd.input().mouse();
    let touches = emd.input().touches().clone();
    let screen_size = emd.screen_size();
    let mouse_position = screen_translation_to_world_translation(
        (screen_size.0 as u32, screen_size.1 as u32),
        &mouse.translation,
        world,
    );

    let touch_world_positions: HashMap<u64, Translation> = touches
        .iter()
        .map(|(id, touch_state)| {
            let sc = (screen_size.0 as u32, screen_size.1 as u32);
            (
                *id,
                screen_translation_to_world_translation(sc, &touch_state.translation, world),
            )
        })
        .collect();

    for (_, (ui_button, transform)) in world.query::<(&mut UIButton, &Transform)>().iter() {
        let button_check =
            is_translation_inside_button(emd, &ui_button, &transform, &mouse_position)
                || check_touches_overlap_button(
                    emd,
                    &touches,
                    &touch_world_positions,
                    &ui_button,
                    &transform,
                );

        if button_check {
            let press = mouse.left.is_pressed
                || touches
                    .iter()
                    .any(|(_key, touch_state)| touch_state.is_pressed());

            if press {
                ui_button.press();
            } else {
                ui_button.release();
            }
        } else {
            // Wipe button state if mouse is not in button
            ui_button.reset();
        }
    }
}

fn check_touches_overlap_button(
    emd: &mut Emerald<'_>,
    touches: &HashMap<u64, TouchState>,
    touch_world_positions: &HashMap<u64, Translation>,
    ui_button: &UIButton,
    ui_button_transform: &Transform,
) -> bool {
    touches.iter().any(|(id, _touch)| {
        let mut is_inside = false;

        if let Some(position) = touch_world_positions.get(id) {
            is_inside =
                is_translation_inside_button(emd, &ui_button, &ui_button_transform, position);
        }

        is_inside
    })
}

// TODO: take into account the scale and rotation of the button.
fn is_translation_inside_button(
    emd: &mut Emerald<'_>,
    ui_button: &UIButton,
    ui_button_transform: &Transform,
    translation: &Translation,
) -> bool {
    let mut is_inside = false;

    let texture_key = ui_button.current_texture();

    if let Some(texture) = emd
        .asset_engine
        .get_asset::<Texture>(&texture_key.asset_key.asset_id)
    {
        if (translation.x >= ui_button_transform.translation.x - texture.size.width as f32 / 2.0)
            && (translation.x
                <= ui_button_transform.translation.x + texture.size.width as f32 / 2.0)
        {
            if (translation.y
                >= ui_button_transform.translation.y - texture.size.height as f32 / 2.0)
                && (translation.y
                    <= ui_button_transform.translation.y + texture.size.height as f32 / 2.0)
            {
                is_inside = true;
            }
        }
    }

    is_inside
}
