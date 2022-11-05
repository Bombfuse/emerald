use std::collections::HashMap;

use emerald::{rendering::components::Sprite, *};

pub fn main() {
    let game = TouchExample {
        bunnies: HashMap::new(),
        sprite: None,
        world: World::new(),
    };
    emerald::start(Box::new(game), GameSettings::default())
}

pub struct TouchExample {
    bunnies: HashMap<u64, Entity>,
    sprite: Option<Sprite>,
    world: World,
}

impl Game for TouchExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        self.sprite = emd.loader().sprite("bunny.png").ok();
        emd.mouse_to_touch(true);
    }

    fn update(&mut self, mut emd: Emerald) {
        let input = emd.input();
        let touches = input.touches().clone();

        let screen = emd.screen_size();
        let screen_center =
            Transform::from_translation((screen.0 as f32 / 2.0, screen.1 as f32 / 2.0));

        for (id, touch) in &touches {
            let bunny_position = touch.translation - screen_center.translation;
            if touch.is_just_pressed() {
                let components: (Sprite, Transform) = (
                    self.sprite.clone().unwrap(),
                    Transform::from_translation(bunny_position),
                );
                self.bunnies.insert(*id, self.world.spawn(components));
            } else if touch.is_just_released() {
                if let Some(x) = self.bunnies.remove(&id) {
                    self.world.despawn(x).unwrap();
                }
            } else {
                let bunny = self
                    .bunnies
                    .get(&id)
                    .copied()
                    .and_then(|ent| self.world.get::<&mut Transform>(ent).ok());
                if let Some(mut bunny) = bunny {
                    bunny.translation = bunny_position;
                }
            }
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
