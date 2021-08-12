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
        Box::new(MyGame {
            elapsed_time_cube: 0.0,
            elapsed_time_round: 0.0,
        }),
        settings,
    )
}

#[derive(Clone, Debug)]
pub struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

#[derive(Clone, Debug)]
pub struct Controller {}

pub struct MyGame {
    elapsed_time_cube: f32,
    elapsed_time_round: f32,
}
impl MyGame {
    fn spawn_bunny_cube(&mut self, position: Position, emd: &mut Emerald) {
        self.spawn_bunny(
            position,
            emd,
            ColliderBuilder::cuboid(6.0, 6.0),
            Velocity { dx: 75.0, dy: 50.0 },
        );
    }

    fn spawn_bunny_round(&mut self, position: Position, emd: &mut Emerald) {
        self.spawn_bunny(
            position,
            emd,
            ColliderBuilder::ball(6.0),
            Velocity {
                dx: -75.0,
                dy: 50.0,
            },
        );
    }

    fn spawn_bunny(
        &mut self,
        position: Position,
        emd: &mut Emerald,
        collider_builder: ColliderBuilder,
        velocity: Velocity,
    ) {
        let sprite = emd.loader().sprite("bunny.png").unwrap();
        let entity = emd.world().spawn((sprite, position));
        let body = emd
            .world()
            .physics()
            .build_body(
                entity,
                RigidBodyBuilder::new_dynamic().linvel(Vector2::new(velocity.dx, velocity.dy)), // Fling it up and to the right
            )
            .unwrap();
        emd.world().physics().build_collider(body, collider_builder);
    }
}
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        let borders = vec![
            (
                Position::new(0.0, RES_HEIGHT / -2.0),
                (RES_WIDTH / 2.0, 3.0),
            ),
            (Position::new(0.0, RES_HEIGHT / 2.0), (RES_WIDTH / 2.0, 3.0)),
            (Position::new(RES_WIDTH / 2.0, 0.0), (3.0, RES_HEIGHT / 2.0)),
            (
                Position::new(RES_WIDTH / -2.0, 0.0),
                (3.0, RES_HEIGHT / 2.0),
            ),
        ];

        for border in borders {
            let (_, border_body) = emd
                .world()
                .spawn_with_body(
                    (border.0,),
                    RigidBodyBuilder::new_static()
                        .translation(Vector2::new(border.0.x, border.0.y)),
                )
                .unwrap();
            emd.world().physics().build_collider(
                border_body,
                ColliderBuilder::cuboid((border.1).0, (border.1).1),
            );
        }
        emd.world().physics().set_gravity(Vector2::new(0.0, -98.0));

        let size = Vector2::new(64.0, 16.0);
        let mut color_rect = ColorRect::new(WHITE, size.x as u32, size.y as u32);
        color_rect.z_index = 10.0;

        // Spawn controller
        let (_, body_handle) = emd
            .world()
            .spawn_with_body(
                (
                    Controller {},
                    Velocity { dx: 0.0, dy: 0.0 },
                    Position::new(0.0, 0.0),
                    color_rect,
                ),
                RigidBodyBuilder::new_kinematic_position_based().can_sleep(false),
            )
            .unwrap();
        emd.world().physics().build_collider(
            body_handle,
            ColliderBuilder::cuboid(size.x / 2.0, size.y / 2.0),
        );
    }

    fn update(&mut self, mut emd: Emerald) {
        let delta = emd.delta();
        self.elapsed_time_cube += emd.delta() as f32;
        self.elapsed_time_round += emd.delta() as f32;
        let mut input = emd.input();

        for (_, (position, _)) in emd.world().query::<(&mut Position, &Controller)>().iter() {
            if input.is_key_pressed(KeyCode::Up) {
                position.y += delta * 80.0;
            } else if input.is_key_pressed(KeyCode::Down) {
                position.y += delta * -80.0;
            }

            if input.is_key_pressed(KeyCode::Left) {
                position.x += delta * -80.0;
            } else if input.is_key_pressed(KeyCode::Right) {
                position.x += delta * 80.0;
            }
        }

        if self.elapsed_time_cube > 0.05 {
            self.elapsed_time_cube = 0.0;
            self.spawn_bunny_cube(Position::new(0.0, RES_HEIGHT / 2.0 - 12.0), &mut emd);
        }

        if self.elapsed_time_round > 0.05 {
            self.elapsed_time_round = 0.0;
            self.spawn_bunny_round(Position::new(0.0, RES_HEIGHT / 2.0 - 12.0), &mut emd);
        }

        emd.world().physics().step(delta);
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();

        if let Some(mut world) = emd.pop_world() {
            emd.graphics().draw_world(&mut world).unwrap();
            emd.push_world(world);
        }

        {
            let fps = emd.fps() as u8;
            let font = emd.loader().font("Roboto-Light.ttf", 48).unwrap();
            let mut label = Label::new(format!("FPS: {}", fps), font, 24);
            label.centered = false;
            emd.graphics()
                .draw_label(&label, &Position::new(24.0, RES_HEIGHT as f32 - 10.0))
                .unwrap();
        }
        // emd.graphics().draw_colliders(Color::new(255, 0, 0, 130));
        emd.graphics().render().unwrap();
    }
}
