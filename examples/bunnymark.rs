use emerald::*;

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
        BunnymarkGame {
            count: 0,
            world: EmeraldWorld::new(),
        },
        settings,
    )
}

pub struct BunnymarkGame {
    count: u64,
    world: EmeraldWorld,
}
impl Game for BunnymarkGame {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        let sprite = emd.loader().sprite("bunny.png").unwrap();

        let mut position = Position::new(0.0, 0.0);

        self.count = 1000;
        self.world.spawn_batch((0..1000).map(|_| {
            position.x += 6.0;
            position.y += 1.0;
            (position, sprite.clone(), Velocity::new(5.0, 3.0))
        }));
    }

    #[inline]
    fn update(&mut self, mut emd: Emerald) {
        let (screen_width, screen_height) = emd.screen_size();
        let sprite_width = 32.0;

        if emd.input().is_key_just_pressed(KeyCode::Space) {
            let mut sprite = emd.loader().sprite("bunny.png").unwrap();
            sprite.offset = Vector2::new(-10.0, 0.0);

            let mut position = Position::new(0.0, 0.0);
            self.count += 1000;
            self.world.spawn_batch((0..1000).map(|_| {
                position.x += 6.0;
                position.y += 1.0;

                (position, sprite.clone(), Velocity::new(5.0, 3.0))
            }));
        }

        for (_, (_, mut position, mut vel)) in self
            .world
            .query::<(&Sprite, &mut Position, &mut Velocity)>()
            .iter()
        {
            if position.x >= screen_width / 2.0 - sprite_width / 2.0 {
                position.x = screen_width / 2.0 - sprite_width / 2.0;
                vel.x *= -1.0;
            }

            if position.x <= -screen_width / 2.0 {
                position.x = -screen_width / 2.0;
                vel.x *= -1.0;
            }

            if position.y >= screen_height / 2.0 - sprite_width {
                position.y = screen_height / 2.0 - sprite_width;
                vel.y = -3.0;
            }

            if position.y <= -screen_height / 2.0 {
                position.y = -screen_height / 2.0;
                vel.y = 3.0;
            }

            position.x += vel.x;
            position.y += vel.y;
        }
    }

    fn draw(&mut self, mut emd: Emerald<'_>) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();

        let font = emd.loader().font("Roboto-Light.ttf", 40).unwrap();
        let label = Label::new(format!("FPS: {}", emd.fps() as u32), font.clone(), 40);
        let bunnycount_label = Label::new(format!("{} bunnies", (self.count)), font, 40);

        emd.graphics()
            .draw_label(&label, &Position::new(500.0, 500.0))
            .unwrap();
        emd.graphics()
            .draw_label(&bunnycount_label, &Position::new(500.0, 100.0))
            .unwrap();

        emd.graphics().render().unwrap();
    }
}
