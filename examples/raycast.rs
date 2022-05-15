use emerald::{parry::shape::Cuboid, *};
use nalgebra::Point2;

pub fn main() {
    emerald::start(
        Box::new(RaycastExample {
            world: World::new(),
        }),
        GameSettings::default(),
    )
}

pub struct RaycastExample {
    world: World,
}
impl Game for RaycastExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        let sprite = emd.loader().sprite("bunny.png").unwrap();

        let (_, rbh1) = self
            .world
            .spawn_with_body(
                (
                    sprite.clone(),
                    Transform::from_translation((200.0, 0.0)),
                    String::from("entity on the right"),
                ),
                RigidBodyBuilder::new_static(),
            )
            .unwrap();
        let (_, rbh2) = self
            .world
            .spawn_with_body(
                (
                    sprite.clone(),
                    Transform::from_translation((-200.0, 0.0)),
                    String::from("entity on the left"),
                ),
                RigidBodyBuilder::new_static(),
            )
            .unwrap();
        let (_, rbh3) = self
            .world
            .spawn_with_body(
                (
                    sprite.clone(),
                    Transform::from_translation((90.0, 200.0)),
                    String::from("entity on the top"),
                ),
                RigidBodyBuilder::new_static(),
            )
            .unwrap();
        let (_, rbh4) = self
            .world
            .spawn_with_body(
                (
                    sprite.clone(),
                    Transform::from_translation((-40.0, -200.0)),
                    String::from("entity on the bottom"),
                ),
                RigidBodyBuilder::new_static(),
            )
            .unwrap();

        self.world
            .physics()
            .build_collider(rbh1, ColliderBuilder::cuboid(20.0, 20.0));
        self.world
            .physics()
            .build_collider(rbh2, ColliderBuilder::cuboid(20.0, 20.0));

        self.world
            .physics()
            .build_collider(rbh3, ColliderBuilder::cuboid(20.0, 20.0));
        self.world
            .physics()
            .build_collider(rbh4, ColliderBuilder::cuboid(20.0, 20.0));
    }

    fn update(&mut self, mut emd: Emerald) {
        let delta = emd.delta();
        let mut ray = None;

        if emd.input().is_key_just_pressed(KeyCode::Left) {
            ray = Some(Ray::new(Point2::new(0.0, 0.0), Vector2::new(-500.0, 0.0)));
        } else if emd.input().is_key_just_pressed(KeyCode::Right) {
            ray = Some(Ray::new(Point2::new(0.0, 0.0), Vector2::new(500.0, 0.0)));
        }

        if let Some(ray) = ray {
            let entity = self.world.physics().cast_ray(RayCastQuery {
                ray,
                ..RayCastQuery::default()
            });

            if let Some(e) = entity {
                if let Ok(s) = self.world.get_mut::<String>(e) {
                    println!("Found {}", s.clone());
                }
            }
        }

        let mut vel = None;

        if emd.input().is_key_just_pressed(KeyCode::Up) {
            vel = Some(Vector2::new(0.0, 50.0));
        } else if emd.input().is_key_just_pressed(KeyCode::Down) {
            vel = Some(Vector2::new(0.0, -50.0));
        }

        if let Some(vel) = vel {
            let shape = Cuboid::new(Vector2::new(40.0, 10.0));

            let entity = self.world.physics().cast_shape(
                &shape,
                ShapeCastQuery {
                    velocity: vel,
                    origin_translation: Translation::default(),
                    max_toi: 30.0,
                    ..ShapeCastQuery::default()
                },
            );

            if let Some(e) = entity {
                if let Ok(s) = self.world.get_mut::<String>(e) {
                    println!("Found {}", s.clone());
                }
            }
        }

        self.world.physics().step(delta);
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
