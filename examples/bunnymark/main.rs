use emerald::*;

///
/// Basic Bunnymark
/// 

pub struct Vel {
    pub x: f32,
    pub y: f32,
}

pub fn main() {
    emerald::start(Box::new(BunnymarkGame { count: 0, }), GameSettings::default())
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
                    "./static/assets/bunny.png",
                    include_bytes!("../static/assets/bunny.png").to_vec()
                );
        }

        let sprite = emd.loader()
            .sprite("./static/assets/bunny.png").unwrap();
        
        let mut position = Position::new(0.0, 0.0);

        self.count = 1000;
        emd.world().insert((),
            (0..1000).map(|_| {
                position.x += 6.0;
                position.y += 1.0;
                let mut s = sprite.clone();
                (position.clone(), s, Vel { x: 5.0, y: 3.0 })
            })
        );
    }

    #[inline]
    fn update(&mut self, mut emd: Emerald) {
        let (screen_width, screen_height) = emd.screen_size();

        if emd.input().is_key_just_pressed(KeyCode::Space) {
            let sprite = emd.loader()
                .sprite("./static/assets/bunny.png").unwrap();
            
            let mut position = Position::new(0.0, 0.0);
            self.count += 1000;
            emd.world().insert((),
                (0..1000).map(|_| {
                    position.x += 6.0;
                    position.y += 1.0;
                    let mut s = sprite.clone();
                    (position.clone(), s, Vel { x: 5.0, y: 3.0 })
                })
            );
        }

        let now = Instant::now();
        let bunny_query = <(Read<Sprite>, Write<Position>, Write<Vel>)>::query();
        for (_, mut position, mut vel) in bunny_query.iter_mut(emd.world().queryable()) {
            position.x += vel.x;
            position.y += vel.y;

            if position.x >= screen_width {
                position.x = screen_width;
                vel.x *= -1.0;
            }

            if position.x <= 0.0 {
                position.x = 0.0;
                vel.x *= -1.0;
            }

            if position.y >= screen_height {
                position.y = screen_height;
                vel.y = -3.0;
            }

            if position.y <= 0.0 {
                position.y = 0.0;
                vel.y = 3.0;
            }
        }
    }
}