use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let render_settings = RenderSettings {
        resolution: (480, 320),
        ..Default::default()
    };
    settings.render_settings = render_settings;
    emerald::start(
        GamepadExample {
            world: World::new(),
        },
        settings,
    )
}

pub struct GamepadExample {
    world: World,
}
impl Game for GamepadExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        match emd.loader().sprite("bunny.png") {
            Ok(sprite) => {
                self.world.spawn((sprite, Transform::from_translation((16.0, 16.0))));
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

        for (_, (transform, sprite)) in self.world.query::<(&mut Transform, &mut Sprite)>().iter() {
            if input.is_button_just_pressed(Button::North) {
                sprite.scale *= 2.0;
            } else if input.is_button_just_pressed(Button::South) {
                sprite.scale *= 0.5;
            }

            if input.is_button_just_pressed(Button::West) {
                sprite.visible = !sprite.visible;
            }

            transform.translation.x += delta * velocity.x;
            transform.translation.y += delta * velocity.y;
        }
    }

    fn draw(&mut self, mut emd: Emerald<'_>) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
