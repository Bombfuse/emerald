use hecs::Entity;
use rapier2d::na::Vector2;
use serde::{Deserialize, Serialize};

use crate::{
    texture::TextureKey,
    tilemap::{get_tilemap_index, TileId, Tilemap},
    AssetLoader, Emerald, EmeraldError, World,
};

#[derive(Deserialize, Serialize)]
struct AutoTileRulesetsResource {
    #[serde(default)]
    pub rulesets: Vec<AutoTileRulesetSchema>,
}

pub fn load_autotile_rulesets_from_resource<T: Into<String>>(
    emd: &mut Emerald,
    resource_path: T,
) -> Result<Vec<AutoTileRuleset>, EmeraldError> {
    let data = emd.loader().string(resource_path.into())?;
    let resource = crate::toml::from_str::<AutoTileRulesetsResource>(&data)?;
    let rulesets = resource
        .rulesets
        .into_iter()
        .map(|schema| schema.to_ruleset())
        .collect::<Result<Vec<AutoTileRuleset>, EmeraldError>>()?;

    Ok(rulesets)
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Deserialize, Serialize)]
pub enum AutoTileRulesetValue {
    Any,
    None,
    Tile,
}

const AUTOTILE_RULESET_GRID_SIZE: usize = 5;

#[derive(Deserialize, Serialize)]
struct AutoTileRulesetSchema {
    /// x position of the tile in the tileset this ruleset belongs to
    pub x: usize,
    /// y position of the tile in the tileset this ruleset belongs to
    pub y: usize,

    /// List of tile rules in the ruleset, tiles not given in the 5x5 grid are assumed to be Any
    #[serde(default)]
    pub rules: Vec<AutoTileRulesetSchemaTile>,
}
impl AutoTileRulesetSchema {
    fn to_ruleset(self) -> Result<AutoTileRuleset, EmeraldError> {
        let mut grid = default_ruleset_grid();
        let max_offset = (AUTOTILE_RULESET_GRID_SIZE / 2) as i8;

        for tile in self.rules {
            if tile.x < -max_offset
                || tile.x > max_offset
                || tile.y < -max_offset
                || tile.y > max_offset
            {
                return Err(EmeraldError::new(format!(
                    "Tile {:?} does not fit inside of the 5x5 ruleset grid.",
                    tile
                )));
            }

            let position = Vector2::new(
                (AUTOTILE_RULESET_GRID_SIZE / 2) as i8 + tile.x,
                (AUTOTILE_RULESET_GRID_SIZE / 2) as i8 + tile.y,
            );
            grid[position.x as usize][position.y as usize] = tile.value;
        }

        // We require the center of the grid to be the target tile.
        grid[2][2] = AutoTileRulesetValue::Tile;

        Ok(AutoTileRuleset {
            x: self.x,
            y: self.y,
            grid,
        })
    }
}

#[derive(Deserialize, Serialize, Debug)]
struct AutoTileRulesetSchemaTile {
    /// relative x position to the center tile of the rule grid
    pub x: i8,
    /// relative y position to the center tile of the rule grid
    pub y: i8,

    /// ex. Any, None, Tile
    pub value: AutoTileRulesetValue,
}

fn default_ruleset_grid(
) -> [[AutoTileRulesetValue; AUTOTILE_RULESET_GRID_SIZE]; AUTOTILE_RULESET_GRID_SIZE] {
    let default_row = [
        AutoTileRulesetValue::Any,
        AutoTileRulesetValue::Any,
        AutoTileRulesetValue::Any,
        AutoTileRulesetValue::Any,
        AutoTileRulesetValue::Any,
    ];
    [
        default_row,
        default_row,
        default_row,
        default_row,
        default_row,
    ]
}

#[derive(Deserialize, Serialize)]
pub struct AutoTileRuleset {
    /// X position of the autotile in the tileset
    pub x: usize,
    /// Y position of the autotile in the tileset
    pub y: usize,

