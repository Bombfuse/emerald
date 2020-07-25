use nanoserde::DeJson;

#[derive(Clone, Copy, Debug, DeJson)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}
impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position { x, y }
    }

    pub fn zero() -> Self {
        Position::new(0.0, 0.0)
    }
}