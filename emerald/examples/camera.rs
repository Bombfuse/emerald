use emerald::{
    rendering::components::{Camera, Sprite},
    *,
};

pub fn main() {
    let mut settings = GameSettings::default();
    settings.render_settings.resolution = (320, 180);
    emerald::start(
        Box::new(CameraExample {
            world: World::new(),
        }),
        settings,
    )
}

pub struct CameraExample {
    world: World,
}
impl Game for CameraExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        let sprite = emd.loader().sprite("bunny.png").unwrap();
        let e = self
            .world
            .spawn((sprite.clone(), Camera::default(), Transform::default()));
        self.world.make_active_camera(e).unwrap();
        self.world
            .spawn((sprite.clone(), Transform::from_translation((-180.0, 0.0))));
        self.world
            .spawn((sprite.clone(), Transform::from_translation((180.0, 0.0))));
        self.world
            .spawn((sprite.clone(), Transform::from_translation((0.0, 180.0))));
        self.world
            .spawn((sprite.clone(), Transform::from_translation((0.0, -180.0))));
    }

    fn update(&mut self, mut emd: Emerald) {
        let mut velocity = Vector2::new(0.0, 0.0);
        let speed = 100.0;
        let delta = emd.delta();
        if emd.input().is_key_pressed(KeyCode::A) {
            velocity.x = -speed * delta;
        } else if emd.input().is_key_pressed(KeyCode::D) {
            velocity.x = speed * delta;
        }
        if emd.input().is_key_pressed(KeyCode::W) {
            velocity.y = speed * delta;
        } else if emd.input().is_key_pressed(KeyCode::S) {
            velocity.y = -speed * delta;
        }

        for (_, transform) in self
            .world
            .query::<&mut Transform>()
            .with::<&Camera>()
            .iter()
        {
            transform.translation.x += velocity.x;
            transform.translation.y += velocity.y;
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
