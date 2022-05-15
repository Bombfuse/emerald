use emerald::*;

const RES_WIDTH: usize = 320;
const RES_HEIGHT: usize = 160;

pub fn main() {
    let mut settings = GameSettings::default();
    let render_settings = RenderSettings {
        resolution: (320 * 2, 160 * 2),
        ..Default::default()
    };
    settings.render_settings = render_settings;

    emerald::start(
        Box::new(MyGame {
            pos: Transform::from_translation((320.0, 160.0)),
            scale: 1.0,
            render_texture: None,
        }),
        settings,
    )
}

pub struct MyGame {
    pos: Transform,
    scale: f32,
    render_texture: Option<TextureKey>,
}
impl Game for MyGame {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        self.render_texture = Some(
            emd.loader()
                .render_texture(RES_WIDTH as usize, RES_HEIGHT as usize)
                .unwrap(),
        );
    }

    fn update(&mut self, mut emd: Emerald) {
        let mut input = emd.input();
        let delta = emd.delta();
        let speed = 150.0;

        if input.is_key_pressed(KeyCode::Left) {
            self.pos.translation.x -= speed * delta;
        }

        if input.is_key_pressed(KeyCode::Right) {
            self.pos.translation.x += speed * delta;
        }

        if input.is_key_pressed(KeyCode::Up) {
            self.pos.translation.y += speed * delta;
        }

        if input.is_key_pressed(KeyCode::Down) {
            self.pos.translation.y -= speed * delta;
        }

        if input.is_key_just_pressed(KeyCode::A) {
            self.scale *= 0.5;
        }

        if input.is_key_just_pressed(KeyCode::S) {
            self.scale *= 2.0;
        }

        println!("pos {:?}", self.pos);
    }

    fn draw(&mut self, mut emd: Emerald) {
        let now = std::time::Instant::now();
        emd.graphics()
            .begin_texture(self.render_texture.as_ref().unwrap().clone())
            .unwrap();

        let rabbit = emd.loader().sprite("bunny.png").unwrap();
        emd.graphics()
            .draw_color_rect(
                &ColorRect::new(WHITE, 500 * 500, 500 * 500),
                &Transform::from_translation(((RES_WIDTH / 2) as f32, (RES_HEIGHT / 2) as f32)),
            )
            .ok();
        emd.graphics()
            .draw_sprite(
                &rabbit,
                &Transform::from_translation(((RES_WIDTH / 2) as f32, (RES_HEIGHT / 2) as f32)),
            )
            .ok();

        let texture_key = emd.graphics().render_texture().unwrap();

        let e = std::time::Instant::now();

        println!("texture render: {:?}", e - now);

        // println!("{:?}", screen_sprite);
        let now = std::time::Instant::now();

        let e = std::time::Instant::now();
        let mut screen_sprite = Sprite::from_texture(texture_key);
        screen_sprite.centered = false;
        screen_sprite.scale.x = self.scale;
        screen_sprite.scale.y = self.scale;

        emd.graphics().begin().unwrap();
        emd.graphics().draw_sprite(&screen_sprite, &self.pos).ok();
        emd.graphics().render().unwrap();

        println!("screen draw: {:?}", e - now);
    }
}
