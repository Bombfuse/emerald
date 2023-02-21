use serde::{Deserialize, Serialize};

use crate::{texture::TextureKey, *};

pub type TileId = usize;

#[derive(Deserialize, Serialize)]
struct TileSchema {
    x: usize,
    y: usize,
    id: TileId,
}

#[derive(Deserialize, Serialize)]
struct TilemapSchema {
    tileset: String,
    width: usize,
    height: usize,
    tile_width: usize,
    tile_height: usize,
    #[serde(default = "default_visibility")]
    visible: bool,
    #[serde(default)]
    z_index: f32,

    #[serde(default)]
    tiles: Vec<TileSchema>,
}
impl TilemapSchema {
    pub fn to_tilemap(self, emd: &mut Emerald) -> Result<Tilemap, EmeraldError> {
        let tileset = emd.loader().texture(self.tileset.clone())?;
        self.to_tilemap_with_tileset(tileset)
    }

    pub fn to_tilemap_with_tileset(self, tileset: TextureKey) -> Result<Tilemap, EmeraldError> {
        let mut tilemap = Tilemap::new(
            tileset,
            Vector2::new(self.tile_width, self.tile_height),
            self.width,
            self.height,
        );
        for tile in self.tiles {
            tilemap.set_tile(tile.x, tile.y, Some(tile.id))?;
        }

        Ok(tilemap)
    }
}

fn default_visibility() -> bool {
    true
}

#[derive(Clone)]
pub struct Tilemap {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) tilesheet: TextureKey,
    pub(crate) tile_size: Vector2<usize>,
    pub(crate) tiles: Vec<Option<TileId>>,
    pub z_index: f32,
    pub visible: bool,
}
impl Tilemap {
    pub fn new(
        tilesheet: TextureKey,
        // Size of a tile in the grid, in pixels
        tile_size: Vector2<usize>,
        width: usize,
        height: usize,
    ) -> Self {
        let mut tiles = Vec::with_capacity(width * height);

        for _ in 0..(width * height) {
            tiles.push(None);
        }

        Tilemap {
            tile_size,
            tilesheet,
            height,
            width,
            tiles,
            z_index: 0.0,
            visible: true,
        }
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Result<Option<TileId>, EmeraldError> {
        let tile_index = get_tilemap_index(x, y, self.width, self.height)?;

        if let Some(tile) = self.tiles.get(tile_index) {
            let tile = tile.map(|id| id);

            return Ok(tile);
        }

        let err_msg = format!(
            "Position {:?} does not exist. Tilemap size is {}",
            (x, y),
            self.size()
        );

        Err(EmeraldError::new(err_msg))
    }

    pub fn set_tile(
        &mut self,
        x: usize,
        y: usize,
        new_tile: Option<TileId>,
    ) -> Result<(), EmeraldError> {
        let tile_index = get_tilemap_index(x, y, self.width, self.height)?;

        if let Some(tile_id) = self.tiles.get_mut(tile_index) {
            *tile_id = new_tile;

            return Ok(());
        }

        let err_msg = format!(
            "Position {:?} does not exist. Tilemap size is {}",
            (x, y),
            self.size()
        );

        Err(EmeraldError::new(err_msg))
    }

    pub fn size(&self) -> Vector2<usize> {
        Vector2::new(self.width, self.height)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn tile_width(&self) -> usize {
        self.tile_size.x
    }

    pub fn tile_height(&self) -> usize {
        self.tile_size.y
    }

    pub fn set_tilesheet(&mut self, tilesheet: TextureKey) {
        self.tilesheet = tilesheet
    }
}

pub(crate) fn get_tilemap_index(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
) -> Result<usize, EmeraldError> {
    if x >= width {
        return Err(EmeraldError::new(format!(
            "Given x: {} is outside the width of {}",
            x, width
        )));
    }

    if y >= height {
        return Err(EmeraldError::new(format!(
            "Given y: {} is outside the height of {}",
            y, height
        )));
    }

    Ok((y * width) + x)
}

#[cfg(test)]
mod tests {
    use crate::tilemap::{TileSchema, TilemapSchema};

    #[test]
    fn deser_tile() {
        let toml = r#"
            x = 3
            y = 6
            id = 10
        "#;
        let schema: TileSchema = crate::toml::from_str(toml).unwrap();
        assert_eq!(schema.id, 10);
        assert_eq!(schema.x, 3);
        assert_eq!(schema.y, 6);
    }

    #[test]
    fn deser_tilemap() {
        let toml = r#"
            tileset = "tileset.png"
            width = 10
            height = 10
            tile_width = 32
            tile_height = 32

            [[tiles]]
            id = 14
            x = 5
            y = 6
        "#;
        let schema: TilemapSchema = crate::toml::from_str(&toml).unwrap();
        let tilemap = schema.to_tilemap_with_tileset(Default::default()).unwrap();
        assert_eq!(tilemap.width(), 10);
        assert_eq!(tilemap.height(), 10);
        assert_eq!(tilemap.tile_width(), 32);
        assert_eq!(tilemap.tile_height(), 32);
        assert_eq!(tilemap.get_tile(5, 6).unwrap().unwrap(), 14);

        let missing_map_size = r#"
            tileset = "tileset.png"
            tile_width = 32
            tile_height = 32
        "#;
        let schema = crate::toml::from_str::<TilemapSchema>(&missing_map_size);
        assert!(schema.is_err());

        let missing_tile_size = r#"
            tileset = "tileset.png"
            map_width = 10
            map_height = 10
        "#;
        let schema = crate::toml::from_str::<TilemapSchema>(&missing_tile_size);
        assert!(schema.is_err());
    }
}
