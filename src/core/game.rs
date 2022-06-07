use crate::*;

pub trait Game {
    fn initialize(&mut self, mut _emd: Emerald<'_, '_, '_>) {}
    fn update(&mut self, _emd: Emerald<'_, '_, '_>) {}
    fn draw(&mut self, mut emd: Emerald<'_, '_, '_>) {
        emd.graphics().begin().unwrap();
        emd.graphics().render().unwrap();
    }
}
