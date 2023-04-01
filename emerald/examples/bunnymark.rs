use emerald::{
    font::FontKey,
    rendering::components::{Label, Sprite},
    *,
};

// Bunnymark is super disappointing right now, need to fix
// https://github.com/Bombfuse/emerald/issues/10

#[derive(Clone, Debug)]
struct Velocity {
    pub x: f32,
    pub y: f32,
}
impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Velocity { x, y }
    }
}

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
pub fn main() {
    let mut settings = GameSettings::default();
    settings.render_settings.resolution = (320 * 5, 180 * 5);
    emerald::start(
        Box::new(BunnymarkGame {
            count: 0,
            world: World::new(),
            fps_label: None,
            bunnymark_label: None,
        }),
        settings,
    )
}

pub struct BunnymarkGame {
    count: u64,
    world: World,
    fps_label: Option<Label>,
    bunnymark_label: Option<Label>,
}
impl Game for BunnymarkGame {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        let sprite = emd.loader().sprite("bunny.png").unwrap();

        let mut transform = Transform::default();

        let start = 1_000;
        self.count = start;
        self.world.spawn_batch((0..start).map(|_| {
            transform.translation.x += 1.0;
            transform.translation.y += 1.0;
            (transform, sprite.clone(), Velocity::new(5.0, 3.0))
        }));

        let font = emd.loader().font("Roboto-Light.ttf", 40).unwrap();
        self.bunnymark_label = Some(Label::new("", font.clone(), 40));
        self.fps_label = Some(Label::new("", font, 40));
    }

    #[inline]
    fn update(&mut self, mut emd: Emerald) {
        let now = std::time::Instant::now();
        let (screen_width, screen_height) = emd.screen_size();
        let screen_width = screen_width as f32;
        let screen_height = screen_height as f32;

        let sprite_width = 32.0;

        if emd.input().is_key_just_pressed(KeyCode::Space) {
            let mut sprite = emd.loader().sprite("bunny.png").unwrap();
            sprite.offset = Vector2::new(-10.0, 0.0);

            let mut transform = Transform::default();
            self.count += 1000;
            self.world.spawn_batch((0..1000).map(|_| {
                transform.translation.x += 6.0;
                transform.translation.y += 1.0;

                (transform, sprite.clone(), Velocity::new(5.0, 3.0))
            }));
        }

        for (_, (_, transform, mut vel)) in self
            .world
            .query::<(&Sprite, &mut Transform, &mut Velocity)>()
            .iter()
        {
            if transform.translation.x >= screen_width / 2.0 - sprite_width / 2.0 {
                transform.translation.x = screen_width / 2.0 - sprite_width / 2.0;
                vel.x *= -1.0;
            }

            if transform.translation.x <= -screen_width / 2.0 {
                transform.translation.x = -screen_width / 2.0;
                vel.x *= -1.0;
            }

            if transform.translation.y >= screen_height / 2.0 - sprite_width {
                transform.translation.y = screen_height / 2.0 - sprite_width;
                vel.y = -3.0;
            }

            if transform.translation.y <= -screen_height / 2.0 {
                transform.translation.y = -screen_height / 2.0;
                vel.y = 3.0;
            }

            transform.translation.x += vel.x;
            transform.translation.y += vel.y;
        }
        println!("update {:?}", std::time::Instant::now() - now);
    }

    fn draw(&mut self, mut emd: Emerald) {
        let now = std::time::Instant::now();
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();

        self.fps_label.as_mut().unwrap().text = format!("FPS: {}", emd.fps() as usize);
        self.bunnymark_label.as_mut().unwrap().text = format!("{} bunnies", (self.count));
        emd.graphics()
            .draw_label(
                &self.fps_label.as_ref().unwrap(),
                &Transform::from_translation((-300.0, 150.0)),
            )
            .unwrap();
        emd.graphics()
            .draw_label(
                &self.bunnymark_label.as_ref().unwrap(),
                &Transform::from_translation((-300.0, 100.0)),
            )
            .unwrap();

        emd.graphics().render().unwrap();
        println!("draw {:?}", std::time::Instant::now() - now);
    }
}
