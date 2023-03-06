use emerald::{
    autotilemap::{AutoTile, AutoTileRuleset, AutoTileRulesetValue, AutoTilemap},
    rendering::components::Camera,
    Emerald, Game, GameSettings, Transform, Vector2, World,
};

pub fn main() {
    emerald::start(
        Box::new(AutotilemapExample {
            world: World::new(),
        }),
        GameSettings::default(),
    )
}

pub struct AutotilemapExample {
    world: World,
}
impl Game for AutotilemapExample {
    fn initialize(&mut self, mut emd: Emerald) {
        emd.set_asset_folder_root(String::from("./examples/assets/"));

        let rulesets = vec![
            // An autotile ruleset that only places the first tile when it is not immediately surrounded by other tiles.
            AutoTileRuleset {
                x: 0,
                y: 0,
                grid: [
                    // Column 1
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                    ],
                    // Column 2
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::None,
                        AutoTileRulesetValue::None,
                        AutoTileRulesetValue::None,
                        AutoTileRulesetValue::Any,
                    ],
                    // Column 3
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::None,
                        AutoTileRulesetValue::Tile,
                        AutoTileRulesetValue::None,
                        AutoTileRulesetValue::Any,
                    ],
                    // Column 4
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::None,
                        AutoTileRulesetValue::None,
                        AutoTileRulesetValue::None,
                        AutoTileRulesetValue::Any,
                    ],
                    // Column 5
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                    ],
                ],
            },
            // An autotile ruleset that only places the second tile when it is surrounded by tiles immediately
            AutoTileRuleset {
                x: 1,
                y: 0,
                grid: [
                    // Column 1
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                    ],
                    // Column 2
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Tile,
                        AutoTileRulesetValue::Tile,
                        AutoTileRulesetValue::Tile,
                        AutoTileRulesetValue::Any,
                    ],
                    // Column 3
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Tile,
                        AutoTileRulesetValue::Tile,
                        AutoTileRulesetValue::Tile,
                        AutoTileRulesetValue::Any,
                    ],
                    // Column 4
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Tile,
                        AutoTileRulesetValue::Tile,
                        AutoTileRulesetValue::Tile,
                        AutoTileRulesetValue::Any,
                    ],
                    // Column 5
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                    ],
                ],
            },
            // An autotile ruleset that places the third tile in any other scenario
            AutoTileRuleset {
                x: 1,
                y: 0,
                grid: [
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                    ],
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                    ],
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Tile,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                    ],
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                    ],
                    [
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                        AutoTileRulesetValue::Any,
                    ],
                ],
            },
        ];

        let tileset_key = emd.loader().texture("tileset.png").unwrap();
        let mut autotilemap =
            AutoTilemap::new(tileset_key, Vector2::new(16, 16), 2, 2, 20, 20, rulesets);

        autotilemap.set_tile(0, 0).unwrap(); // tile_id 2
        autotilemap.set_tile(10, 4).unwrap(); // tile_id 2
        autotilemap.set_tile(11, 4).unwrap(); // tile_id 2
        autotilemap.set_tile(19, 4).unwrap(); // tile_id 0
        autotilemap.set_tile(10, 10).unwrap(); // tile_id 0

        autotilemap.set_tile(15, 15).unwrap();
        autotilemap.set_tile(16, 15).unwrap();
        autotilemap.set_tile(17, 15).unwrap();
        autotilemap.set_tile(15, 16).unwrap();
        autotilemap.set_tile(16, 16).unwrap(); // we should see tile_id 1 here
        autotilemap.set_tile(17, 16).unwrap();
        autotilemap.set_tile(15, 17).unwrap();
        autotilemap.set_tile(16, 17).unwrap();
        autotilemap.set_tile(17, 17).unwrap();

        autotilemap.bake().unwrap();

        let e = self
            .world
            .spawn((autotilemap, Transform::default(), Camera::default()));
        self.world.make_active_camera(e).unwrap();
    }

    fn update(&mut self, _emd: Emerald) {}

    fn draw(&mut self, mut emd: Emerald) {
        emd.graphics().begin().unwrap();
        emd.graphics().draw_world(&mut self.world).unwrap();
        emd.graphics().render().unwrap();
    }
}
