use rapier2d::na::Vector2;
use serde::{Deserialize, Serialize};

use crate::{
    texture::TextureKey,
    tilemap::{get_tilemap_index, TileId, Tilemap},
    Emerald, EmeraldError,
};

#[derive(Clone, Copy, Debug, PartialEq, Hash, Deserialize, Serialize)]
pub enum AutoTileRulesetValue {
    Any,
    None,
    Tile,
}

const AUTOTILE_RULESET_GRID_SIZE: usize = 5;

#[derive(Deserialize, Serialize)]
struct AutoTileRulesetSchema {
    /// id of the tile in the tileset this rule belongs to
    pub tile_id: TileId,

    /// List of tile rules in the ruleset, tiles not given in the 5x5 grid are assumed to be Any
    pub rules: Vec<AutoTileRulesetSchemaTile>,
}

#[derive(Deserialize, Serialize, Debug)]
struct AutoTileRulesetSchemaTile {
    /// Position of this tile rule in the 5x5 grid, cannot be greater than 4
    pub x: usize,

    /// Position of this tile rule in the 5x5 grid, cannot be greater than 4
    pub y: usize,

    /// ex. Any, None, Tile
    pub value: AutoTileRulesetValue,
}
impl AutoTileRulesetSchema {
    fn to_ruleset(self) -> Result<AutoTileRuleset, EmeraldError> {
        let mut grid = default_ruleset_grid();

        for tile in self.rules {
            if tile.x >= AUTOTILE_RULESET_GRID_SIZE || tile.y >= AUTOTILE_RULESET_GRID_SIZE {
                return Err(EmeraldError::new(format!(
                    "Tile {:?} does not fit inside of the 5x5 ruleset grid.",
                    tile
                )));
            }

            grid[tile.x][tile.y] = tile.value;
        }

        // We require the center of the grid to be the target tile.
        grid[2][2] = AutoTileRulesetValue::Tile;

        Ok(AutoTileRuleset {
            tile_id: self.tile_id,
            grid,
        })
    }
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
    pub tile_id: TileId,

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
struct AutoTileMapSchema {
    pub tileset: String,

    #[serde(default)]
    pub rulesets: Vec<AutoTileRulesetSchema>,

    pub tile_height: usize,
    pub tile_width: usize,
    pub map_width: usize,
    pub map_height: usize,

    #[serde(default)]
    pub z_index: f32,

    #[serde(default = "default_visibility")]
    pub visible: bool,
}
impl AutoTileMapSchema {
    pub fn to_autotilemap(self, emd: &mut Emerald) -> Result<AutoTilemap, EmeraldError> {
        let tileset = emd.loader().texture(self.tileset.clone())?;
        self.to_autotilemap_with_tileset(tileset)
    }

    pub fn to_autotilemap_with_tileset(
        self,
        tileset: TextureKey,
    ) -> Result<AutoTilemap, EmeraldError> {
        let rulesets = self
            .rulesets
            .into_iter()
            .map(|ruleset_schema| ruleset_schema.to_ruleset())
            .collect::<Result<Vec<AutoTileRuleset>, EmeraldError>>()?;
        let mut autotilemap = AutoTilemap::new(
            tileset,
            Vector2::new(self.tile_width, self.tile_height),
            self.map_width,
            self.map_height,
            rulesets,
        );

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
        map_width: usize,
        map_height: usize,
        rulesets: Vec<AutoTileRuleset>,
    ) -> Self {
        let tilemap = Tilemap::new(tilesheet, tile_size, map_width, map_height);

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
                self.tilemap.set_tile(x, y, self.compute_tile_id(x, y)?)?;
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
        self.rulesets
            .iter()
            .find(|ruleset| ruleset.tile_id == tile_id)
    }

    pub fn remove_ruleset(&mut self, tile_id: TileId) -> Option<AutoTileRuleset> {
        if let Some(index) = self
            .rulesets
            .iter()
            .position(|ruleset| ruleset.tile_id == tile_id)
        {
            return Some(self.rulesets.remove(index));
        }

        None
    }

    /// Computes the tileid for the given autotile position.
    pub fn compute_tile_id(&self, x: usize, y: usize) -> Result<Option<TileId>, EmeraldError> {
        if let Some(ruleset) = self
            .rulesets
            .iter()
            .find(|ruleset| ruleset.matches(&self.autotiles, self.width(), self.height(), x, y))
        {
            return Ok(Some(ruleset.tile_id));
        }

        Ok(None)
    }
    pub fn get_tile_id(&self, x: usize, y: usize) -> Result<Option<TileId>, EmeraldError> {
        self.tilemap.get_tile(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::{AutoTileMapSchema, AutoTileRulesetSchema, AutoTileRulesetValue};

    #[test]
    fn deser_ruleset() {
        let ruleset_toml = r#"
            tile_id = 10

            [[rules]]
            x = 0
            y = 0
            value = "None"

            [[rules]]
            x = 1
            y = 1
            value = "Tile"
        "#;
        let schema: AutoTileRulesetSchema = crate::toml::from_str(ruleset_toml).unwrap();
        let ruleset = schema.to_ruleset().unwrap();
        assert_eq!(ruleset.tile_id, 10);
        assert_eq!(ruleset.grid[0][0], AutoTileRulesetValue::None);
        assert_eq!(ruleset.grid[1][1], AutoTileRulesetValue::Tile);

        // Check target tile is a tile
        assert_eq!(ruleset.grid[2][2], AutoTileRulesetValue::Tile);
    }

    #[test]
    fn deser_autotilemap() {
        let autotilemap_toml = r#"
            tileset = "tileset.png"
            tile_width = 32
            tile_height = 32
            map_width = 10
            map_height = 10
        "#;
        let schema: AutoTileMapSchema = crate::toml::from_str(&autotilemap_toml).unwrap();
        let autotilemap = schema
            .to_autotilemap_with_tileset(Default::default())
            .unwrap();
        assert_eq!(autotilemap.width(), 10);
        assert_eq!(autotilemap.height(), 10);
        assert_eq!(autotilemap.tile_size().x, 32);
        assert_eq!(autotilemap.tile_size().y, 32);

        let missing_map_size = r#"
            tileset = "tileset.png"
            tile_width = 32
            tile_height = 32
        "#;
        let schema = crate::toml::from_str::<AutoTileMapSchema>(&missing_map_size);
        assert!(schema.is_err());

        let missing_tile_size = r#"
            tileset = "tileset.png"
            map_width = 10
            map_height = 10
        "#;
        let schema = crate::toml::from_str::<AutoTileMapSchema>(&missing_tile_size);
        assert!(schema.is_err());
    }
}
