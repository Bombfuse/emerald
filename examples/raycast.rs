use emerald::*;
use nalgebra::Point2;

pub fn main() {
    emerald::start(
        Box::new(RaycastExample {
            world: EmeraldWorld::new(),
        }),
        GameSettings::default(),
    )
}

pub struct RaycastExample {
    world: EmeraldWorld,
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
                    Position::new(200.0, 0.0),
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
                    Position::new(-200.0, 0.0),
                    String::from("entity on the left"),
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

            println!("hit {:?}", entity);
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
