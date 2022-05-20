use std::{collections::HashMap, ops::Index};

use nalgebra::Vector2;

use crate::{
    tilemap::{TileId, Tilemap},
    EmeraldError, TextureKey,
};

pub enum AutoTileValue {
    None,
    Tile,
    Any,
}

pub struct AutoTileRuleset {
    pub tile_id: TileId,

    /// A grid determining the ruleset that displays this tile.
    /// Most grids will only need to cover a 3x3 area around the center tile,
    /// however we offer a 5x5 to cover larger rulesets.
    /// Ex 1.
    /// [
    ///     [None, None, None, None, None]
    ///     [None, None, None, None, None]
    ///     [None, None, THIS_TILE, None, None]
    ///     [None, None, None, None, None]
    ///     [None, None, None, None, None]
    /// ]
    /// The above grid displays the tile when it is completely alone, and not surrounded by any tiles.
    /// The value in the center of the ruleset grid is ignored, as this space is reserved for the AutoTile.
    ///
    /// Ex 2.
    /// [
    ///     [None, None, None, None, None]
    ///     [None, None, Any, None, None]
    ///     [None, None, THIS_TILE, None, None]
    ///     [None, None, Tile, None, None]
    ///     [None, None, None, None, None]
    /// ]
    /// This example will display the given tile, when there is a tile below the center and
    /// does not care about the tile above center.
    pub grid: [[AutoTileValue; 5]; 5],
}

pub struct Autotilemap {
    tilemap: Tilemap,
    pub z_index: f32,
    pub visible: bool,
    rulesets: Vec<AutoTileRuleset>,
    tiles: Vec<AutoTileValue>,
}
impl Autotilemap {
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
        let mut tiles = Vec::with_capacity(tile_count);
        for _ in 0..tile_count {
            tiles.push(AutoTileValue::None);
        }

        Autotilemap {
            tilemap,
            z_index: 0.0,
            visible: true,
            rulesets,
            tiles,
        }
    }

    /// Bakes the inner tileset in accordance to the Autotilemap
    /// feature(physics): Additionally bakes the colliders
    pub fn bake(&mut self) {}

    pub fn width(&self) -> usize {
        self.tilemap.width
    }

    pub fn height(&self) -> usize {
        self.tilemap.height
    }

    pub fn add_ruleset(&mut self, ruleset: AutoTileRuleset) {
        self.rulesets.push(ruleset);
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

    fn get_index(&self, x: usize, y: usize) -> Result<usize, EmeraldError> {
        if x >= self.width() {
            return Err(EmeraldError::new(format!(
                "{} is not within the width ({}) of the Autotilemap.",
                x,
                self.width()
            )));
        }

        if y >= self.height() {
            return Err(EmeraldError::new(format!(
                "{} is not within the height ({}) of the Autotilemap.",
                y,
                self.height()
            )));
        }

        Ok(y * self.width() + x)
    }
}
#[test]
fn test_autotiles() {}
