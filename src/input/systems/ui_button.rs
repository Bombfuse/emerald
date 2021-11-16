use crate::{Emerald, EmeraldWorld, Position, UIButton, screen_position_to_world_position};

/// Updates the status of UI Buttons.
/// Presses the button if the user has pressed it, etc...
pub fn ui_button_system(emd: &mut Emerald<'_>, world: &mut EmeraldWorld) {
    let mouse = emd.input().mouse();
    let screen_size = emd.screen_size();
    let mouse_position = screen_position_to_world_position((screen_size.0 as u32, screen_size.1 as u32), &mouse.position, world);

    for (_, (ui_button, position)) in world.query::<(&mut UIButton, &Position)>().iter() {
        if is_mouse_inside_button(emd, &ui_button, &position, &mouse_position) {
            if mouse.left.is_just_pressed() || (ui_button.is_pressed() && mouse.left.is_pressed) {
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

fn is_mouse_inside_button(emd: &mut Emerald<'_>, ui_button: &UIButton, ui_button_position: &Position, mouse_position: &Position) -> bool {
    let mut is_inside = false;

    let texture_key = if ui_button.is_pressed() {
        &ui_button.pressed_texture
    } else {
        &ui_button.unpressed_texture
    };

    if let Some(texture) = emd.asset_store.get_texture(texture_key) {
        if (mouse_position.x >= ui_button_position.x - texture.width as f32 / 2.0) && (mouse_position.x <= ui_button_position.x + texture.width as f32/ 2.0) {
            if (mouse_position.y >= ui_button_position.y - texture.height as f32 / 2.0) && (mouse_position.y <= ui_button_position.y + texture.height as f32 / 2.0) {
                is_inside = true;
            }
        }
    }

    is_inside
}