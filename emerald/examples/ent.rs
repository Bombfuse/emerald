use emerald::{ent::EntLoadConfig, rendering::components::aseprite_update_system, *};

pub fn main() {
    emerald::start(
        Box::new(EntLoadingExample {
            world: World::new(),
        }),
        GameSettings::default(),
    )
}

struct PlayerData {
    pub name: String,
    pub max_hp: i64,
}

fn custom_component_loader(
    loader: &mut AssetLoader<'_>,
    entity: Entity,
    world: &mut World,
    toml_value: toml::Value,
    toml_key: String,
) -> Result<(), EmeraldError> {
    // We want to match here because in a real game we'll probably have many custom components
    match toml_key.as_str() {
        "my_custom_player_component" => {
            let name = toml_value
                .get("name")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string();
            let max_hp = toml_value.get("max_hp").unwrap().as_integer().unwrap();

            world
                .insert_one(entity, PlayerData { max_hp, name })
                .unwrap();
        }
        _ => {}
    }

    Ok(())
}

pub struct EntLoadingExample {
    world: World,
}
impl Game for EntLoadingExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root("./examples/assets/".to_string());
        emd.loader()
            .set_custom_component_loader(custom_component_loader);

        let entity = emd
            .loader()
            .ent(&mut self.world, "bunny.ent", Transform::default())
            .unwrap();

        // assert that we've successfully loaded a user defined component
        assert!(self.world.get::<&PlayerData>(entity).is_ok());
    }

    fn update(&mut self, emd: Emerald) {
        aseprite_update_system(&mut self.world, emd.delta());
    }

    fn draw(&mut self, mut emd: Emerald<'_>) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics()
            .draw_colliders(&mut self.world, Color::new(255, 0, 0, 100))
            .unwrap();
        emd.graphics().render().unwrap();
    }
}
