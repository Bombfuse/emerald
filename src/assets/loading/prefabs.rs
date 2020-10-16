use crate::*;
use nanoserde::*;

#[derive(Clone, Debug, DeJson)]
struct WorldPrefab {
    pub entities: Vec<Vec<ComponentPrefab>>,
}

#[derive(Clone, Debug, DeJson)]
enum ComponentPrefab {
    Sprite { path: String, offset: Vec2Prefab },
    Position { x: f32, y: f32 },
}

#[derive(Clone, Debug, DeJson)]
struct Vec2Prefab {
    pub x: f32,
    pub y: f32,
}

pub fn deserialize_world_from_json(json: &String, loader: &mut AssetLoader) -> Result<EmeraldWorld, EmeraldError> {
    let mut new_world = EmeraldWorld::new();
    let world_prefab: WorldPrefab = DeJson::deserialize_json(json)?;
    
    for entity_prefab in world_prefab.entities {
        let entity = new_world.spawn((Position::new(0.0, 0.0), ));

        for component_prefab in entity_prefab {
            match component_prefab {
                ComponentPrefab::Sprite { path, offset } => {
                    let mut sprite = loader.sprite(path)?;
                    let offset = Vector2::new(offset.x, offset.y);
                    sprite.offset = offset;
                    new_world.insert(entity.clone(), (sprite,))?;
                },
                ComponentPrefab::Position { x, y } => {
                    new_world.insert(entity.clone(), (Position::new(x, y),))?;
                },
            }
        }
    }

    Ok(new_world)
}