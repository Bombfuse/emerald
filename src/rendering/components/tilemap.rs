use crate::*;

#[derive(Clone, Debug)]
pub struct Tilemap {
    columns: u32,
    rows: u32,
    tilesheet: TextureKey,
    tile_size: (u32, u32),
    tiles: Vec<Vec<u32>>,
    pub z_index: i32,
}
impl Tilemap {
    pub(crate) fn new(
            columns: u32,
            rows: u32,
            tile_size: (u32, u32), 
            tilesheet: TextureKey,
            z_index: i32) -> Self {
        Tilemap {
            tile_size,
            tilesheet,
            rows,
            columns,
            tiles: Vec::new(),
            z_index: 0,
        }
    }

    pub fn set_tile(&mut self, new_tile: u32, position: (u32, u32)) -> Result<(), EmeraldError> {
        if let Some(column) = self.tiles.get_mut(position.0) {
            if let Some(tile) = column.get_mut(position.1) {
                *tile = new_tile;
                
                return Ok(());
            }
        }

        let err_msg = format!("Position {} does not exist. Tilemap size is {}", position, self.size());

        Err(EmeraldError::new(err_msg))
    }

    pub fn set_tilesheet(&mut self, tilesheet: TextureKey) {
        self.tilesheet = tilesheet
    }
}