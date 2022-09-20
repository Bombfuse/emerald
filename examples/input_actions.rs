use emerald::*;

pub fn main() {
    let mut settings = GameSettings::default();
    let render_settings = RenderSettings {
        resolution: (320 * 3, 180 * 3),
        ..Default::default()
    };
    settings.render_settings = render_settings;
    emerald::start(
        Box::new(InputActionsExample {
            world: World::new(),
        }),
        settings,
    )
}

const ACTION_TEST: &str = "test_action";

pub struct InputActionsExample {
    world: World,
}
impl Game for InputActionsExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));
        emd.input()
            .add_action_binding_key(&ACTION_TEST.to_string(), KeyCode::A);
        emd.input()
            .add_action_binding_key(&ACTION_TEST.to_string(), KeyCode::P);
        emd.input()
            .add_action_binding_key(&ACTION_TEST.to_string(), KeyCode::G);
    }

    fn update(&mut self, mut emd: Emerald) {
        if emd.input().is_action_just_pressed(&ACTION_TEST.to_string()) {
            println!("Test Action Pressed");
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().ok();
        emd.graphics().draw_world(&mut self.world).ok();
        emd.graphics().render().ok();
    }
}
