use emerald::*;

pub fn main() {
    emerald::start(
        Box::new(WindowManipulationExample {}),
        GameSettings::default(),
    )
}

pub struct WindowManipulationExample {}
impl Game for WindowManipulationExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.graphics().set_fullscreen(true).unwrap();
    }

    fn update(&mut self, mut emd: Emerald) {
        if emd.input().is_key_just_pressed(KeyCode::A) {
            emd.graphics().set_window_size(600, 600).unwrap();
        }
        if emd.input().is_key_just_pressed(KeyCode::D) {
            emd.graphics().set_window_size(1200, 1200).unwrap();
        }
        if emd.input().is_key_just_pressed(KeyCode::Escape) {
            emd.graphics().set_fullscreen(false).unwrap();
        }
        if emd.input().is_key_just_pressed(KeyCode::Enter) {
            emd.graphics().set_fullscreen(true).unwrap();
        }
    }
}
