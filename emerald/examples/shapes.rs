use emerald::{
    rendering::components::ColorTri, Emerald, Game, GameSettings, KeyCode, Transform, Vector2,
    World, BLACK, WHITE,
};
use rapier2d::prelude::{ConvexPolygon, Point};

pub fn main() {
    emerald::start(Box::new(ShapesExample {}), GameSettings::default())
}

pub struct ShapesExample {}
impl Game for ShapesExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root("./examples/assets/".to_string());
    }

    fn update(&mut self, emd: Emerald) {}

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
        emd.graphics()
            .draw_convex_polygon(
                &BLACK,
                &ConvexPolygon::from_convex_polyline(vec![
                    Point::new(0.0, 0.0),
                    Point::new(10.0, 0.0),
                    Point::new(20.0, 10.0),
                    Point::new(20.0, 20.0),
                    Point::new(10.0, 30.0),
                    Point::new(0.0, 30.0),
                    Point::new(-10.0, 20.0),
                    Point::new(-10.0, 10.0),
                ])
                .unwrap(),
                &Transform::from_translation((-50.0, 100.0)),
            )
            .unwrap();
        emd.graphics()
            .draw_convex_polygon(
                &BLACK,
                &ConvexPolygon::from_convex_polyline(vec![
                    Point::new(-10.0, 0.0),
                    Point::new(10.0, 10.0),
                    Point::new(10.0, 20.0),
                    Point::new(15.0, 40.0),
                ])
                .unwrap(),
                &Transform::from_translation((100.0, 100.0)),
            )
            .unwrap();
        emd.graphics().render().unwrap();
    }
}
