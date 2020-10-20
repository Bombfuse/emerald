use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (320, 180);
    settings.render_settings = render_settings;
    emerald::start(Box::new(MyGame {}), settings)
}

pub struct MyGame;
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

        let sprite = emd.loader().sprite("./examples/assets/bunny.png").unwrap();

        let entity = emd.world().spawn((sprite, Position::new(32.0, 32.0)));
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
        emd.world().physics().set_gravity(Vector2::new(0.0, -20.0));
    }

    fn update(&mut self, mut emd: Emerald) {
        emd.world().physics().step();
    }
}
