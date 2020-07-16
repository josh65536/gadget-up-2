use cgmath::prelude::*;
use cgmath::BaseFloat;
use cgmath::Vector2;

pub const TAUf32: f32 = std::f32::consts::PI * 2.0;
pub const TAUf64: f64 = std::f64::consts::PI * 2.0;

pub trait Vector2Ex<S: BaseFloat> {
    /// Rotates the vector 90 degrees counterclockwise
    fn right_ccw(&self) -> Self;
}

impl<S: BaseFloat> Vector2Ex<S> for Vector2<S> {
    fn right_ccw(&self) -> Self {
        Vector2::new(-self.y, self.x)
    }
}
