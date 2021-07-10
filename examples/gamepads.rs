use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (480, 320);
    settings.render_settings = render_settings;
    emerald::start(Box::new(GamepadExample {}), settings)
}

pub struct GamepadExample;
impl Game for GamepadExample {
    fn initialize(&mut self, mut emd: Emerald) {
        // Pack all game files into WASM binary
        #[cfg(target_arch = "wasm32")]
        {
            emd.loader()
                .pack_bytes(
                    "./examples/assets/bunny.png",
                    include_bytes!("./assets/bunny.png").to_vec(),
                )
                .unwrap();
        }

        match emd.loader().sprite("./examples/assets/bunny.png") {
            Ok(sprite) => {
                emd.world().spawn((sprite, Position::new(16.0, 16.0)));
            }
            Err(_) => {}
        };
    }

    fn update(&mut self, mut emd: Emerald) {
        let delta = emd.delta();
        let mut input = emd.input();
        let mut velocity = Vector2::new(0.0, 0.0);
        let speed = 500.0;

        let mut direction = input.joystick(Joystick::Left);

        if input.is_button_pressed(Button::DPadNorth) {
            direction.1 = 1.0;
        } else if input.is_button_pressed(Button::DPadSouth) {
            direction.1 = -1.0;
        }

        if input.is_button_pressed(Button::DPadWest) {
            direction.0 = -1.0;
        } else if input.is_button_pressed(Button::DPadEast) {
            direction.0 = 1.0;
        }

        velocity.x = direction.0 * speed;
        velocity.y = direction.1 * speed;

        for (_, (position, sprite)) in emd.world().query::<(&mut Position, &mut Sprite)>().iter() {
            if input.is_button_just_pressed(Button::North) {
                sprite.scale *= 2.0;
            } else if input.is_button_just_pressed(Button::South) {
                sprite.scale *= 0.5;
            }

            if input.is_button_just_pressed(Button::West) {
                sprite.visible = !sprite.visible;
            }

            position.x += delta * velocity.x;
            position.y += delta * velocity.y;
        }
    }
}
