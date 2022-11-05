use crate::{texture::TextureKey, *};

pub type TileId = usize;

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
