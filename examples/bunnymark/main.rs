use emerald::*;

pub fn main() {
    emerald::start(Box::new(BunnymarkGame { }), GameSettings::default())
}

pub struct BunnymarkGame;
impl Game for BunnymarkGame {
    fn initialize(&mut self, mut emd: Emerald) {
        let sprite = emd.loader()
            .sprite("./static/assets/bunny.png").unwrap();
        
        let mut position = Position::new(0.0, 0.0);

        emd.world().insert((),
            (0..10).map(|_| {
                position.x += 4.0;
                (position.clone(), sprite.clone())
            })
        );
    }

    fn update(&mut self, mut emd: Emerald) {
        // println!("{}", emd.input().is_key_just_pressed(KeyCode::Space));

        if emd.input().is_key_just_pressed(KeyCode::Space) {
            // println!("pressed {}", emd.delta());

            let sprite = emd.loader()
                .sprite("./static/assets/bunny.png").unwrap();
            
            let mut position = Position::new(0.0, 0.0);
            emd.world().insert((),
                (0..10).map(|_| {
                    position.x += 4.0;
                    (position.clone(), sprite.clone())
                })
            );
        }

        let bunny_query = <(Read<Sprite>, Write<Position>)>::query();
        for (_, mut position) in bunny_query.iter(emd.world().queryable()) {
            position.x += 1.0;
        }
    }
}