    /// A grid determining the ruleset that displays this tile.
    /// Most grids will only need to cover a 3x3 area around the center tile,
    /// however we offer a 5x5 to cover larger rulesets. If you don't care to use all
    /// all of the 5x5 grid, place the outer rings with AutoTile::Any.
    ///
    /// These grids are rotated 90* clockwise visually.
    /// Ex 1.
    /// [
    ///     [Any, Any, Any, Any, Any]
    ///     [Any, None, None, None, Any]
    ///     [Any, None, Tile, None, Any]
    ///     [Any, None, None, None, Any]
    ///     [Any, Any, Any, Any, Any]
    /// ]
    /// The above grid displays the tile when it is completely alone, and not surrounded by any tiles.
    /// The value in the center of the ruleset grid is ignored, as this space is reserved for the AutoTile.
    ///
    /// Ex 2.
    /// [
    ///     [Any, Any, Any, Any, Any]
    ///     [Any, None, Any, None, Any]
    ///     [Any, None, Tile, None, Any]
    ///     [Any, None, Tile, None, Any]
    ///     [Any, Any, Any, Any, Any]
    /// ]
    /// This example will display the given tile, when there is a tile right of the center and
    /// does not care about the tile left of center.
    pub grid: [[AutoTileRulesetValue; AUTOTILE_RULESET_GRID_SIZE]; AUTOTILE_RULESET_GRID_SIZE],
}
impl AutoTileRuleset {
    /// Tests a 5x5 area centering on the given x, y values and determines if it's a match.
    pub(crate) fn matches(
        &self,
        autotiles: &Vec<AutoTile>,
        autotilemap_width: usize,
        autotilemap_height: usize,
        x: usize,
        y: usize,
    ) -> bool {
        match get_tilemap_index(x, y, autotilemap_width, autotilemap_height) {
            Err(_) => {
                return false;
            }
            Ok(index) => {
                if autotiles[index] != AutoTile::Tile {
                    return false;
                }
            }
        }

        for ruleset_x in 0..AUTOTILE_RULESET_GRID_SIZE {
            for ruleset_y in 0..AUTOTILE_RULESET_GRID_SIZE {
                // If center tile or any, skip
                if (ruleset_x == (AUTOTILE_RULESET_GRID_SIZE / 2)
                    && ruleset_y == (AUTOTILE_RULESET_GRID_SIZE / 2))
                    || self.grid[ruleset_x][ruleset_y] == AutoTileRulesetValue::Any
                {
                    continue;
                }

                let autotile_ruleset_value = self.get_autotile_ruleset_value(
                    autotiles,
                    autotilemap_width,
                    autotilemap_height,
                    x as isize - (AUTOTILE_RULESET_GRID_SIZE / 2) as isize + ruleset_x as isize,
                    y as isize - (AUTOTILE_RULESET_GRID_SIZE / 2) as isize + ruleset_y as isize,
                );

                if self.grid[ruleset_x][ruleset_y] != autotile_ruleset_value {
                    return false;
                }
            }
        }

        true
    }

