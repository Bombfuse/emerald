use std::collections::HashMap;

use crate::{
    screen_position_to_world_position, Emerald, EmeraldWorld, Position, TouchState, UIButton,
};

/// Updates the status of UI Buttons.
/// Presses the button if the user has pressed it, etc...
pub fn ui_button_system(emd: &mut Emerald<'_>, world: &mut EmeraldWorld) {
    let input = emd.input();
    let mouse = input.mouse();
    let touches = input.touches();
    let screen_size = emd.screen_size();
    let mouse_position = screen_position_to_world_position(
        (screen_size.0 as u32, screen_size.1 as u32),
        &mouse.position,
        world,
    );

    let touch_world_positions: HashMap<u64, Position> = touches
        .iter()
        .map(|(id, touch_state)| {
            let sc = (screen_size.0 as u32, screen_size.1 as u32);
            (
                *id,
                screen_position_to_world_position(sc, &touch_state.position, world),
            )
        })
        .collect();

    for (_, (ui_button, position)) in world.query::<(&mut UIButton, &Position)>().iter() {
        let button_check = is_position_inside_button(emd, &ui_button, &position, &mouse_position)
            || check_touches_overlap_button(
                emd,
                touches,
                &touch_world_positions,
                &ui_button,
                &position,
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
    touch_world_positions: &HashMap<u64, Position>,
    ui_button: &UIButton,
    ui_button_position: &Position,
) -> bool {
    touches.iter().any(|(id, _touch)| {
        let mut is_inside = false;

        if let Some(position) = touch_world_positions.get(id) {
            is_inside = is_position_inside_button(emd, &ui_button, &ui_button_position, position);
        }

        is_inside
    })
}

fn is_position_inside_button(
    emd: &mut Emerald<'_>,
    ui_button: &UIButton,
    ui_button_position: &Position,
    position: &Position,
) -> bool {
    let mut is_inside = false;

    let texture_key = if ui_button.is_pressed() {
        &ui_button.pressed_texture
    } else {
        &ui_button.unpressed_texture
    };

    if let Some(texture) = emd.asset_store.get_texture(texture_key) {
        if (position.x >= ui_button_position.x - texture.width as f32 / 2.0)
            && (position.x <= ui_button_position.x + texture.width as f32 / 2.0)
        {
            if (position.y >= ui_button_position.y - texture.height as f32 / 2.0)
                && (position.y <= ui_button_position.y + texture.height as f32 / 2.0)
            {
                is_inside = true;
            }
        }
    }

    is_inside
}
