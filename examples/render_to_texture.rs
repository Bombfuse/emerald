use emerald::*;

const RES_WIDTH: usize = 320;
const RES_HEIGHT: usize = 160;

pub fn main() {
    let mut settings = GameSettings::default();
    let mut render_settings = RenderSettings::default();
    render_settings.resolution = (320 * 2, 160 * 2);
    // render_settings.scalar = ScreenScalar::Stretch;
    settings.render_settings = render_settings;

    emerald::start(Box::new(MyGame { pos: Position::new(320.0, 160.0), scale: 1.0 }), settings)
}

pub struct MyGame {
    pos: Position,
    scale: f32,
}
impl Game for MyGame {
    fn update(&mut self, mut emd: Emerald) {
        let mut input = emd.input();
        let delta = emd.delta();
        let speed = 150.0;

        if input.is_key_pressed(KeyCode::Left) {
            self.pos.x -= speed * delta;
        }

        if input.is_key_pressed(KeyCode::Right) {
            self.pos.x += speed * delta;
        }

        if input.is_key_pressed(KeyCode::Up) {
            self.pos.y += speed * delta;
        }

        if input.is_key_pressed(KeyCode::Down) {
            self.pos.y -= speed * delta;
        }

        if input.is_key_just_pressed(KeyCode::A) {
            self.scale *= 0.5;
        }

        if input.is_key_just_pressed(KeyCode::S) {
            self.scale *= 2.0;
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        let now = std::time::Instant::now();
        emd.graphics().begin_texture_new(RES_WIDTH, RES_HEIGHT);

        let rabbit = emd.loader().sprite("./examples/assets/bunny.png").unwrap();
        emd.graphics().draw_color_rect(&ColorRect::new(WHITE, 500 * 500, 500 * 500), &self.pos);
        emd.graphics().draw_sprite(&rabbit, &Position::new((RES_WIDTH / 2) as f32, (RES_HEIGHT / 2) as f32));

        let texture_key = emd.graphics().render_texture().unwrap();
        
        let mut sprite = Sprite::from_texture(texture_key);
        sprite.scale.x = self.scale;
        sprite.scale.y = self.scale;

        println!("{:?}", sprite);

        emd.graphics().begin();
        emd.graphics().draw_sprite(&sprite, &self.pos);
        emd.graphics().render();
        let e = std::time::Instant::now();

        println!("{:?}", e - now);
    }
}
