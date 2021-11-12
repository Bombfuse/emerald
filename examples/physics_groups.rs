use emerald::*;

const RES_WIDTH: f32 = 640.0;
const RES_HEIGHT: f32 = 480.0;

pub fn main() {
    let mut settings = GameSettings::default();
    let render_settings = RenderSettings {
        resolution: (RES_WIDTH as u32, RES_HEIGHT as u32),
        ..Default::default()
    };
    settings.render_settings = render_settings;
    emerald::start(
        PhysicsGroupsExample {
            e1: None,
            e2: None,
            e3: None,
            world: EmeraldWorld::new(),
        },
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
    world: EmeraldWorld,
}
impl Game for PhysicsGroupsExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        let (entity1, body1) = self.world
            .spawn_with_body(
                (
                    Position::new(0.0, 40.0),
                    ColorRect::new(Color::new(0, 0, 255, 255), 32, 16),
                ),
                RigidBodyBuilder::new_dynamic(),
            )
            .unwrap();

        self.world.physics().build_collider(
            body1,
            ColliderBuilder::cuboid(16.0, 8.0).collision_groups(GROUP_ONE),
        );

        self.world.physics().build_collider(
            body1,
            ColliderBuilder::cuboid(16.0, 8.0)
                .collision_groups(GROUP_ONE)
                .sensor(true),
        );

        let (entity2, body2) = self.world
            .spawn_with_body(
                (
                    Position::new(0.0, 0.0),
                    ColorRect::new(Color::new(0, 255, 0, 255), 32, 16),
                ),
                RigidBodyBuilder::new_kinematic_position_based()
                    .translation(Vector2::new(0.0, 0.0)),
            )
            .unwrap();

        self.world.physics().build_collider(
            body2,
            ColliderBuilder::cuboid(16.0, 8.0).collision_groups(GROUP_TWO),
        );

        let (entity3, body3) = self.world
            .spawn_with_body(
                (
                    Position::new(0.0, 80.0),
                    ColorRect::new(Color::new(0, 255, 0, 255), 32, 16),
                ),
                RigidBodyBuilder::new_dynamic(),
            )
            .unwrap();

        self.world.physics().build_collider(
            body3,
            ColliderBuilder::cuboid(16.0, 8.0).collision_groups(GROUP_TWO),
        );

        self.e1 = Some(entity1);
        self.e2 = Some(entity2);
        self.e3 = Some(entity3);

        self.world.physics().set_gravity(Vector2::new(0.0, -18.8));
    }

    fn update(&mut self, emd: Emerald) {
        let delta = emd.delta();
        self.world.physics().step(delta);
    }

    fn draw(&mut self, mut emd: Emerald<'_>) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
