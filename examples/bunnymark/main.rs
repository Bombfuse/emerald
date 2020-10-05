use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    settings.render_settings.resolution = (320, 180);
    emerald::start(Box::new(BunnymarkGame { count: 0, }), settings)
}

pub struct BunnymarkGame {
    count: u64,
}
impl Game for BunnymarkGame {
    fn initialize(&mut self, mut emd: Emerald) {
        // Pack all game files into WASM binary
        #[cfg(target_arch = "wasm32")]
        {
            emd.loader()
                .pack_texture(
                    "./examples/assets/bunny.png",
                    include_bytes!("../examples/assets/bunny.png").to_vec()
                );
        }

        let mut sprite = emd.loader()
            .sprite("./examples/assets/bunny.png").unwrap();
        sprite.offset = Vector2::new(-10.0, 0.0);
        
        let mut position = Position::new(0.0, 0.0);

        self.count = 1000;
        emd.world().inner().extend(
            (0..1000).map(|_| {
                position.x += 6.0;
                position.y += 1.0;
                let mut s = sprite.clone();
                (position.clone(), s, Velocity::linear(5.0, 3.0))
            })
        );
    }

    #[inline]
    fn update(&mut self, mut emd: Emerald) {
        let (screen_width, screen_height) = emd.screen_size();
        let sprite_width = 32.0;

        if emd.input().is_key_just_pressed(KeyCode::Space) {
            let mut sprite = emd.loader()
                .sprite("./examples/assets/bunny.png").unwrap();
            sprite.offset = Vector2::new(-10.0, 0.0);
            
            let mut position = Position::new(0.0, 0.0);
            self.count += 1000;
            emd.world().inner().extend(
                (0..1000).map(|_| {
                    position.x += 6.0;
                    position.y += 1.0;
                    let mut s = sprite.clone();
                    (position.clone(), s, Velocity::linear(5.0, 3.0))
                })
            );
        }

        let now = Instant::now();
        let mut bunny_query = <(&Sprite, &mut Position, &mut Velocity)>::query();

        for (_, mut position, mut vel) in bunny_query.iter_mut(emd.world().inner()) {
            if position.x >= screen_width - sprite_width / 2.0 {
                position.x = screen_width - sprite_width / 2.0;
                vel.linear.x *= -1.0;
            }

            if position.x <= 0.0 {
                position.x = 0.0;
                vel.linear.x *= -1.0;
            }

            if position.y >= screen_height - sprite_width {
                position.y = screen_height - sprite_width;
                vel.linear.y = -3.0;
            }

            if position.y <= 0.0 {
                position.y = 0.0;
                vel.linear.y = 3.0;
            }

        }

        emd.world().physics().step();
        println!("{}, {}", self.count, emd.fps());
    }
}