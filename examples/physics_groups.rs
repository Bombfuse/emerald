use emerald::*;

const RES_WIDTH: f32 = 640.0;
const RES_HEIGHT: f32 = 480.0;

pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (RES_WIDTH as u32, RES_HEIGHT as u32);
    settings.render_settings = render_settings;
    emerald::start(
        Box::new(PhysicsGroupsExample {
            e1: None,
            e2: None,
            e3: None,
        }),
        settings,
    )
}

const GROUP_ONE: InteractionGroups = InteractionGroups::new(1, 1);
const GROUP_TWO: InteractionGroups = InteractionGroups::new(2, 2);

#[derive(Clone, Debug)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Clone, Debug)]
pub struct Controller {}

pub struct PhysicsGroupsExample {
    e1: Option<Entity>,
    e2: Option<Entity>,
    e3: Option<Entity>,
}
impl Game for PhysicsGroupsExample {
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

        let (entity1, body1) = emd
            .world()
            .spawn_with_body(
                (
                    Position::new(0.0, 40.0),
                    ColorRect::new(Color::new(0, 0, 255, 255), 32, 16),
                ),
                RigidBodyBuilder::new_dynamic(),
            )
            .unwrap();

        emd.world().physics().build_collider(
            body1,
            ColliderBuilder::cuboid(16.0, 8.0).collision_groups(GROUP_ONE),
        );

        emd.world().physics().build_collider(
            body1,
            ColliderBuilder::cuboid(16.0, 8.0)
                .collision_groups(GROUP_ONE)
                .sensor(true),
        );

        let (entity2, body2) = emd
            .world()
            .spawn_with_body(
                (
                    Position::new(0.0, 0.0),
                    ColorRect::new(Color::new(0, 255, 0, 255), 32, 16),
                ),
                RigidBodyBuilder::new_kinematic().translation(0.0, 0.0),
            )
            .unwrap();

        emd.world().physics().build_collider(
            body2,
            ColliderBuilder::cuboid(16.0, 8.0).collision_groups(GROUP_TWO),
        );

        let (entity3, body3) = emd
            .world()
            .spawn_with_body(
                (
                    Position::new(0.0, 80.0),
                    ColorRect::new(Color::new(0, 255, 0, 255), 32, 16),
                ),
                RigidBodyBuilder::new_dynamic(),
            )
            .unwrap();

        emd.world().physics().build_collider(
            body3,
            ColliderBuilder::cuboid(16.0, 8.0).collision_groups(GROUP_TWO),
        );

        self.e1 = Some(entity1);
        self.e2 = Some(entity2);
        self.e3 = Some(entity3);

        emd.world().physics().set_gravity(Vector2::new(0.0, -18.8));
    }

    fn update(&mut self, mut emd: Emerald) {
        let delta = emd.delta();
        emd.world().physics().step(delta);
    }
}
