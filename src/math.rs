use cgmath::prelude::*;
use cgmath::BaseNum;
use cgmath::Vector2;
use std::ops::Neg;

pub const TAUf32: f32 = std::f32::consts::PI * 2.0;
pub const TAUf64: f64 = std::f64::consts::PI * 2.0;

pub trait Vector2Ex<S: BaseNum + Neg> {
    /// Rotates the vector 90 degrees counterclockwise
    fn right_ccw(self) -> Self;

    /// Dot product; not restricted to floats
    fn dot_ex(self, other: Self) -> S;
}

impl<S> Vector2Ex<S> for Vector2<S>
where
    S: BaseNum + Neg<Output = S>,
{
    fn right_ccw(self) -> Self {
        Vector2::new(-self.y, self.x)
    }

    fn dot_ex(self, other: Self) -> S {
        self.x * other.x + self.y * other.y
    }
}

pub type Vec2i = Vector2<i32>;
