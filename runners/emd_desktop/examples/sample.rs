use emerald::Game;

struct Sample {}
impl Game for Sample {
    fn initialize(&mut self, mut _emd: emerald::Emerald) {}

    fn update(&mut self, _emd: emerald::Emerald) {}

    fn draw(&mut self, mut emd: emerald::Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().render().unwrap();
    }
}

fn main() {
    emd_desktop::start(Box::new(Sample {}));
}
