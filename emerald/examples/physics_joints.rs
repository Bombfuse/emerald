use emerald::{
    nalgebra::Point2, render_settings::RenderSettings, rendering::components::ColorRect, *,
};
use rapier2d::prelude::{RevoluteJointBuilder, RigidBodyType};

const RES_WIDTH: f32 = 960.0;
const RES_HEIGHT: f32 = 540.0;

struct Controller;

#[derive(Clone)]
struct Velocity {
    pub dx: f32,
    pub dy: f32,
}

pub fn main() {
    let mut settings = GameSettings::default();
    let render_settings = RenderSettings {
        resolution: (RES_WIDTH as u32, RES_HEIGHT as u32),
        ..Default::default()
    };
    settings.render_settings = render_settings;
    emerald::start(
        Box::new(PhysicsJointsExample {
            world: World::new(),
        }),
        settings,
    )
}

pub struct PhysicsJointsExample {
    world: World,
}
impl PhysicsJointsExample {}
impl Game for PhysicsJointsExample {
    fn initialize(&mut self, mut emd: Emerald) {
        self.world.physics().set_gravity(Vector2::new(0.0, -9.8));
        let mut body_handles = Vec::new();
        let numk = 1;
        let numi = 10;
        let shift = 15.0;
        for k in 0..numk {
            for i in 0..numi {
                let fk = k as f32;
                let fi = i as f32;

                let status = if i == 0 && k == 0 {
                    RigidBodyType::Dynamic
                } else {
                    RigidBodyType::Dynamic
                };

                let (entity, child_handle) = self
                    .world
                    .spawn_with_body(
                        (
                            Transform::from_translation((fk * shift, -fi * shift)),
                            ColorRect::new(WHITE, 10, 10),
                        ),
                        RigidBodyBuilder::new(status),
                    )
                    .unwrap();

                if i == 0 && k == 0 {
                    self.world
                        .insert(entity, (Controller {}, Velocity { dx: 0.0, dy: 0.0 }))
                        .ok();
                }

                self.world
                    .physics()
                    .build_collider(child_handle, ColliderBuilder::ball(0.4));

                // Vertical joint.
                if i > 0 {
                    let parent_handle = *body_handles.last().unwrap();
                    let joint = RevoluteJointBuilder::new().local_anchor2(Point2::new(0.0, shift));
                    self.world
                        .physics()
                        .build_joint(parent_handle, child_handle, joint, true);
                }

                // Horizontal joint.
                if k > 0 {
                    let parent_index = body_handles.len() - numi;
                    let parent_handle = body_handles[parent_index];
                    let joint = RevoluteJointBuilder::new().local_anchor2(Point2::new(-shift, 0.0));
                    self.world
                        .physics()
                        .build_joint(parent_handle, child_handle, joint, true);
                }

                body_handles.push(child_handle);
            }
        }
    }

    fn update(&mut self, mut emd: Emerald) {
        let delta = emd.delta();
        let mouse = emd.input().mouse();

        let mouse_translation = screen_translation_to_world_translation(
            (RES_WIDTH as u32, RES_HEIGHT as u32),
            &mouse.translation,
            &mut self.world,
        );
        let speed = 10.0;

        for (_, (velocity, transform)) in self
            .world
            .query::<(&mut Velocity, &mut Transform)>()
            .with::<&Controller>()
            .iter()
        {
            let diff = mouse_translation - transform.translation;
            velocity.dx = diff.x * speed;
            velocity.dy = diff.y * speed;
        }

        velocity_system(&mut self.world, delta).unwrap();

        self.world.physics().step(delta);
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}

fn velocity_system(world: &mut World, delta: f32) -> Result<(), EmeraldError> {
    let mut velocity_updates = Vec::new();
    for (id, (velocity, body, pos)) in world
        .query::<(&Velocity, &RigidBodyHandle, &Transform)>()
        .iter()
    {
        velocity_updates.push((id, velocity.clone(), body.clone(), pos.clone()));
    }

    let mut kinematic_updates = Vec::new();
    for update in velocity_updates {
        let id = update.0;
        let velocity = update.1;
        let body_handle = update.2;
        let pos = update.3;

        if let Some(body) = world.physics().rigid_body_mut(body_handle) {
            if body.is_kinematic() {
                kinematic_updates.push((id, velocity.clone(), pos));
            }

            if body.is_dynamic() {
                body.set_linvel(Vector2::new(velocity.dx, velocity.dy), true);
            }
        }
    }

    for (id, velocity, _pos) in kinematic_updates {
        if let Ok(mut transform) = world.get::<&mut Transform>(id.clone()) {
            transform.translation.x += velocity.dx * delta;
            transform.translation.y += velocity.dy * delta;
        }
    }

    Ok(())
}