    fn get_autotile_ruleset_value(
        &self,
        autotiles: &Vec<AutoTile>,
        autotilemap_width: usize,
        autotilemap_height: usize,
        x: isize,
        y: isize,
    ) -> AutoTileRulesetValue {
        // Treat tiles outside the boundaries of the map as Any.
        if x < 0 || y < 0 {
            return AutoTileRulesetValue::Any;
        }

        match get_tilemap_index(
            x as usize,
            y as usize,
            autotilemap_width,
            autotilemap_height,
        ) {
            Ok(index) => match autotiles[index] {
                AutoTile::None => AutoTileRulesetValue::None,
                AutoTile::Tile => AutoTileRulesetValue::Tile,
            },
            // Out of bounds, so we assume any
            Err(_) => AutoTileRulesetValue::Any,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub enum AutoTile {
    None = 0,
    Tile = 1,
}

fn default_visibility() -> bool {
    true
}

#[derive(Deserialize, Serialize)]
struct AutoTileSchema {
    x: usize,
    y: usize,
}

#[derive(Deserialize, Serialize)]
struct AutoTileMapSchema {
    pub tileset: String,

    /// Height of tileset in tiles
    pub tileset_height: usize,
    /// Width of tileset in tiles
    pub tileset_width: usize,
    pub tile_height_px: usize,
    pub tile_width_px: usize,

    /// Path to the rulesets resource.
    /// Appends onto any defined rulesets in this autotilemap.
    #[serde(default)]
    pub resource: Option<String>,

    #[serde(default)]
    pub rulesets: Vec<AutoTileRulesetSchema>,

    pub width: usize,
    pub height: usize,

    #[serde(default)]
    pub tiles: Vec<AutoTileSchema>,

    #[serde(default)]
    pub z_index: f32,

    #[serde(default = "default_visibility")]
    pub visible: bool,
}
impl AutoTileMapSchema {
    pub fn to_autotilemap(self, loader: &mut AssetLoader) -> Result<AutoTilemap, EmeraldError> {
        let tileset = loader.texture(self.tileset.clone())?;
        let mut rulesets = Vec::new();

        if let Some(resource) = &self.resource {
            let toml = loader.string(resource.clone())?;
            let resource = crate::toml::from_str::<AutoTileRulesetsResource>(&toml)?;
            rulesets = resource.rulesets;
        }

        self.to_autotilemap_ext(tileset, rulesets)
    }

    pub fn to_autotilemap_ext(
        self,
        tileset: TextureKey,
        extra_rulesets: Vec<AutoTileRulesetSchema>,
    ) -> Result<AutoTilemap, EmeraldError> {
        let mut ruleset_schemas = self.rulesets;
        ruleset_schemas.extend(extra_rulesets);

        let rulesets = ruleset_schemas
            .into_iter()
            .map(|ruleset_schema| ruleset_schema.to_ruleset())
            .collect::<Result<Vec<AutoTileRuleset>, EmeraldError>>()?;

        let mut autotilemap = AutoTilemap::new(
            tileset,
            Vector2::new(self.tile_width_px, self.tile_height_px),
            self.tileset_width,
            self.tileset_height,
            self.width,
            self.height,
            rulesets,
        );

        for tile in self.tiles {
            autotilemap.set_tile(tile.x, tile.y)?;
        }

        autotilemap.set_z_index(self.z_index);
        autotilemap.set_visible(self.visible);

        Ok(autotilemap)
    }
}

pub struct AutoTilemap {
    pub(crate) tilemap: Tilemap,
    rulesets: Vec<AutoTileRuleset>,
    autotiles: Vec<AutoTile>,
}
impl AutoTilemap {
    pub fn new(
        tilesheet: TextureKey,
        // Size of a tile in the grid, in pixels
        tile_size: Vector2<usize>,
        // Width of tilesheet in tiles
        tilesheet_width: usize,
        // Height of tilesheet in tiles
        tilesheet_height: usize,
        map_width: usize,
        map_height: usize,
        rulesets: Vec<AutoTileRuleset>,
    ) -> Self {
        let tilemap = Tilemap::new(
            tilesheet,
            tile_size,
            tilesheet_width,
            tilesheet_height,
            map_width,
            map_height,
        );

        let tile_count = map_width * map_height;
        let mut autotiles = Vec::with_capacity(tile_count);
        for _ in 0..tile_count {
            autotiles.push(AutoTile::None);
        }

        Self {
            tilemap,
            rulesets,
            autotiles,
        }
    }

    /// Bakes the inner tileset in accordance to the Autotilemap
    /// TODO: feature(physics): Additionally bakes the colliders
    pub fn bake(&mut self) -> Result<(), EmeraldError> {
        for x in 0..self.width() {
            for y in 0..self.height() {
                let id = self.compute_tileset_tile_id(x, y)?;
                self.tilemap.set_tile(x, y, id)?;
            }
        }

        Ok(())
    }

    pub fn set_z_index(&mut self, z_index: f32) {
        self.tilemap.z_index = z_index;
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.tilemap.visible = visible;
    }

    pub fn width(&self) -> usize {
        self.tilemap.width
    }

    pub fn height(&self) -> usize {
        self.tilemap.height
    }

    pub fn tilesheet(&self) -> TextureKey {
        self.tilemap.tilesheet.clone()
    }

    pub fn tile_size(&self) -> Vector2<usize> {
        self.tilemap.tile_size.clone()
    }

    pub fn add_ruleset(&mut self, ruleset: AutoTileRuleset) {
        self.rulesets.push(ruleset);
    }

    pub fn get_autotile(&mut self, x: usize, y: usize) -> Result<AutoTile, EmeraldError> {
        let index = get_tilemap_index(x, y, self.width(), self.height())?;
        Ok(self.autotiles[index])
    }

    pub fn set_tile(&mut self, x: usize, y: usize) -> Result<(), EmeraldError> {
        self.set_autotile(x, y, AutoTile::Tile)
    }

    pub fn set_none(&mut self, x: usize, y: usize) -> Result<(), EmeraldError> {
        self.set_autotile(x, y, AutoTile::None)
    }

    pub fn set_autotile(
        &mut self,
        x: usize,
        y: usize,
        new_tile_id: AutoTile,
    ) -> Result<(), EmeraldError> {
        let index = get_tilemap_index(x, y, self.width(), self.height())?;
        self.autotiles[index] = new_tile_id;

        Ok(())
    }

    pub fn tiles(&self) -> &Vec<Option<TileId>> {
        &self.tilemap.tiles
    }

    pub fn get_ruleset(&self, tile_id: TileId) -> Option<&AutoTileRuleset> {
        let width = self.width();
        let height = self.height();

        self.rulesets.iter().find(|ruleset| {
            if let Ok(id) = get_tilemap_index(ruleset.x, ruleset.y, width, height) {
                id == tile_id
            } else {
                false
            }
        })
    }

    pub fn remove_ruleset(&mut self, tile_id: TileId) -> Option<AutoTileRuleset> {
        let width = self.width();
        let height = self.height();

        if let Some(index) = self.rulesets.iter().position(|ruleset| {
            if let Ok(id) = get_tilemap_index(ruleset.x, ruleset.y, width, height) {
                id == tile_id
            } else {
                false
            }
        }) {
            return Some(self.rulesets.remove(index));
        }

        None
    }

    /// Computes the tileid for the given autotile position.
    pub fn compute_tileset_tile_id(
        &self,
        x: usize,
        y: usize,
    ) -> Result<Option<TileId>, EmeraldError> {
        let width = self.width();
        let height = self.height();

        if let Some(ruleset) = self
            .rulesets
            .iter()
            .find(|ruleset| ruleset.matches(&self.autotiles, width, height, x, y))
        {
            let id = get_tilemap_index(
                ruleset.x,
                ruleset.y,
                self.tilemap.tilesheet_width,
                self.tilemap.tilesheet_height,
            )?;
            return Ok(Some(id));
        }

        Ok(None)
    }
    pub fn get_tile_id(&self, x: usize, y: usize) -> Result<Option<TileId>, EmeraldError> {
        self.tilemap.get_tile(x, y)
    }
}

pub(crate) fn load_ent_autotilemap<'a>(
    loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: &toml::Value,
) -> Result<(), EmeraldError> {
    let schema: AutoTileMapSchema = toml::from_str(&toml.to_string())?;
    let mut autotilemap = schema.to_autotilemap(loader)?;
    autotilemap.bake()?;
    world.insert_one(entity, autotilemap)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{AutoTileMapSchema, AutoTileRulesetSchema, AutoTileRulesetValue};

    #[test]
    fn deser_ruleset() {
        let ruleset_toml = r#"
            x = 10
            y = 11

            [[rules]]
            x = -1
            y = -1
            value = "None"

            [[rules]]
            x = -1
            y = 0
            value = "None"

            [[rules]]
            x = 1
            y = 1
            value = "Tile"
        "#;
        let schema: AutoTileRulesetSchema = crate::toml::from_str(ruleset_toml).unwrap();
        let ruleset = schema.to_ruleset().unwrap();
        assert_eq!(ruleset.x, 10);
        assert_eq!(ruleset.y, 11);
        assert_eq!(ruleset.grid[1][1], AutoTileRulesetValue::None);
        assert_eq!(ruleset.grid[1][2], AutoTileRulesetValue::None);
        assert_eq!(ruleset.grid[3][3], AutoTileRulesetValue::Tile);

        // Check target tile is a tile
        assert_eq!(ruleset.grid[2][2], AutoTileRulesetValue::Tile);

        let out_of_bounds_ruleset = r#"
            x = 10
            y = 11

            [[rules]]
            x = -3
            y = 0
            value = "None"
        "#;
        let schema: AutoTileRulesetSchema = crate::toml::from_str(out_of_bounds_ruleset).unwrap();
        assert!(schema.to_ruleset().is_err());
    }

    // #[test]
    // fn deser_autotilemap() {
    //     let autotilemap_toml = r#"
    //         tileset = "tileset.png"
    //         tileset_height = 2
    //         tileset_width = 2
    //         tile_width_px = 32
    //         tile_height_px = 32
    //         width = 10
    //         height = 10
    //     "#;
    //     let schema: AutoTileMapSchema = crate::toml::from_str(&autotilemap_toml).unwrap();
    //     let autotilemap = schema
    //         .to_autotilemap_ext(Default::default(), Vec::new())
    //         .unwrap();
    //     assert_eq!(autotilemap.width(), 10);
    //     assert_eq!(autotilemap.height(), 10);
    //     assert_eq!(autotilemap.tile_size().x, 32);
    //     assert_eq!(autotilemap.tile_size().y, 32);

    //     let missing_map_size = r#"
    //         tileset = "tileset.png"
    //         tile_width = 32
    //         tile_height = 32
    //     "#;
    //     let schema = crate::toml::from_str::<AutoTileMapSchema>(&missing_map_size);
    //     assert!(schema.is_err());

    //     let missing_tile_size = r#"
    //         tileset = "tileset.png"
    //         width = 10
    //         height = 10
    //     "#;
    //     let schema = crate::toml::from_str::<AutoTileMapSchema>(&missing_tile_size);
    //     assert!(schema.is_err());
    // }
}
