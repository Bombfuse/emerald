use std::time::Duration;

use emerald::*;

pub fn main() {
    emerald::start(
        Box::new(WorldMergingExample {
            world: World::new(),
        }),
        GameSettings::default(),
    )
}

pub struct WorldMergingExample {
    world: World,
}
impl Game for WorldMergingExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        let sprite = emd.loader().sprite("bunny.png").unwrap();
        self.world.spawn((sprite, Transform::default()));
        self.world.physics().set_gravity(Vector2::new(0.0, -19.8));
    }

    fn update(&mut self, mut emd: Emerald) {
        let mut input = emd.input();

        if input.is_key_just_pressed(KeyCode::A) {
            let amount = 100;
            let world = build_other_world(
                &mut emd,
                Transform::from_translation((100.0, 100.0)),
                amount,
            )
            .unwrap();
            let now = emd.now();
            self.world
                .merge(world, Transform::from_translation((100.0, 100.0)))
                .unwrap();
            let after = emd.now();
            println!(
                "merged {} bunnies in {:?}us",
                amount,
                Duration::from_secs_f64(after - now).as_micros()
            );
        }

        self.world.physics().step(emd.delta());
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}

fn build_other_world(
    emd: &mut Emerald,
    offset: Transform,
    amount: i32,
) -> Result<World, EmeraldError> {
    let mut world = World::new();
    let sprite = emd.loader().sprite("bunny.png")?;
    let amount = (amount as f32).sqrt() as usize;

    for x in 0..amount {
        for y in 0..amount {
            let (_entity, rbh) = world
                .spawn_with_body(
                    (
                        Transform::from_translation((
                            x as f32 * 30.0 - 300.0,
                            y as f32 * 30.0 - 300.0,
                        )) + offset,
                        sprite.clone(),
                    ),
                    RigidBodyBuilder::dynamic()
                        .can_sleep(false)
                        .linvel(Vector2::new(10.0, 10.0)),
                )
                .unwrap();

            world
                .physics()
                .build_collider(rbh, ColliderBuilder::cuboid(1.0, 1.0));
        }
    }

    Ok(world)
}
