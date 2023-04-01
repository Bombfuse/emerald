use emerald::{
    audio::components::sound_player::SoundPlayer, render_settings::RenderSettings,
    rendering::components::Label, *,
};

/// Music found from https://opengameart.org/content/5-chiptunes-action
pub fn main() {
    let mut settings = GameSettings::default();
    let render_settings = RenderSettings {
        ..Default::default()
    };
    settings.render_settings = render_settings;
    emerald::start(
        Box::new(Example {
            world: World::new(),
        }),
        settings,
    )
}

pub struct Example {
    world: World,
}
impl Game for Example {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        let mut sound_player = SoundPlayer::new("sfx");
        sound_player.add_sound("test", emd.loader().sound("test_sound.wav").unwrap());
        self.world.spawn((sound_player,));
    }

    fn update(&mut self, mut emd: Emerald) {
        let volume = emd.audio().mixer("test").unwrap().get_volume().unwrap();
        if emd.input().is_key_just_pressed(KeyCode::A) {
            emd.audio()
                .mixer("test")
                .unwrap()
                .set_volume(volume - 0.1)
                .unwrap();
            emd.audio()
                .mixer("test2")
                .unwrap()
                .set_volume(volume - 0.1)
                .unwrap();
        } else if emd.input().is_key_just_pressed(KeyCode::D) {
            emd.audio()
                .mixer("test")
                .unwrap()
                .set_volume(volume + 0.1)
                .unwrap();
            emd.audio()
                .mixer("test2")
                .unwrap()
                .set_volume(volume + 0.1)
                .unwrap();
        }

        if emd.input().is_key_just_pressed(KeyCode::C) {
            for (_, player) in self.world.query::<&SoundPlayer>().iter() {
                player.play(&mut emd, "test").unwrap();
            }
        }

        if emd.input().is_key_just_pressed(KeyCode::Space) {
            let snd = emd.loader().sound("test_music.wav").unwrap();
            emd.audio()
                .mixer("test")
                .unwrap()
                .play_and_loop(&snd)
                .unwrap();
        }

        if emd.input().is_key_just_pressed(KeyCode::Z) {
            for _ in 0..10 {
                let snd = emd.loader().sound("test_sound.wav").unwrap();
                emd.audio().mixer("test2").unwrap().play(&snd).unwrap();
            }
        }
    }

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        let font = emd.loader().font("Roboto-Light.ttf", 48).unwrap();
        let volume = emd.audio().mixer("test").unwrap().get_volume().unwrap();

        let volume_label = Label::new(format!("Volume: {:05.2}", volume), font, 48);
        emd.graphics()
            .draw_label(&volume_label, &Transform::from_translation((240.0, 180.0)))
            .unwrap();

        emd.graphics().render().unwrap();
    }
}
