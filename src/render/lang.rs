use cgmath::prelude::*;
use cgmath::{vec2, vec3, Vector2, Vector3, Vector4};
use fnv::FnvHashMap;
use ref_thread_local::{ref_thread_local, RefThreadLocal};
use std::cell::RefCell;
use std::collections::HashMap;
use std::iter::Sum;
use std::ops::Add;
use std::rc::Rc;

use super::{GadgetRenderInfo, Triangles, Vertex};
use crate::gadget::{Gadget, GadgetDef, State, PP};
use crate::math::{Mat2, Vec2, Vec3, Vector2Ex, TAU_F64};
use crate::shape::{Circle, Path, Shape};
use crate::static_map::StaticMap;

fn bez3(points: &[Vec2; 4], t: f64) -> Vec2 {
    let t1 = 1.0 - t;
    points[0] * t1 * t1 * t1
        + points[1] * 3.0 * t * t1 * t1
        + points[2] * 3.0 * t * t * t1
        + points[3] * t * t * t
}

/// X axis: clockwise tangent
/// Y axis: direction
fn bez3_dir(points: &[Vec2; 4], t: f64) -> Mat2 {
    let t1 = 1.0 - t;
    let dir = (points[0] * -3.0 * t1 * t1
        + points[1] * 3.0 * (t1 * t1 - 2.0 * t * t1)
        + points[2] * 3.0 * (2.0 * t * t1 - t * t)
        + points[3] * 3.0 * t * t)
        .normalize();

    Mat2::from_cols(dir.right_cw(), dir)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Term {
    pub ports: PP,
    pub t: f64,
    /// Factor for point on path between two ports
    pub point_factor: f64,
    /// Factor for normalized direction on path between two ports,
    /// with the x factor being for the clockwise tangent
    ///  and the y factor being for the direction
    pub dir_factors: Vec2,
}

impl Term {
    /// Computes the vector.
    fn vector(&self, port_positions: &[Vec2]) -> Vec3 {
        let positions: [Vec2; 2] = [
            port_positions[self.ports.0.id()],
            port_positions[self.ports.1.id()],
        ];
        let mut bezier = [vec2(0.0, 0.0), vec2(0.0, 0.0)];

        let offset = 0.25;

        for (pos, bez) in positions.iter().zip(bezier.iter_mut()) {
            *bez = pos
                + if pos.x.floor() == pos.x {
                    // on vertical edge
                    if pos.x == 0.0 {
                        // on left edge
                        vec2(offset, 0.0)
                    } else {
                        // on right edge
                        vec2(-offset, 0.0)
                    }
                } else {
                    // on horizontal edge
                    if pos.y == 0.0 {
                        // on bottom edge
                        vec2(0.0, offset)
                    } else {
                        // on top edge
                        vec2(0.0, -offset)
                    }
                }
        }

        // Same-port traversal; make it look like a loop
        if bezier[0] == bezier[1] {
            let dv = (bezier[0] - positions[0]).right_ccw();

            bezier[0] += dv;
            bezier[1] -= dv;
        }

        let points = [positions[0], bezier[0], bezier[1], positions[1]];
        (bez3(&points, self.t) * self.point_factor + bez3_dir(&points, self.t) * self.dir_factors)
            .extend(0.0)
    }
}

/// Position is some terms added together
// #[derive(Clone, Debug, PartialEq)]
// pub struct PositionDef {
//     abs: Vec3,
//     terms: Vec<Term>,
// }

// impl PositionDef {
//     fn new_abs(abs: Vec3) -> Self {
//         Self {
//             abs,
//             terms: vec![],
//         }
//     }

//     fn new_term(term: Term) -> Self {
//         Self {
//             abs: vec3(0., 0., 0.),
//             terms: vec![term],
//         }
//     }
// }

// impl Add<PositionDef> for PositionDef {
//     type Output = PositionDef;

//     fn add(mut self, mut rhs: PositionDef) -> Self::Output {
//         self.terms.append(&mut rhs.terms);

//         Self {
//             abs: self.abs + rhs.abs,
//             terms: self.terms,
//         }
//     }
// }

// impl Sum for PositionDef {
//     fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
//         iter.fold(PositionDef {
//             abs: vec3(0., 0., 0.),
//             terms: vec![],
//         }, |acc, elem| acc + elem)
//     }
// }

// #[derive(Clone, Debug, PartialEq)]
// pub struct VertexDef {
//     position: PositionDef,
//     color: Vector4<f32>,
// }

// impl VertexDef {
//     /// Computes the vertex.
//     fn vertex(&self, port_positions: &[Vec2]) -> Vertex {
//         Vertex::new(
//             (self.position.abs
//                 + self
//                     .position.terms
//                     .iter()
//                     .map(|t| t.vector(port_positions))
//                     .sum::<Vec3>())
//             .cast::<f32>()
//             .unwrap(),
//             vec3(0.0, 0.0, 0.0),
//             self.color,
//             [],
//         )
//     }
// }

// impl VertexDef {
//     fn new(position: PositionDef, color: Vector4<f32>) -> Self {
//         Self { position, color }
//     }
// }

// /// The rendering info for a gadget state
// #[derive(Clone, Debug, PartialEq)]
// pub struct StateModelDef {
//     vertices: Vec<VertexDef>,
//     indexes: Vec<u32>,
// }

// impl StateModelDef {
//     fn triangles(&self, gadget: &Gadget) -> Triangles {
//         let port_positions = gadget.port_positions();
//         Triangles::new(
//             self.vertices
//                 .iter()
//                 .map(|v| v.vertex(&port_positions))
//                 .collect(),
//             self.indexes.clone(),
//         )
//     }
// }

// /// The rendering info for a gadget.
// /// Used to specify where the triangles should go
// #[derive(Clone, Debug, PartialEq)]
// pub struct ModelDef {
//     states: Vec<StateModelDef>,
// }

// impl ModelDef {
//     pub fn triangles(&self, gadget: &Gadget) -> Triangles {
//         self.states[gadget.state().id()].triangles(gadget)
//     }
// }

/// A position in the gadget renderer language
#[derive(Clone, Debug, PartialEq)]
pub enum GrlPosition {
    Absolute(Vec3),
    Term(Term),
    Add(Vec<GrlPosition>),
}

impl GrlPosition {
    fn position(&self, port_positions: &[Vec2]) -> Vec3 {
        match self {
            GrlPosition::Absolute(pos) => *pos,
            GrlPosition::Term(term) => term.vector(port_positions),
            GrlPosition::Add(vec) => vec.into_iter().map(|p| p.position(port_positions)).sum(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GrlPath {
    Line { points: (GrlPosition, GrlPosition) },
    Circle { position: GrlPosition, radius: f64 },
    PortPath { ports: PP, ts: (f64, f64), z: f64 },
}

impl GrlPath {
    fn path(&self, thickness: f64, port_positions: &[Vec2]) -> Path {
        match self {
            GrlPath::Line { points } => {
                let p0 = points.0.position(port_positions);
                let p1 = points.1.position(port_positions);

                Path::new(vec![p0.truncate(), p1.truncate()], p0.z, thickness, false)
            }

            GrlPath::Circle { position, radius } => {
                let center = position.position(port_positions);

                Path::new(
                    Circle::new(center.x, center.y, center.z, *radius)
                        .positions_f64()
                        .into_iter()
                        .map(|xyz| xyz.truncate())
                        .collect(),
                    center.z,
                    thickness,
                    true,
                )
            }

            GrlPath::PortPath { ports, ts, z } => {
                let resolution = ((ts.1 - ts.0).abs() * 32.0).ceil() as usize;

                Path::new(
                    (0..resolution)
                        .map(|i| {
                            Term {
                                ports: *ports,
                                t: ts.0 + (ts.1 - ts.0) * i as f64 / resolution as f64,
                                point_factor: 1.0,
                                dir_factors: vec2(0.0, 0.0),
                            }
                            .vector(port_positions)
                            .truncate()
                        })
                        .collect(),
                    *z,
                    thickness,
                    false,
                )
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GrlLineStyle {
    Solid,
    Dotted { on_space: f64, off_space: f64 },
}

/// A shape in the gadget renderer language
#[derive(Clone, Debug, PartialEq)]
pub enum GrlShape {
    Circle {
        position: GrlPosition,
        radius: f64,
        color: Vector4<f32>,
    },
    Rectangle {
        position: GrlPosition,
        up: GrlPosition,
        width: f64,
        height: f64,
        color: Vector4<f32>,
    },
    Path {
        path: GrlPath,
        line_style: GrlLineStyle,
        thickness: f64,
        end_arrow_wh: Option<(f64, f64)>,
        color: Vector4<f32>,
    },
    Triangles {
        vertices: Vec<(GrlPosition, Vector4<f32>)>,
        indexes: Vec<u32>,
    },
}

// Skipping expressions when
#[rustfmt::skip]
fn rectangle_points(center: Vec3, up: Vec3, width: f64, height: f64) -> Vec<Vec3> {
    let right = up.truncate().right_cw().extend(0.0);

    vec![
        center + right * -width / 2.0 + up * -height / 2.0,
        center + right *  width / 2.0 + up * -height / 2.0,
        center + right *  width / 2.0 + up *  height / 2.0,
        center + right * -width / 2.0 + up *  height / 2.0,
    ]
}

impl GrlShape {
    fn triangles(&self, port_positions: &[Vec2]) -> Triangles {
        match self {
            GrlShape::Circle {
                position,
                radius,
                color,
            } => {
                let position = position.position(port_positions);
                Circle::new(position.x, position.y, position.z, *radius).triangles(*color)
            }

            GrlShape::Rectangle {
                position,
                up,
                width,
                height,
                color,
            } => (
                rectangle_points(
                    position.position(port_positions),
                    up.position(port_positions),
                    *width,
                    *height,
                ),
                vec![0, 1, 2, 2, 3, 0u32],
            )
                .triangles(*color),

            GrlShape::Path {
                path,
                line_style,
                thickness,
                end_arrow_wh,
                color,
            } => {
                let mut path = path.path(*thickness, port_positions);
                let z = path.z();
                let mut extra_tris = Triangles::default();

                if let Some((w, h)) = end_arrow_wh {
                    path = path.iter().subpath(path.len() - *h);

                    let dir = path.end_direction();
                    extra_tris.append(Triangles::new(
                        vec![
                            Vertex::new(
                                (path.end_position() + dir.right_cw() * *w / 2.0)
                                    .extend(z)
                                    .cast::<f32>()
                                    .unwrap(),
                                vec3(0., 0., 0.),
                                *color,
                                [],
                            ),
                            Vertex::new(
                                (path.end_position() + dir * *h)
                                    .extend(z)
                                    .cast::<f32>()
                                    .unwrap(),
                                vec3(0., 0., 0.),
                                *color,
                                [],
                            ),
                            Vertex::new(
                                (path.end_position() + dir.right_ccw() * *w / 2.0)
                                    .extend(z)
                                    .cast::<f32>()
                                    .unwrap(),
                                vec3(0., 0., 0.),
                                *color,
                                [],
                            ),
                        ],
                        vec![0, 1, 2],
                    ));
                }

                match line_style {
                    GrlLineStyle::Solid => extra_tris.append(path.triangles(*color)),

                    GrlLineStyle::Dotted {
                        on_space,
                        off_space,
                    } => {
                        let mut iter = path.iter();
                        while !iter.finished() {
                            extra_tris.append(iter.subpath(*on_space).triangles(*color));
                            iter.advance(*off_space);
                        }
                    }
                }

                extra_tris
            }

            GrlShape::Triangles { vertices, indexes } => Triangles::new(
                vertices
                    .iter()
                    .map(|v| {
                        Vertex::new(
                            v.0.position(port_positions).cast::<f32>().unwrap(),
                            vec3(0., 0., 0.),
                            v.1,
                            [],
                        )
                    })
                    .collect(),
                indexes.clone(),
            ),
        }
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct GrlState {
    pub shapes: Vec<GrlShape>,
}

impl GrlState {
    fn triangles(&self, gadget: &Gadget) -> Triangles {
        let mut triangles = Triangles::default();
        let port_positions = gadget.port_positions();

        for new_triangles in self.shapes.iter().map(|s| s.triangles(&port_positions)) {
            triangles.append(new_triangles);
        }

        triangles
    }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Grl {
    pub states: Vec<GrlState>,
}

impl Grl {
    pub const Z: f64 = GadgetRenderInfo::PATH_Z;
    pub const DEFAULT_LINE_THICKNESS: f64 = 0.04;
    pub const DEFAULT_ARROW_WIDTH: f64 = 0.16;
    pub const DEFAULT_ARROW_HEIGHT: f64 = 0.16;
    pub const DEFAULT_DOTTED_ON_SPACE: f64 = 0.04;
    pub const DEFAULT_DOTTED_OFF_SPACE: f64 = 0.08;

    pub fn triangles(&self, gadget: &Gadget) -> Triangles {
        self.states[gadget.state().id()].triangles(gadget)
    }
}

/// Macro for creating a gadget render using the gadget renderer language
#[macro_export]
macro_rules! grl {
    ( $( { $( $shapes:tt ),* $(,)? } )* ) => {
        $crate::render::lang::Grl { states: vec![
            $($crate::render::lang::GrlState { shapes: vec![
                $(grl!(shape $shapes)),*
            ]}),*
        ] }
    };

    // Unnecessary parentheses
    ( $name:ident ($($args:tt)*) ) => {
        grl!($name $($args)*)
    };

    // Position
    ( position $first:tt $(+ $rest:tt)+) => {
        $crate::render::lang::GrlPosition::Add(vec![grl!(position $first), $(grl!(position $rest))+])
    };

    ( position $x:expr, $y:expr, $z:expr ) => {
        $crate::render::lang::GrlPosition::Absolute(cgmath::vec3($x, $y, $z))
    };

    ( position $p0:expr => $p1:expr, $t:expr; $fac:expr, $rfac:expr, $ufac:expr ) => {
        $crate::render::lang::GrlPosition::Term(
            $crate::render::lang::Term {
                ports: ($crate::gadget::Port($p0), $crate::gadget::Port($p1)),
                t: $t,
                point_factor: $fac,
                dir_factors: cgmath::vec2($rfac, $ufac),
            }
        )
    };

    ( position $p0:expr => $p1:expr, $t:expr ) => {
        grl!(position $p0 => $p1, $t; 1.0, 0.0, 0.0)
    };

    ( position $p0:expr => $p1:expr, $t:expr; dir $rfac:expr, $ufac:expr ) => {
        grl!(position $p0 => $p1, $t; 0.0, $rfac, $ufac)
    };

    ( position z $z:expr ) => {
        grl!(position 0.0, 0.0, $z)
    };

    // Path
    ( path line $pos0:tt => $pos1:tt ) => {
        $crate::render::lang::GrlPath::Line {
            points: (grl!(position $pos0), grl!(position $pos1))
        }
    };

    ( path port_path $p0:expr => $p1:expr, $t0:expr => $t1:expr, $z:expr ) => {
        $crate::render::lang::GrlPath::PortPath {
            ports: ($crate::gadget::Port($p0), $crate::gadget::Port($p1)),
            ts: ($t0, $t1),
            z: $z,
        }
    };

    ( path circle $pos:tt, $rad:expr ) => {
        $crate::render::lang::GrlPath::Circle {
            position: grl!(position $pos),
            radius: $rad,
        }
    };

    // Line style
    ( line_style solid ) => {
        $crate::render::lang::GrlLineStyle::Solid
    };

    ( line_style dotted $on:expr, $off:expr ) => {
        $crate::render::lang::GrlLineStyle::Dotted {
            on_space: $on,
            off_space: $off,
        }
    };

    ( line_style dotted ) => {
        grl!(line_style dotted $crate::render::lang::Grl::DEFAULT_DOTTED_ON_SPACE,
            $crate::render::lang::Grl::DEFAULT_DOTTED_OFF_SPACE)
    };

    // Shapes
    ( shape circle $pos:tt, $rad:expr, ($r:expr, $g:expr, $b:expr, $a:expr) ) => {
        $crate::render::lang::GrlShape::Circle {
            position: grl!(position $pos),
            radius: $rad,
            color: cgmath::vec4($r, $g, $b, $a)
        }
    };

    ( shape circle $pos:tt, $rad:expr ) => {
        grl!(shape circle $pos, $rad, (0.0, 0.0, 0.0, 1.0))
    };

    ( shape circle $pos:tt, $rad:expr, fade ) => {
        grl!(shape circle $pos, $rad, (0.6, 0.65, 0.7, 1.0))
    };

    ( shape rect $pos:tt, $up:tt, $w:expr, $h:expr, ($r:expr, $g:expr, $b:expr, $a:expr) ) => {
        $crate::render::lang::GrlShape::Rectangle {
            position: grl!(position $pos),
            up: grl!(position $up),
            width: $w,
            height: $h,
            color: cgmath::vec4($r, $g, $b, $a)
        }
    };

    ( shape rect $pos:tt, $up:tt, $w:expr, $h:expr ) => {
        grl!(shape rect $pos, $up, $w, $h, (0.0, 0.0, 0.0, 1.0))
    };

    ( shape rect $pos:tt, $up:tt, $w:expr, $h:expr, fade ) => {
        grl!(shape rect $pos, $up, $w, $h, (0.6, 0.65, 0.7, 1.0))
    };

    ( shape path_internal $path:tt, $style:tt, $thick:expr, $end:expr, ($r:expr, $g:expr, $b:expr, $a:expr) ) => {
        $crate::render::lang::GrlShape::Path {
            path: grl!(path $path),
            line_style: grl!(line_style $style),
            thickness: $thick,
            end_arrow_wh: $end,
            color: cgmath::vec4($r, $g, $b, $a)
        }
    };

    ( shape path $path:tt, $style:tt, ($r:expr, $g:expr, $b:expr, $a:expr) ) => {
        grl!(shape path_internal $path, $style, $crate::render::lang::Grl::DEFAULT_LINE_THICKNESS, None, ($r, $g, $b, $a))
    };

    ( shape path $path:tt, $style:tt ) => {
        grl!(shape path $path, $style, (0.0, 0.0, 0.0, 1.0))
    };

    ( shape path $path:tt, $style:tt, fade ) => {
        grl!(shape path $path, $style, (0.6, 0.65, 0.7, 1.0))
    };

    ( shape path $path:tt, $style:tt, |>, ($r:expr, $g:expr, $b:expr, $a:expr) ) => {
        grl!(shape path_internal $path, $style, $crate::render::lang::Grl::DEFAULT_LINE_THICKNESS, Some(
            ($crate::render::lang::Grl::DEFAULT_ARROW_WIDTH, $crate::render::lang::Grl::DEFAULT_ARROW_HEIGHT)
        ), ($r, $g, $b, $a))
    };

    ( shape path $path:tt, $style:tt, |> ) => {
        grl!(shape path $path, $style, |>, (0.0, 0.0, 0.0, 1.0))
    };

    ( shape path $path:tt, $style:tt, |>, fade ) => {
        grl!(shape path $path, $style, |>, (0.6, 0.65, 0.7, 1.0))
    };

    ( shape path $path:tt, $style:tt, $thick:expr, ($r:expr, $g:expr, $b:expr, $a:expr) ) => {
        grl!(shape path_internal $path, $style, $thick, None, ($r, $g, $b, $a))
    };

    ( shape path $path:tt, $style:tt, $thick:expr ) => {
        grl!(shape path $path, $style, $thick, (0.0, 0.0, 0.0, 1.0))
    };

    ( shape path $path:tt, $style:tt, $thick:expr, fade ) => {
        grl!(shape path $path, $style, $thick, (0.6, 0.65, 0.7, 1.0))
    };

    ( shape path $path:tt, $style:tt, $thick:expr, |> $w:expr, $h:expr, ($r:expr, $g:expr, $b:expr, $a:expr) ) => {
        grl!(shape path_internal $path, $style, $thick, Some(($w, $h)), ($r, $g, $b, $a))
    };

    ( shape path $path:tt, $style:tt, $thick:expr, |> $w:expr, $h:expr ) => {
        grl!(shape path $path, $style, $thick, |> $w, $h, (0.0, 0.0, 0.0, 1.0))
    };

    ( shape path $path:tt, $style:tt, $thick:expr, |> $w:expr, $h:expr, fade ) => {
        grl!(shape path $path, $style, $thick, |> $w, $h, (0.6, 0.65, 0.7, 1.0))
    };

    ( shape tris {$($pos:tt, ($r:expr, $g:expr, $b:expr, $a:expr));* $(;)?}, [$($idx:expr),* $(,)?] ) => {
        $crate::render::lang::GrlShape::Triangles {
            vertices: vec![
                $((grl!(position $pos), cgmath::vec4($r, $g, $b, $a))),*
            ],
            indexes: vec![$($idx),*]
        }
    };
}

struct GrlCache(RefCell<HashMap<String, Rc<Grl>>>);

impl GrlCache {
    fn new() -> Self {
        GrlCache(RefCell::new(HashMap::new()))
    }

    /// Purposefully get the default renderer, even if there's a custom one
    fn get_default(def: &GadgetDef) -> Grl {
        let mut grl = Grl::default();

        for state in (0..def.num_states()).map(State) {
            let mut shapes = GrlState::default();

            let port_traversals = def.port_traversals_in_state(state);
            for (p0, p1) in port_traversals.iter() {
                let directed = !port_traversals.contains(&(*p1, *p0));

                if directed {
                    shapes.shapes.push(
                        grl!(shape path (port_path p0.id() => p1.id(), 0. => 1., Grl::Z), solid, |>)
                    );
                } else {
                    shapes.shapes.push(
                        grl!(shape path (port_path p0.id() => p1.id(), 0. => 1., Grl::Z), solid),
                    );
                }
            }

            grl.states.push(shapes);
        }

        grl
    }

    fn get(&self, def: &GadgetDef) -> Rc<Grl> {
        let hash_string = def.hash_string();

        if let Some(grl) = self.0.borrow().get(&hash_string) {
            return Rc::clone(grl);
        } else if let Some(grl) = GRLS.borrow().get(&hash_string) {
            return Rc::clone(grl);
        }

        let grl = Rc::new(Self::get_default(def));
        self.0
            .borrow_mut()
            .insert(def.hash_string(), Rc::clone(&grl));
        grl
    }
}

pub fn get_grl(def: &GadgetDef) -> Rc<Grl> {
    GRL_CACHE.borrow().get(def)
}

ref_thread_local! {
    static managed GRL_CACHE: GrlCache = GrlCache::new();
}

type GrlMap = FnvHashMap<String, Rc<Grl>>;

ref_thread_local!(
    pub static managed GRLS: StaticMap<String, Rc<Grl>, fn(Vec<(Rc<GadgetDef>, Grl, bool)>) -> GrlMap, Vec<(Rc<GadgetDef>, Grl, bool)>> = StaticMap::new(
        grl_map
    );
);

/// If the boolean is true, then replace; otherwise combine with the default
fn grl_map(map: Vec<(Rc<GadgetDef>, Grl, bool)>) -> GrlMap {
    map.into_iter()
        .map(|(def, mut grl, replace)| {
            (def.hash_string(), {
                if !replace {
                    let default = GrlCache::get_default(&def);
                    for (state, mut default_state) in
                        grl.states.iter_mut().zip(default.states.into_iter())
                    {
                        state.shapes.append(&mut default_state.shapes)
                    }
                }

                Rc::new(grl)
            })
        })
        .collect()
}

#[cfg(test)]
#[allow(unused_variables)]
mod test {
    // Have some compile-time tests, and some runtime ones too

    use super::*;
    use crate::gadget::Port;

    #[test]
    fn test_grl_empty() {
        let test = grl!();
    }

    #[test]
    fn test_grl_position_absolute() {
        let test = grl!(position(0.0, 1.0, 2.0));
        assert_eq!(test, GrlPosition::Absolute(vec3(0.0, 1.0, 2.0)));
    }

    #[test]
    fn test_grl_position_term() {
        let test = grl!(position (1 => 2, 0.25; 1.0, 0.5, 0.75));
        assert_eq!(
            test,
            GrlPosition::Term(Term {
                ports: (Port(1), Port(2)),
                t: 0.25,
                point_factor: 1.0,
                dir_factors: vec2(0.5, 0.75),
            })
        );
    }

    #[test]
    fn test_grl_position_add() {
        let test = grl!(position ((1.0, 2.0, 3.0) + (0 => 0, 0.0; 0.0, 0.0, 0.0)));
        assert_eq!(
            test,
            GrlPosition::Add(vec![
                grl!(position(1.0, 2.0, 3.0)),
                grl!(position (0 => 0, 0.0; 0.0, 0.0, 0.0))
            ])
        );
    }

    #[test]
    fn test_grl_path_line() {
        let test = grl!(path (line (0.0, 1.0, 2.0) => ((3.0, 4.0, 5.0) + (1.0, 2.0, 1.0))));
        assert_eq!(
            test,
            GrlPath::Line {
                points: (
                    grl!(position(0.0, 1.0, 2.0)),
                    grl!(position((3.0, 4.0, 5.0) + (1.0, 2.0, 1.0)))
                )
            }
        )
    }

    #[test]
    fn test_grl_path_port_path() {
        let test = grl!(path (port_path 2 => 1, 0.25 => 0.75, 0.));
        assert_eq!(
            test,
            GrlPath::PortPath {
                ports: (Port(2), Port(1)),
                ts: (0.25, 0.75),
                z: 0.
            }
        )
    }

    // TODO: More run-time tests

    #[test]
    fn test_grl_line_style_solid() {
        let test = grl!(line_style(solid));
    }

    #[test]
    fn test_grl_line_style_dotted() {
        let test = grl!(line_style (dotted 1.0, 2.0));
    }

    #[test]
    fn test_shape_circle() {
        let test = grl!(shape circle (1.0, 2.0, 3.0), 0.5, (0.0, 0.0, 0.0, 1.0));
        let test = grl!(shape circle (1.0, 2.0, 3.0), 0.5);
    }

    #[test]
    fn test_shape_rectangle() {
        let test = grl!(shape rect (0 => 1, 0.5; 0.0, 1.0, 0.0), (1.0, 2.0, 3.0), 1.0, 2.0, (0.1, 0.2, 0.3, 1.0));
        let test = grl!(shape rect (0 => 1, 0.5; 0.0, 1.0, 0.0), (1.0, 2.0, 3.0), 1.0, 2.0);
    }

    #[test]
    fn test_shape_path() {
        let test =
            grl!(shape path (port_path 0 => 1, 0.1 => 0.9, 1.0), solid, 0.1, (0.5, 0.5, 0.5, 1.0));
        let test = grl!(shape path (port_path 0 => 1, 0.1 => 0.9, 1.0), solid, 0.1);
        let test = grl!(shape path (port_path 0 => 1, 0.1 => 0.9, 1.0), solid, 0.1, |> 1.0, 2.0, (0.5, 0.5, 0.5, 1.0));
        let test = grl!(shape path (port_path 0 => 1, 0.1 => 0.9, 1.0), solid, 0.1, |> 1.0, 2.0);
    }

    #[test]
    fn test_shape_triangles() {
        let test = grl!(shape tris {
            ((1., 2., 3.) + (5., 6., 7.)), (0.1, 0.2, 0.3, 1.0);
            ((0 => 1, 0.5; 1., 0., 1.)), (1.0, 1.0, 1.0, 1.0);
            ((2., 4., 3.) + (1 => 2, 0.1; 1., 0., 0.)), (0.7, 0.6, 0.9, 1.0);
        }, [0, 1, 2]);
    }

    #[test]
    fn test_complex() {
        let test = grl!(
            {}
            {
                (circle (0., 1., 2.), 0.5),
                (rect (0 => 1, 0.5; 0.1, 0.9, 0.8), (1., 0., 0.), 1.0, 1.0, (0., 1., 0., 1.)),
            }
        );
    }
}
