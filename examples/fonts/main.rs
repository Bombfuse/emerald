use emerald::*;

pub fn main() {
    emerald::start(Box::new(FontsExample {}), GameSettings::default())
}

pub struct FontsExample;
impl Game for FontsExample {
    fn initialize(&mut self, mut emd: Emerald) {
        let font = emd
            .loader()
            .font("./examples/assets/Roboto-Light.ttf")
            .unwrap();
        let label = Label::new("Emerald Game Engine", font);
        let position = Position::new(100.0, 100.0);

        let e = emd.world().spawn((position, label, Camera::default()));

        emd.make_active_camera(e);
    }
}
