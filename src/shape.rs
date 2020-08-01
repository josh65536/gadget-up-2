use cgmath::prelude::*;
use cgmath::{vec3, Vector3, Vector4};
use itertools::izip;

use crate::math::TAU_F64;
use crate::math::{Vec2, Vector2Ex};
use crate::render::{Triangles, Vertex};

pub trait Shape {
    fn num_vertices(&self) -> usize;

    fn positions_f64(&self) -> Vec<Vector3<f64>>;

    fn positions(&self) -> Vec<Vector3<f32>> {
        self.positions_f64()
            .into_iter()
            .map(|v| v.cast::<f32>().unwrap())
            .collect()
    }

    fn indexes(&self) -> Vec<u32>;

    /// Gets the triangles that this shape represents.
    /// `color` is in RGBA format
    fn triangles(&self, color: Vector4<f32>) -> Triangles {
        Triangles::new(
            self.positions()
                .into_iter()
                .map(|p| Vertex::new(p, vec3(0.0, 0.0, 0.0), color, []))
                .collect(),
            self.indexes(),
        )
    }
}

// For convenience of providing a color
impl Shape for (Vec<Vector3<f64>>, Vec<u32>) {
    fn num_vertices(&self) -> usize {
        self.0.len()
    }

    fn positions_f64(&self) -> Vec<Vector3<f64>> {
        self.0.clone()
    }

    fn indexes(&self) -> Vec<u32> {
        self.1.clone()
    }
}

#[derive(Clone, Debug)]
pub struct Rectangle {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
    z: f64,
}

impl Rectangle {
    pub fn new(min_x: f64, max_x: f64, min_y: f64, max_y: f64, z: f64) -> Self {
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

    #[rustfmt::skip]
    fn positions_f64(&self) -> Vec<Vector3<f64>> {
        vec![
            vec3(self.min_x, self.min_y, self.z),
            vec3(self.max_x, self.min_y, self.z),
            vec3(self.max_x, self.max_y, self.z),
            vec3(self.min_x, self.max_y, self.z),
        ]
    }

    fn indexes(&self) -> Vec<u32> {
        vec![0, 1, 2, 2, 3, 0]
    }
}

#[derive(Clone, Debug)]
pub struct Circle {
    x: f64,
    y: f64,
    z: f64,
    radius: f64,
}

impl Circle {
    const RESOLUTION: usize = 32;

    /// Circle is placed parallel to the xy plane
    pub fn new(x: f64, y: f64, z: f64, radius: f64) -> Self {
        Self { x, y, z, radius }
    }
}

impl Shape for Circle {
    fn num_vertices(&self) -> usize {
        Self::RESOLUTION + 1
    }

    fn positions_f64(&self) -> Vec<Vector3<f64>> {
        (0..Self::RESOLUTION)
            .map(|i| {
                vec3(
                    (TAU_F64 * i as f64 / Self::RESOLUTION as f64).cos() * self.radius + self.x,
                    (TAU_F64 * i as f64 / Self::RESOLUTION as f64).sin() * self.radius + self.y,
                    self.z,
                )
            })
            .collect()
    }

    fn indexes(&self) -> Vec<u32> {
        (1..(Self::RESOLUTION - 1))
            .flat_map(|i| vec![0, i as u32, i as u32 + 1])
            .collect()
    }
}

#[derive(Clone, Debug)]
/// A series of line segments
pub struct Path {
    xys: Vec<Vec2>,
    z: f64,
    thickness: f64,
    closed: bool,
}

#[allow(dead_code)]
impl Path {
    pub fn new(xys: Vec<Vec2>, z: f64, thickness: f64, closed: bool) -> Self {
        Self {
            xys,
            z,
            thickness,
            closed,
        }
    }

    pub fn z(&self) -> f64 {
        self.z
    }

