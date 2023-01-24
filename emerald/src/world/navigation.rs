use std::collections::HashMap;

use rapier2d::{na::Vector2, prelude::ConvexPolygon};

/// An id corresponding to a polygon in the navigation mesh.
pub type NavigationMeshPolygonId = usize;

pub struct NavigationMesh {
    polygons: HashMap<NavigationMeshPolygonId, ConvexPolygon>,
    polygon_uid: NavigationMeshPolygonId,

    bake_on_change: bool,
}
impl NavigationMesh {
    pub fn new() -> Self {
        NavigationMesh {
            polygons: HashMap::new(),
            polygon_uid: 0,
            bake_on_change: true,
        }
    }

    pub fn add_polygon(&mut self, polygon: ConvexPolygon) -> NavigationMeshPolygonId {
        self.polygon_uid += 1;

        self.polygons.insert(self.polygon_uid, polygon);

        self.polygon_uid
    }

    pub fn remove_polygon(
        &mut self,
        polygon_id: &NavigationMeshPolygonId,
    ) -> Option<ConvexPolygon> {
        self.polygons.remove(polygon_id)
    }

    pub fn path(&self, origin: Vector2<f32>, target: Vector2<f32>) -> Option<NavigationMeshPath> {
        None
    }

    fn get_polygon_with_point(&self, point: &Vector2<f32>) -> Option<NavigationMeshPolygonId> {
        None
    }
}

pub struct NavigationMeshPath {}
