use cgmath::prelude::*;
use itertools::izip;
use three_d::Vec2;

use crate::log;
use crate::math::TAUf32;
use crate::math::Vector2Ex;

pub trait Shape {
    fn num_vertices(&self) -> usize;

    fn positions(&self) -> Vec<f32>;

    fn indexes(&self) -> Vec<u32>;

    fn append_to(&self, positions: &mut Vec<f32>, indexes: &mut Vec<u32>) {
        let index = positions.len() as u32 / 3;

        positions.extend(self.positions().into_iter());
        indexes.extend(self.indexes().into_iter().map(|i| i + index));
    }
}

#[derive(Clone, Debug)]
pub struct Rectangle {
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
    z: f32,
}

impl Rectangle {
    pub fn new(min_x: f32, max_x: f32, min_y: f32, max_y: f32, z: f32) -> Self {
        Self {
            min_x,
            max_x,
            min_y,
            max_y,
            z,
        }
    }
}

impl Shape for Rectangle {
    fn num_vertices(&self) -> usize {
        4
    }

    fn positions(&self) -> Vec<f32> {
        vec![
            self.min_x, self.min_y, self.z, self.max_x, self.min_y, self.z, self.max_x, self.max_y,
            self.z, self.min_x, self.max_y, self.z,
        ]
    }

    fn indexes(&self) -> Vec<u32> {
        vec![0, 1, 2, 2, 3, 0]
    }
}

#[derive(Clone, Debug)]
pub struct Circle {
    x: f32,
    y: f32,
    z: f32,
    radius: f32,
}

impl Circle {
    const RESOLUTION: usize = 32;

    /// Circle is placed parallel to the xy plane
    pub fn new(x: f32, y: f32, z: f32, radius: f32) -> Self {
        Self { x, y, z, radius }
    }
}

impl Shape for Circle {
    fn num_vertices(&self) -> usize {
        Self::RESOLUTION + 1
    }

    fn positions(&self) -> Vec<f32> {
        (0..Self::RESOLUTION)
            .flat_map(|i| {
                vec![
                    (TAUf32 * i as f32 / Self::RESOLUTION as f32).cos() * self.radius + self.x,
                    (TAUf32 * i as f32 / Self::RESOLUTION as f32).sin() * self.radius + self.y,
                    self.z,
                ]
            })
            .chain(vec![self.x, self.y, self.z])
            .collect()
    }

    fn indexes(&self) -> Vec<u32> {
        (0..Self::RESOLUTION)
            .flat_map(|i| {
                vec![
                    Self::RESOLUTION as u32,
                    i as u32,
                    (if i + 1 == Self::RESOLUTION { 0 } else { i + 1 }) as u32,
                ]
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
/// A series of line segments
pub struct Path {
    xys: Vec<Vec2>,
    z: f32,
    thickness: f32,
    closed: bool,
}

impl Path {
    pub fn new(xys: Vec<Vec2>, z: f32, thickness: f32, closed: bool) -> Self {
        Self {
            xys,
            z,
            thickness,
            closed,
        }
    }

    /// Splits a cubic bezier curve into line segments
    /// The xys are [vertex 0, end control 0, start control 1, vertex 1]
    pub fn from_bezier3(xys: [Vec2; 4], z: f32, thickness: f32) -> Self {
        const RESOLUTION: usize = 32;

        let xys = (0..=RESOLUTION)
            .map(|i| {
                let t = i as f32 / RESOLUTION as f32;

                let tr = 1.0 - t;
                let t2 = t * t;
                let tr2 = tr * tr;
                tr2 * tr * xys[0]
                    + 3.0 * tr2 * t * xys[1]
                    + 3.0 * t2 * tr * xys[2]
                    + t2 * t * xys[3]
            })
            .collect();

        Self {
            xys,
            z,
            thickness,
            closed: false,
        }
    }

    pub fn start_direction(&self) -> Vec2 {
        (self.xys[1] - self.xys[0]).normalize()
    }

    pub fn end_direction(&self) -> Vec2 {
        (self.xys[self.xys.len() - 1] - self.xys[self.xys.len() - 2]).normalize()
    }
}

impl Shape for Path {
    fn num_vertices(&self) -> usize {
        self.xys.len() * 2
    }

    fn positions(&self) -> Vec<f32> {
        let mut vec = Vec::new();
        vec.reserve(self.num_vertices());

        let last = self.xys.last().copied();
        let first = self.xys.first().copied();

        // Iterate over triples of previous, current, and next positions
        let mut iter = izip!(
            last.iter().chain(self.xys.iter()),
            self.xys.iter(),
            self.xys.iter().skip(1).chain(first.iter())
        )
        .enumerate();

        if !self.closed {
            if let Some((_, (_, v1, v2))) = iter.next() {
                let dv1: Vec2 = v2 - v1;
                let dv1 = dv1.right_ccw().normalize_to(self.thickness / 2.0);

                vec.extend(&[
                    v1.x + dv1.x,
                    v1.y + dv1.y,
                    self.z,
                    v1.x - dv1.x,
                    v1.y - dv1.y,
                    self.z,
                ]);
            }
        }

        for (i, (v0, v1, v2)) in iter {
            if i == self.xys.len() - 1 && !self.closed {
                let dv0: Vec2 = v1 - v0;
                let dv0 = dv0.right_ccw().normalize_to(self.thickness / 2.0);

                vec.extend(&[
                    v1.x + dv0.x,
                    v1.y + dv0.y,
                    self.z,
                    v1.x - dv0.x,
                    v1.y - dv0.y,
                    self.z,
                ]);
            } else {
                let dv0: Vec2 = (v1 - v0).normalize();
                let dv1: Vec2 = (v2 - v1).normalize();

                let dv = (dv1.right_ccw() + dv0.right_ccw()).normalize_to(self.thickness / 2.0);
                vec.extend(&[
                    v1.x + dv.x,
                    v1.y + dv.y,
                    self.z,
                    v1.x - dv.x,
                    v1.y - dv.y,
                    self.z,
                ]);
            }
        }

        vec
    }

    fn indexes(&self) -> Vec<u32> {
        if self.closed {
            (0..self.xys.len() as u32)
                .flat_map(|i| {
                    let j = if i == self.xys.len() as u32 - 1 {
                        0
                    } else {
                        i + 1
                    };
                    vec![
                        2 * i + 1,
                        2 * j + 1,
                        2 * j + 0,
                        2 * j + 0,
                        2 * i + 0,
                        2 * i + 1,
                    ]
                })
                .collect()
        } else {
            (0..self.xys.len() as u32 - 1)
                .flat_map(|i| {
                    vec![
                        2 * i + 1,
                        2 * i + 3,
                        2 * i + 2,
                        2 * i + 2,
                        2 * i + 0,
                        2 * i + 1,
                    ]
                })
                .collect()
        }
    }
}