    /// Splits a cubic bezier curve into line segments
    /// The xys are [vertex 0, end control 0, start control 1, vertex 1]
    pub fn from_bezier3(xys: [Vec2; 4], z: f64, thickness: f64) -> Self {
        const RESOLUTION: usize = 32;

        let xys = (0..=RESOLUTION)
            .map(|i| {
                let t = i as f64 / RESOLUTION as f64;

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

    pub fn start_position(&self) -> Vec2 {
        self.xys[0]
    }

    pub fn end_position(&self) -> Vec2 {
        *self.xys.last().unwrap()
    }

    pub fn start_direction(&self) -> Vec2 {
        (self.xys[1] - self.xys[0]).normalize()
    }

    pub fn end_direction(&self) -> Vec2 {
        (self.xys[self.xys.len() - 1] - self.xys[self.xys.len() - 2]).normalize()
    }

    pub fn iter(&self) -> PathIter {
        if self.closed {
            unimplemented!("Path iter not supported for closed paths yet");
        }

        PathIter::new(self)
    }

    pub fn len(&self) -> f64 {
        let mut len = self
            .xys
            .iter()
            .zip(self.xys.iter().skip(1))
            .map(|(p0, p1)| p0.distance(*p1))
            .sum();

        if self.closed {
            len += self.xys.last().unwrap().distance(self.xys[0]);
        }

        len
    }
}

pub struct PathIter<'a> {
    path: &'a Path,
    point: usize,
    t: f64, // ranges from 0 to segment_len
    segment_len: f64,
}

impl<'a> PathIter<'a> {
    pub fn new(path: &'a Path) -> Self {
        let mut iter = PathIter {
            path,
            point: 0,
            t: 0.0,
            segment_len: 0.0,
        };

        iter.update_segment_len();
        iter
    }

    pub fn update_segment_len(&mut self) {
        self.segment_len = self.path.xys[self.point].distance(self.path.xys[self.point + 1]);
    }

    pub fn finished(&self) -> bool {
        self.point == self.path.xys.len() - 2 && self.t >= self.segment_len
    }

    pub fn curr_point(&self) -> Vec2 {
        let t = self.t / self.segment_len;
        self.path.xys[self.point] * (1.0 - t) + self.path.xys[self.point + 1] * t
    }

    pub fn subpath(&mut self, mut length: f64) -> Path {
        let mut xys = vec![self.curr_point()];

        self.t += length;

        while self.t >= self.segment_len && !self.finished() {
            self.t -= self.segment_len;
            self.point += 1;
            self.update_segment_len();
            xys.push(self.path.xys[self.point]);
        }

        self.t = self.t.min(self.segment_len);

        xys.push(self.curr_point());

        Path::new(xys, self.path.z, self.path.thickness, false)
    }

    /// Like subpath, but intentionally drops the path
    pub fn advance(&mut self, length: f64) {
        self.subpath(length);
    }
}

impl Shape for Path {
    fn num_vertices(&self) -> usize {
        self.xys.len() * 2
    }

    fn positions_f64(&self) -> Vec<Vector3<f64>> {
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
                    vec3(v1.x + dv1.x, v1.y + dv1.y, self.z),
                    vec3(v1.x - dv1.x, v1.y - dv1.y, self.z),
                ]);
            }
        }

        for (i, (v0, v1, v2)) in iter {
            if i == self.xys.len() - 1 && !self.closed {
                let dv0: Vec2 = v1 - v0;
                let dv0 = dv0.right_ccw().normalize_to(self.thickness / 2.0);

                vec.extend(&[
                    vec3(v1.x + dv0.x, v1.y + dv0.y, self.z),
                    vec3(v1.x - dv0.x, v1.y - dv0.y, self.z),
                ]);
            } else {
                let dv0: Vec2 = (v1 - v0).normalize();
                let dv1: Vec2 = (v2 - v1).normalize();

                let dv = (dv1.right_ccw() + dv0.right_ccw()).normalize_to(self.thickness / 2.0);
                vec.extend(&[
                    vec3(v1.x + dv.x, v1.y + dv.y, self.z),
                    vec3(v1.x - dv.x, v1.y - dv.y, self.z),
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
