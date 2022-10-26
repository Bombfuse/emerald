use emerald::{render_settings::RenderSettings, *};

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

const ACTION_ATTACK_P1: &str = "p1_attack";
const ACTION_ATTACK_P2: &str = "p2_attack";

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

        emd.input()
            .add_action_binding_button(&ACTION_ATTACK_P1.to_string(), Button::West, 0);

        emd.input()
            .add_action_binding_button(&ACTION_ATTACK_P2.to_string(), Button::West, 1);
    }

    fn update(&mut self, mut emd: Emerald) {
        if emd.input().is_action_just_pressed(&ACTION_TEST.to_string()) {
            println!("Test Action Pressed");
        }
        if emd
            .input()
            .is_action_just_pressed(&ACTION_ATTACK_P1.to_string())
        {
            println!("p1 attack");
        }
        if emd
            .input()
            .is_action_just_pressed(&ACTION_ATTACK_P2.to_string())
        {
            println!("p2 attack");
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().ok();
        emd.graphics().draw_world(&mut self.world).ok();
        emd.graphics().render().ok();
    }
}
