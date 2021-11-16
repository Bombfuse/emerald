use std::collections::HashMap;

use emerald::*;

pub fn main() {
    let game = TouchExample {
        bunnies: HashMap::new(),
        sprite: None,
        world: EmeraldWorld::new(),
    };
    emerald::start(game, GameSettings::default())
}

pub struct TouchExample {
    bunnies: HashMap<u64, Entity>,
    sprite: Option<Sprite>,
    world: EmeraldWorld,
}

impl Game for TouchExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        self.sprite = emd.loader().sprite("bunny.png").ok();
        emd.mouse_to_touch(true);
    }

    fn update(&mut self, mut emd: Emerald) {
        let input = emd.input();
        let touches = input.touches();

        let screen = emd.screen_size();
        let screen_center = Position::new(screen.0 / 2.0, screen.1 / 2.0);

        for (&id, touch) in touches {
            let bunny_position = touch.position - screen_center;
            if touch.is_just_pressed() {
                let components: (Sprite, Position) = (self.sprite.clone().unwrap(), bunny_position);
                self.bunnies.insert(id, self.world.spawn(components));
            } else if touch.is_just_released() {
                if let Some(x) = self.bunnies.remove(&id) {
                    self.world.despawn(x).unwrap();
                }
            } else {
                let bunny = self
                    .bunnies
                    .get(&id)
                    .copied()
                    .and_then(|ent| self.world.get_mut::<Position>(ent).ok());
                if let Some(mut bunny) = bunny {
                    *bunny = bunny_position;
                }
            }
        }
    }

    fn draw(&mut self, mut emd: Emerald<'_>) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
