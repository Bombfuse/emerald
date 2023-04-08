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
pub struct TilesetResource {
    pub texture: String,
    /// Height in tiles
    pub height: usize,
    /// Width in tiles
    pub width: usize,
}

fn load_tileset_resource<T: Into<String>>(
    emd: &mut Emerald,
    resource_path: T,
) -> Result<TilesetResource, EmeraldError> {
    let data = emd.loader().string(resource_path.into())?;
    let resource = crate::toml::from_str::<TilesetResource>(&data)?;

    Ok(resource)
}

#[derive(Deserialize, Serialize)]
struct TilemapSchema {
    width: usize,
    height: usize,
    #[serde(default)]
    tileset: Option<TilesetResource>,
    #[serde(default)]
    resource: Option<String>,
    #[serde(default = "default_visibility")]
    visible: bool,
    #[serde(default)]
    z_index: f32,

    /// Takes a list of tiles, rather than a grid for usability.
    #[serde(default)]
    tiles: Vec<TileSchema>,
}
impl TilemapSchema {
    pub fn to_tilemap(self, loader: &mut AssetLoader) -> Result<Tilemap, EmeraldError> {
        if self.tileset.is_none() && self.resource.is_none() {
            return Err(EmeraldError::new(
                "Tilemaps require either a tileset texture or a path to a tileset resource.",
            ));
        }

        let resource = if let Some(resource) = self.tileset {
            resource
        } else {
            let data = loader.string(&self.resource.unwrap())?;
            crate::toml::from_str::<TilesetResource>(&data)?
        };

        let texture = loader.texture(resource.texture.clone())?;
        let tile_size = Vector2::new(
            texture.size().0 as usize / resource.width,
            texture.size().1 as usize / resource.height,
        );
        let mut tilemap = Tilemap::new(
            texture,
            tile_size,
            resource.width,
            resource.height,
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
    // Width of tilesheet in tiles
    pub(crate) tilesheet_width: usize,
    // Height of tilesheet in tiles
    pub(crate) tilesheet_height: usize,
    pub z_index: f32,
    pub visible: bool,
}
impl Tilemap {
    pub fn new(
        tilesheet: TextureKey,
        // Size of a tile in the grid, in pixels
        tile_size: Vector2<usize>,
        // Width of tilesheet in tiles
        tilesheet_width: usize,
        // Height of tilesheet in tiles
        tilesheet_height: usize,
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
            tilesheet_height,
            tilesheet_width,
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

pub(crate) fn load_ent_tilemap<'a>(
    loader: &mut AssetLoader<'a>,
    entity: Entity,
    world: &mut World,
    toml: &toml::Value,
) -> Result<(), EmeraldError> {
    let schema: TilemapSchema = toml::from_str(&toml.to_string())?;
    let tilemap = schema.to_tilemap(loader)?;
    world.insert_one(entity, tilemap)?;

    Ok(())
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

    // #[test]
    // fn deser_tilemap() {
    //     let toml = r#"
    //         tileset = "tileset.png"
    //         tileset_width = 2
    //         tileset_height = 2

    //         width = 10
    //         height = 10
    //         tile_width_px = 32
    //         tile_height_px = 32

    //         [[tiles]]
    //         id = 14
    //         x = 5
    //         y = 6
    //     "#;
    //     let schema: TilemapSchema = crate::toml::from_str(&toml).unwrap();
    //     let tilemap = schema.to_tilemap_ext(Default::default()).unwrap();
    //     assert_eq!(tilemap.width(), 10);
    //     assert_eq!(tilemap.height(), 10);
    //     assert_eq!(tilemap.tile_width(), 32);
    //     assert_eq!(tilemap.tile_height(), 32);
    //     assert_eq!(tilemap.get_tile(5, 6).unwrap().unwrap(), 14);

    //     let missing_map_size = r#"
    //         tileset = "tileset.png"
    //         tileset_width = 2
    //         tileset_height = 2
    //         tile_width = 32
    //         tile_height = 32
    //     "#;
    //     let schema = crate::toml::from_str::<TilemapSchema>(&missing_map_size);
    //     assert!(schema.is_err());

    //     let missing_tile_size = r#"
    //         tileset = "tileset.png"
    //         tileset_width = 2
    //         tileset_height = 2
    //         map_width = 10
    //         map_height = 10
    //     "#;
    //     let schema = crate::toml::from_str::<TilemapSchema>(&missing_tile_size);
    //     assert!(schema.is_err());
    // }
}
