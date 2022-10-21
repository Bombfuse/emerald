use emerald::{rendering::components::Sprite, *};

// Bunnymark is super disappointing right now, need to fix
// https://github.com/Bombfuse/emerald/issues/10

#[derive(Clone, Debug)]
struct Velocity {
    pub x: f32,
    pub y: f32,
}
impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Velocity { x, y }
    }
}

pub fn main() {
    let mut settings = GameSettings::default();
    settings.render_settings.resolution = (320 * 5, 180 * 5);
    emerald::start(
        Box::new(BunnymarkGame {
            count: 0,
            world: World::new(),
        }),
        settings,
    )
}

pub struct BunnymarkGame {
    count: u64,
    world: World,
}
impl Game for BunnymarkGame {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        let sprite = emd.loader().sprite("bunny.png").unwrap();

        let mut transform = Transform::default();

        let start = 1_000;
        self.count = start;
        self.world.spawn_batch((0..start).map(|_| {
            transform.translation.x += 1.0;
            transform.translation.y += 1.0;
            (transform, sprite.clone(), Velocity::new(5.0, 3.0))
        }));
    }

    #[inline]
    fn update(&mut self, mut emd: Emerald) {
        let (screen_width, screen_height) = emd.screen_size();
        let screen_width = screen_width as f32;
        let screen_height = screen_height as f32;

        let sprite_width = 32.0;

        if emd.input().is_key_just_pressed(KeyCode::Space) {
            let mut sprite = emd.loader().sprite("bunny.png").unwrap();
            sprite.offset = Vector2::new(-10.0, 0.0);

            let mut transform = Transform::default();
            self.count += 1000;
            self.world.spawn_batch((0..1000).map(|_| {
                transform.translation.x += 6.0;
                transform.translation.y += 1.0;

                (transform, sprite.clone(), Velocity::new(5.0, 3.0))
            }));
        }

        for (_, (_, transform, mut vel)) in self
            .world
            .query::<(&Sprite, &mut Transform, &mut Velocity)>()
            .iter()
        {
            if transform.translation.x >= screen_width / 2.0 - sprite_width / 2.0 {
                transform.translation.x = screen_width / 2.0 - sprite_width / 2.0;
                vel.x *= -1.0;
            }

            if transform.translation.x <= -screen_width / 2.0 {
                transform.translation.x = -screen_width / 2.0;
                vel.x *= -1.0;
            }

            if transform.translation.y >= screen_height / 2.0 - sprite_width {
                transform.translation.y = screen_height / 2.0 - sprite_width;
                vel.y = -3.0;
            }

            if transform.translation.y <= -screen_height / 2.0 {
                transform.translation.y = -screen_height / 2.0;
                vel.y = 3.0;
            }

            transform.translation.x += vel.x;
            transform.translation.y += vel.y;
        }

        println!("fps: {:?} count: {:?}", emd.fps() as u32, self.count);
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();

        // let font = emd.loader().font("Roboto-Light.ttf", 40).unwrap();
        // let label = Label::new(format!("FPS: {}", emd.fps() as u32), font.clone(), 40);
        // let bunnycount_label = Label::new(format!("{} bunnies", (self.count)), font, 40);

        // emd.graphics()
        //     .draw_label(&label, &Transform::from_translation((500.0, 500.0)))
        //     .unwrap();
        // emd.graphics()
        //     .draw_label(
        //         &bunnycount_label,
        //         &Transform::from_translation((500.0, 100.0)),
        //     )
        //     .unwrap();

        emd.graphics().render().unwrap();
    }
}
