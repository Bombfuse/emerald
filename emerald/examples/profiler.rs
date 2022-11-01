use emerald::*;

pub fn main() {
    emerald::start(
        Box::new(ProfilingExample {
            world: World::new(),
        }),
        GameSettings::default(),
    )
}

pub struct ProfilingExample {
    world: World,
}
impl Game for ProfilingExample {
    fn initialize(&mut self, mut emd: Emerald) {
        let profile_initialize = "initialize";
        emd.profiler(profile_initialize).start_frame().unwrap();

        emd.set_asset_folder_root(String::from("./examples/assets/"));
        let sprite = emd.loader().sprite("bunny.png").unwrap();
        self.world.spawn((sprite, Transform::default()));

        let initialize_time = emd.profiler(profile_initialize).finish_frame().unwrap();
        println!("initialize time: {}", initialize_time);
    }

    fn update(&mut self, mut emd: Emerald) {
        emd.profiler("update").start_frame().unwrap();

        let update_time = emd.profiler("update").finish_frame().unwrap();
        println!("update time: {}", update_time);
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.profiler("draw").start_frame().unwrap();

        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();

        let draw_time = emd.profiler("draw").finish_frame().unwrap();
        println!("draw time: {}", draw_time);
    }
}
