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
    settings.render_settings.resolution = (320, 180);
    emerald::start(Box::new(BunnymarkGame { count: 0 }), settings)
}

pub struct BunnymarkGame {
    count: u64,
}
impl Game for BunnymarkGame {
    fn initialize(&mut self, mut emd: Emerald) {
        // Pack all game files into WASM binary
        #[cfg(target_arch = "wasm32")]
        {
            emd.loader().pack_bytes(
                "./examples/assets/bunny.png",
                include_bytes!("../examples/assets/bunny.png").to_vec(),
            );
        }

        let sprite = emd.loader().sprite("./examples/assets/bunny.png").unwrap();

        let mut position = Position::new(0.0, 0.0);

        self.count = 1000;
        emd.world().spawn_batch((0..1000).map(|_| {
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
            let mut sprite = emd.loader().sprite("./examples/assets/bunny.png").unwrap();
            sprite.offset = Vector2::new(-10.0, 0.0);

            let mut position = Position::new(0.0, 0.0);
            self.count += 1000;
            emd.world().spawn_batch((0..1000).map(|_| {
                position.x += 6.0;
                position.y += 1.0;

                (position, sprite.clone(), Velocity::new(5.0, 3.0))
            }));
        }

        for (_, (_, mut position, mut vel)) in emd
            .world()
            .query::<(&Sprite, &mut Position, &mut Velocity)>()
            .iter()
        {
            if position.x >= screen_width - sprite_width / 2.0 {
                position.x = screen_width - sprite_width / 2.0;
                vel.x *= -1.0;
            }

            if position.x <= 0.0 {
                position.x = 0.0;
                vel.x *= -1.0;
            }

            if position.y >= screen_height - sprite_width {
                position.y = screen_height - sprite_width;
                vel.y = -3.0;
            }

            if position.y <= 0.0 {
                position.y = 0.0;
                vel.y = 3.0;
            }

            position.x += vel.x;
            position.y += vel.y;
        }

        // emd.world().physics().step();
        println!("{}, {}", self.count, emd.fps());
    }
}
