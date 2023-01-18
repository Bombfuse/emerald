use crate::*;

#[derive(Debug, Copy, Clone)]
pub struct ColorTri {
    pub color: Color,
    pub offset: Vector2<f32>,
    pub visible: bool,
    pub centered: bool,
    pub z_index: f32,
    pub points: [Vector2<f32>; 3],
}
impl ColorTri {
    pub fn new(color: Color, points: [Vector2<f32>; 3]) -> Self {
        ColorTri {
            color,
            points,
            ..Default::default()
        }
    }

    pub fn get_bounding_rect(&self) -> Rectangle {
        get_bounding_box_of_triangle(&self.points)
    }

    pub fn get_max_x(&self) -> f32 {
        get_max_x(&self.points)
    }

    pub fn get_min_x(&self) -> f32 {
        get_min_x(&self.points)
    }

    pub fn get_max_y(&self) -> f32 {
        get_max_y(&self.points)
    }

    pub fn get_min_y(&self) -> f32 {
        get_min_y(&self.points)
    }
}
impl Default for ColorTri {
    fn default() -> ColorTri {
        ColorTri {
            color: WHITE,
            offset: Vector2::new(0.0, 0.0),
            visible: true,
            centered: true,
            z_index: 0.0,
            // default of 1 pixel wide triangle
            points: [
                Vector2::new(-0.5, -0.5),
                Vector2::new(0.0, 0.5),
                Vector2::new(0.5, -0.5),
            ],
        }
    }
}

pub fn get_bounding_box_of_triangle(points: &[Vector2<f32>; 3]) -> Rectangle {
    let max_x = get_max_x(points);
    let max_y = get_max_y(points);
    let min_x = get_min_x(points);
    let min_y = get_min_y(points);

    Rectangle::new(min_x, min_y, max_x - min_x, max_y - min_y)
}

fn get_max_x(points: &[Vector2<f32>; 3]) -> f32 {
    let (x1, x2, x3) = (points[0].x, points[1].x, points[2].x);
    x1.max(x2).max(x3)
}

fn get_min_x(points: &[Vector2<f32>; 3]) -> f32 {
    let (x1, x2, x3) = (points[0].x, points[1].x, points[2].x);
    x1.min(x2).min(x3)
}

fn get_max_y(points: &[Vector2<f32>; 3]) -> f32 {
    let (y1, y2, y3) = (points[0].y, points[1].y, points[2].y);
    y1.max(y2).max(y3)
}

fn get_min_y(points: &[Vector2<f32>; 3]) -> f32 {
    let (y1, y2, y3) = (points[0].y, points[1].y, points[2].y);
    y1.min(y2).min(y3)
}
