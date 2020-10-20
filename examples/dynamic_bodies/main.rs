use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (320, 180);
    settings.render_settings = render_settings;
    emerald::start(Box::new(MyGame { elapsed_time: 0.0 }), settings)
}

#[derive(Clone, Debug)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

pub struct MyGame {
    elapsed_time: f32,
}
impl MyGame {
    fn spawn_bunny(&mut self, position: Position, mut emd: &mut Emerald) {
        let sprite = emd.loader().sprite("./examples/assets/bunny.png").unwrap();
        let entity = emd.world().spawn((sprite, position, Velocity { dx: 75.0, dy: 50.0 }));
        let body = emd
            .world()
            .physics()
            .build_body(
                entity,
                RigidBodyBuilder::new_dynamic()
                    .linvel(50.0, 50.0) // Fling it up and to the right
                    .can_sleep(false),
            )
            .unwrap();
        let collider = emd
            .world()
            .physics()
            .build_collider(body, ColliderBuilder::cuboid(4.0, 4.0));
    }
}
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        // Pack all game files into WASM binary
        #[cfg(target_arch = "wasm32")]
        {
            emd.loader()
                .pack_bytes(
                    "./examples/assets/bunny.png",
                    include_bytes!("../assets/bunny.png").to_vec(),
                )
                .unwrap();
        }

        let borders = vec![
            (Position::new(0.0, 90.0), (160.0, 3.0)),
            (Position::new(0.0, -90.0), (160.0, 3.0)),
            (Position::new(160.0, 0.0), (3.0, 90.0)),
            (Position::new(-160.0, 0.0), (3.0, 90.0)),
        ];

        for border in borders {
            let (_, border_body) = emd
                .world()
                .spawn_with_body(
                    (border.0,),
                    RigidBodyBuilder::new_static().translation(border.0.x, border.0.y),
                )
                .unwrap();
            emd.world().physics().build_collider(
                border_body,
                ColliderBuilder::cuboid((border.1).0, (border.1).1),
            );
        }
        emd.world().physics().set_gravity(Vector2::new(0.0, -98.0));
        
        self.spawn_bunny(Position::new(0.0, 0.0), &mut emd);
    }

    fn update(&mut self, mut emd: Emerald) {
        let start = emd.now();
        self.elapsed_time += emd.delta() as f32;

        if self.elapsed_time > 0.5 {
            self.elapsed_time = 0.0;
            self.spawn_bunny(Position::new(0.0, 0.0), &mut emd);
        }

        emd.world().physics().step();

        let end = emd.now();

        emd.logger().info(format!("{:?}", end - start));
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin();
        emd.graphics().draw_world();
        emd.graphics().draw_colliders(Color::new(255, 0, 0, 130));
        emd.graphics().render();
    }
}
