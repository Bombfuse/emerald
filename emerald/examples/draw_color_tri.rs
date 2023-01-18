use emerald::{
    rendering::components::ColorTri, Emerald, Game, GameSettings, Transform, Vector2, World, BLACK,
    WHITE,
};

pub fn main() {
    emerald::start(Box::new(DrawColorTriExample {}), GameSettings::default())
}

pub struct DrawColorTriExample {}
impl Game for DrawColorTriExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root("./examples/assets/".to_string());
    }

    fn update(&mut self, _emd: Emerald) {}

    fn draw(&mut self, mut emd: Emerald<'_>) {
        let color_tri = ColorTri::new(
            WHITE,
            [
                Vector2::new(-10.0, -10.0),
                Vector2::new(10.0, -10.0),
                Vector2::new(0.0, 10.0),
            ],
        );
        emd.graphics().begin().unwrap();
        emd.graphics()
            .draw_color_tri(&color_tri, &Default::default())
            .unwrap();
        emd.graphics()
            .draw_color_tri(&color_tri, &Transform::from_translation((30.0, 30.0)))
            .unwrap();
        emd.graphics()
            .draw_color_tri(
                &ColorTri::new(
                    BLACK,
                    [
                        Vector2::new(-20.0, -10.0),
                        Vector2::new(10.0, -20.0),
                        Vector2::new(0.0, 20.0),
                    ],
                ),
                &Transform::from_translation((-50.0, -50.0)),
            )
            .unwrap();
        emd.graphics().render().unwrap();
    }
}
