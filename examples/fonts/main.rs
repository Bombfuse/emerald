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
        let label = emd.loader().label("Emerald Game Engine", font).unwrap();
        let position = Position::new(100.0, 100.0);

        emd.world().spawn((position, label));
    }
}
