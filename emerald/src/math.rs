use core::{
    fmt,
    ops::{Add, Div},
};

use fixed_sqrt::FixedSqrt;
use num_traits::PrimInt;

use fixed::{traits::ToFixed, types::I48F16};
use serde::{Deserialize, Serialize};

// define fixed point used
pub type FixedPoint = I48F16;

#[derive(Copy, PartialEq, Eq, Clone, Debug, Deserialize, Serialize)]
pub struct Vector2 {
    pub x: FixedPoint,
    pub y: FixedPoint,
}

impl Vector2 {
    pub fn new(x: FixedPoint, y: FixedPoint) -> Self {
        Self { x: x, y: y }
    }
    pub fn from_float<T: ToFixed>(x: T, y: T) -> Self {
        Self::new(FixedPoint::from_num(x), FixedPoint::from_num(y))
    }

    pub fn from_int<T: ToFixed>(x: T, y: T) -> Self {
        Self::new(FixedPoint::from_num(x), FixedPoint::from_num(y))
    }

    pub fn length(&self) -> FixedPoint {
        ((self.x * self.x) + (self.y * self.y)).sqrt()
    }

    pub fn normalized(&self) -> Self {
        Self::new(self.x / self.length(), self.y / self.length())
    }
}

impl fmt::Display for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({0}, {1})", self.x, self.y)
    }
}

// todo!("implement all math operations for vectors")
impl Div<FixedPoint> for Vector2 {
    type Output = Self;

    fn div(self, rhs: FixedPoint) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs)
    }
}

// todo!("implement all math operations for vectors")
impl Add<FixedPoint> for Vector2 {
    type Output = Self;

    fn add(self, rhs: FixedPoint) -> Self::Output {
        Self::new(self.x + rhs, self.y + rhs)
    }
}

// todo!("implement all math operations for vectors")
impl Add<Vector2> for Vector2 {
    type Output = Self;

    fn add(self, rhs: Vector2) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y)
    }
}
