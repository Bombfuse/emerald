use crate::*;

pub trait Game {
    fn initialize(&mut self, _emd: Emerald<'_>) {}
    fn update(&mut self, _emd: Emerald<'_>) {}
    fn draw(&mut self, mut emd: Emerald<'_>) {
        emd.graphics().begin().unwrap();

        if let Some(mut world) = emd.pop_world() {
            emd.graphics().draw_world(&mut world).unwrap();
            emd.push_world(world);
        }

        emd.graphics().render().unwrap();
    }
}
