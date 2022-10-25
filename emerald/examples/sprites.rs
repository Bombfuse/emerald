use emerald::{rendering::components::Sprite, *};

pub fn main() {
    let mut settings = GameSettings::default();
    settings.render_settings.resolution = (320, 180);
    emerald::start(
        Box::new(SpritesExample {
            world: World::new(),
        }),
        settings,
    )
}

pub struct SpritesExample {
    world: World,
}
impl Game for SpritesExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        let sprite = emd.loader().sprite("bunny.png").unwrap();
        self.world.spawn((sprite, Transform::default()));
        let sprite = emd.loader().sprite("smiley.png").unwrap();
        self.world
            .spawn((sprite, Transform::from_translation((-32.0, 0.0))));
    }

    fn update(&mut self, mut emd: Emerald) {
        let mouse = emd.input().mouse();
        let screen_size = emd.screen_size();
        let mouse_position = screen_translation_to_world_translation(
            (screen_size.0 as u32, screen_size.1 as u32),
            &mouse.translation,
            &mut self.world,
        );

        // Spawn with left mouse
        if mouse.left.is_just_pressed() {
            let mut sprite = emd.loader().sprite("bunny.png").unwrap();
            sprite.offset = Vector2::new(-10.0, 0.0);

            let mut transform = Transform::default();
            transform.translation = mouse_position;
            self.world.spawn((sprite, transform));
        }

        // move to mouse position
        if mouse.right.is_pressed {
            let speed = 0.1 * emd.delta();

            for (_, (transform, _sprite)) in
                self.world.query::<(&mut Transform, &mut Sprite)>().iter()
            {
                transform.translation.x += (mouse_position.x - transform.translation.x) * speed;
                transform.translation.y += (mouse_position.y - transform.translation.y) * speed;
            }
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
