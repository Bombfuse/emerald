use emerald::{
    rendering::components::{ColorRect, Sprite},
    *,
};

pub fn main() {
    let game = MouseExample {
        rect: ColorRect::new(BLACK, 0, 0),
        transform: Transform::default(),
        background: ColorRect::new(BLACK, 0, 0),
        screen_center: Translation::default(),
        world: World::new(),
    };
    emerald::start(Box::new(game), GameSettings::default())
}

pub struct MouseExample {
    rect: ColorRect,
    transform: Transform,
    background: ColorRect,
    screen_center: Translation,
    world: World,
}

impl Game for MouseExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        if let Ok(sprite) = emd.loader().sprite("bunny.png") {
            self.world
                .spawn((sprite, Transform::from_translation((16.0, 16.0))));
        }

        emd.touches_to_mouse(true);
    }

    fn update(&mut self, mut emd: Emerald) {
        let mouse = emd.input().mouse();
        let (width, height) = emd.screen_size();
        let translation = screen_translation_to_world_translation(
            (width, height),
            &mouse.translation,
            &self.world,
        );
        self.transform.translation.x = translation.x;
        self.transform.translation.y = translation.y;
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

        println!("Color should be {:?}", color);

        self.rect = ColorRect::new(color, 40, 40);

        self.screen_center = Translation::new(width as f32 / 2.0, height as f32 / 2.0);
        self.background = ColorRect::new(flash, width as u32, height as u32);

        for (_, (transform, _)) in self.world.query::<(&mut Transform, &mut Sprite)>().iter() {
            // It's important to convert coordinates to the physical world space.
            *transform = self.transform;
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();

        emd.graphics()
            .draw_color_rect(
                &self.background,
                &Transform::from_translation(self.screen_center),
            )
            .ok();
        emd.graphics()
            .draw_color_rect(&self.rect, &self.transform)
            .ok();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
