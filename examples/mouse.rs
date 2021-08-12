use emerald::*;

pub fn main() {
    let game = MouseExample {
        rect: ColorRect::new(BLACK, 0, 0),
        position: Position::zero(),
        background: ColorRect::new(BLACK, 0, 0),
        screen_center: Position::zero(),
    };
    emerald::start(Box::new(game), GameSettings::default())
}

pub struct MouseExample {
    rect: ColorRect,
    position: Position,
    background: ColorRect,
    screen_center: Position,
}

impl Game for MouseExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        if let Ok(sprite) = emd.loader().sprite("bunny.png") {
            emd.world().spawn((sprite, Position::new(16.0, 16.0)));
        }

        emd.touches_to_mouse(true);
    }

    fn update(&mut self, mut emd: Emerald) {
        let mouse = emd.input().mouse();

        let (width, height) = emd.screen_size();
        self.position.x = mouse.position.x;
        self.position.y = mouse.position.y;

        let mut color = Color::new(0, 0, 0, 255);
        let mut flash = Color::new(128, 128, 128, 128);

        if mouse.left.is_pressed {
            color.r = 255;
        }
        if mouse.left.is_just_pressed() {
            flash.r = 192;
        }

        if mouse.middle.is_pressed {
            color.g = 255;
        }
        if mouse.middle.is_just_pressed() {
            flash.g = 192;
        }

        if mouse.right.is_pressed {
            color.b = 255;
        }
        if mouse.right.is_just_pressed() {
            flash.b = 192;
        }

        self.rect = ColorRect::new(color, 40, 40);

        self.screen_center = Position::new(width / 2.0, height / 2.0);
        self.background = ColorRect::new(flash, width as u32, height as u32);

        for (_, (pos, _)) in emd.world().query::<(&mut Position, &mut Sprite)>().iter() {
            // It's important to convert coordinates to the physical world space.
            *pos = self.position - self.screen_center;
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();

        emd.graphics()
            .draw_color_rect(&self.background, &self.screen_center);
        emd.graphics().draw_color_rect(&self.rect, &self.position);

        if let Some(mut world) = emd.pop_world() {
            emd.graphics().draw_world(&mut world).unwrap();
            emd.push_world(world);
        }

        emd.graphics().render().unwrap();
    }
}
