use super::{Camera, ShaderType, TrianglesEx, TrianglesType, SHADERS, TRIANGLESES};
use super::{Model, ModelType, Triangles, Vertex, MODELS};
use super::lang::{self, Grl};
use crate::gadget::{Agent, Gadget, PP};
use crate::grid::{WH, XY};

use crate::math::{Mat4, Vec2, Vec2i, Vector2Ex};
use crate::shape::{Circle, Path, Shape};

use cgmath::{vec2, vec3, vec4};
use fnv::FnvHashMap;

use golem::UniformValue;
use golem::{Context, ShaderProgram};
use golem::{ElementBuffer, GeometryMode, VertexBuffer};

use ref_thread_local::RefThreadLocal;
use std::cell::{Ref, RefCell};
use std::rc::Rc;

pub struct GadgetRenderInfo {
    grl: Rc<Grl>,
    grl_set: bool,
    triangles: Triangles,
    //paths: FnvHashMap<PP, Path>,
    /// Cached model
    model: RefCell<Option<Model>>,
}

impl GadgetRenderInfo {
    pub const RECTANGLE_Z: f64 = -0.001;
    pub const OUTLINE_Z: f64 = -0.002;
    pub const PATH_Z: f64 = -0.003;
    pub const PORT_Z: f64 = -0.004;

    pub fn triangles(&self) -> &Triangles {
        &self.triangles
    }

    /// Returns the model for this gadget, if it changed
    pub fn model(&self, gl: &Context) -> Ref<Model> {
        {
            let mut model = self.model.borrow_mut();

            if model.is_none() {
                *model = Some(Model::new(
                    gl,
                    &SHADERS.borrow()[&ShaderType::Basic],
                    &self.triangles,
                ));
            }
        }
        Ref::map(self.model.borrow(), |m| m.as_ref().unwrap())
    }

    pub(crate) fn new() -> Self {
        Self {
            triangles: Triangles::new(vec![], vec![]),
            //paths: FnvHashMap::default(),
            model: RefCell::new(None),
            grl: Rc::new(Grl::default()),
            grl_set: false,
        }
    }

    fn has_outline(&self, gadget: &Gadget) -> bool {
        gadget.def().num_states() > 1
    }

    /// Gets the path a robot takes to go from p0 to p1
    //fn port_path(ports: PP, port_positions: &Vec<Vec2>) -> Path {
    //    let positions: [Vec2; 2] = [port_positions[ports.0.id()], port_positions[ports.1.id()]];
    //    let mut bezier = [vec2(0.0, 0.0), vec2(0.0, 0.0)];

    //    let offset = 0.25;

    //    for (pos, bez) in positions.iter().zip(bezier.iter_mut()) {
    //        *bez = pos
    //            + if pos.x.floor() == pos.x {
    //                // on vertical edge
    //                if pos.x == 0.0 {
    //                    // on left edge
    //                    vec2(offset, 0.0)
    //                } else {
    //                    // on right edge
    //                    vec2(-offset, 0.0)
    //                }
    //            } else {
    //                // on horizontal edge
    //                if pos.y == 0.0 {
    //                    // on bottom edge
    //                    vec2(0.0, offset)
    //                } else {
    //                    // on top edge
    //                    vec2(0.0, -offset)
    //                }
    //            }
    //    }

    //    // Same-port traversal; make it look like a loop
    //    if bezier[0] == bezier[1] {
    //        let dv = (bezier[0] - positions[0]).right_ccw();

    //        bezier[0] += dv;
    //        bezier[1] -= dv;
    //    }

    //    Path::from_bezier3(
    //        [positions[0], bezier[0], bezier[1], positions[1]],
    //        GadgetRenderInfo::PATH_Z,
    //        0.05,
    //    )
    //}

    /// Updates the rendering information so
    /// that it is correct when rendering
    pub(crate) fn update(&mut self, gadget: &Gadget) {
        self.triangles.clear();
        //self.paths.clear();
        *self.model.borrow_mut() = None;

        // Surrounding rectangle
        self.triangles.append({
            let mut triangles = (*TRIANGLESES.borrow()[&TrianglesType::GadgetRectangle]).clone();

            for v in triangles.vertices_mut() {
                v.position.x *= gadget.size().0 as f32;
                v.position.y *= gadget.size().1 as f32;
            }

            triangles
        });

        // Port circles
        let port_positions = gadget.port_positions();
        for vec in port_positions.iter() {
            self.triangles.append(
                Circle::new(vec.x, vec.y, GadgetRenderInfo::PORT_Z, 0.05)
                    .triangles(vec4(0.0, 0.0, 0.75, 1.0)),
            );
        }

        // Outline
        if self.has_outline(gadget) {
            let path = Path::new(
                vec![
                    vec2(0.0, 0.0),
                    vec2(0.0, gadget.size().1 as f64),
                    vec2(gadget.size().0 as f64, gadget.size().1 as f64),
                    vec2(gadget.size().0 as f64, 0.0),
                ],
                GadgetRenderInfo::OUTLINE_Z,
                0.05,
                true,
            );

            self.triangles
                .append(path.triangles(vec4(0.0, 0.0, 0.0, 1.0)));
        }

        // Paths
        if !self.grl_set {
            self.grl_set = true;
            self.grl = lang::get_grl(gadget.def());
        } 

        let tris = self.grl.triangles(gadget);
        self.triangles.append(tris);
        //for ports in gadget.def().port_traversals_in_state(gadget.state()) {
        //    let path = GadgetRenderInfo::port_path(ports, &port_positions);

        //    self.paths.insert(ports, path);
        //}

        //for ((p0, p1), path) in &self.paths {
        //    let directed = self.paths.get(&(*p1, *p0)).is_none();

        //    // No redundant path drawing!
        //    if p0 <= p1 || directed {
        //        self.triangles
        //            .append(path.triangles(vec4(0.0, 0.0, 0.0, 1.0)));
        //    }

        //    if directed {
        //        let dir = path.end_direction();
        //        let end: Vec2 = port_positions[p1.id()];

        //        let v0: Vec2 = end + dir * -0.2 + dir.right_ccw() * -0.1;
        //        let v2: Vec2 = end + dir * -0.2 + dir.right_ccw() * 0.1;

        //        self.triangles.append(Triangles::new(
        //            vec![
        //                Vertex::new(
        //                    vec3(v0.x as f32, v0.y as f32, GadgetRenderInfo::PATH_Z as f32),
        //                    vec3(0.0, 0.0, 0.0),
        //                    vec4(0.0, 0.0, 0.0, 1.0),
        //                    [],
        //                ),
        //                Vertex::new(
        //                    vec3(end.x as f32, end.y as f32, GadgetRenderInfo::PATH_Z as f32),
        //                    vec3(0.0, 0.0, 0.0),
        //                    vec4(0.0, 0.0, 0.0, 1.0),
        //                    [],
        //                ),
        //                Vertex::new(
        //                    vec3(v2.x as f32, v2.y as f32, GadgetRenderInfo::PATH_Z as f32),
        //                    vec3(0.0, 0.0, 0.0),
        //                    vec4(0.0, 0.0, 0.0, 1.0),
        //                    [],
        //                ),
        //            ],
        //            vec![0, 1, 2],
        //        ));
        //    }
        //}
    }
}

impl Clone for GadgetRenderInfo {
    fn clone(&self) -> Self {
        Self {
            grl: Rc::clone(&self.grl),
            grl_set: self.grl_set,
            triangles: self.triangles.clone(),
            //paths: self.paths.clone(),
            model: RefCell::new(None),
        }
    }
}

pub trait GridItemRenderer {
    type Item;

    /// Start the rendering of the grid
    fn begin(&mut self, camera: &Camera);

    /// Render a specific item
    fn render(&mut self, item: Option<&Self::Item>, position: XY, size: WH, z: f64);

    /// Finalize the rendering of the grid
    fn end(&mut self);
}

pub struct GadgetRenderer {
    program: Rc<ShaderProgram>,
    gl: Rc<Context>,
    camera: Camera,
    /// Extra attributes: offset (vec3)
    triangles: TrianglesEx<[f32; 3]>,
    /// For the background
    background: Rc<Model>,
    instance_buffer: VertexBuffer,
    instance_positions: Vec<f32>,
}

impl GadgetRenderer {
    pub fn new(gl: &Rc<Context>) -> Self {
        Self {
            program: Rc::clone(&SHADERS.borrow()[&ShaderType::Offset]),
            gl: Rc::clone(gl),
            camera: Camera::new(),
            triangles: TrianglesEx::default(),
            background: Rc::clone(&MODELS.borrow()[&ModelType::GadgetRectangleInstanced]),
            instance_buffer: VertexBuffer::new(gl).unwrap(),
            instance_positions: vec![],
        }
    }

    pub fn render_gadget(&mut self, gadget: &Gadget, position: XY, _size: WH, z: f64) {
        gadget
            .renderer()
            .model(&self.gl)
            .prepare_render()
            .render_position(vec3(position.x as f64, position.y as f64, z), &self.camera);
    }
}

impl GridItemRenderer for GadgetRenderer {
    type Item = Gadget;

    /// Start the rendering of the grid
    fn begin(&mut self, camera: &Camera) {
        self.triangles.clear();
        self.instance_positions.clear();
        self.camera = camera.clone();
    }

    /// Render a specific item
    fn render(&mut self, item: Option<&Self::Item>, position: XY, size: WH, z: f64) {
        if let Some(gadget) = item {
            self.render_gadget(gadget, position, size, z);
        } else {
            let x = position.x as f32;
            let y = position.y as f32;
            self.instance_positions.extend_from_slice(&[
                x,
                y,
                (z + GadgetRenderInfo::RECTANGLE_Z) as f32,
            ]);
        }
    }

    /// Finalize the rendering of the grid
    fn end(&mut self) {
        let world_view_projection = self.camera.get_projection() * self.camera.get_view();

        self.program.bind();
        self.program
            .set_uniform(
                "transform",
                UniformValue::Matrix4(*world_view_projection.cast::<f32>().unwrap().as_ref()),
            )
            .unwrap();

        if self.instance_positions.len() > 0 {
            self.instance_buffer.set_data(&self.instance_positions);

            // Same program; transform already set
            self.background
                .prepare_render_instanced(&self.instance_buffer, &["v_offset"])
                .render_raw(self.instance_positions.len() as i32 / 3);
        }
    }
}

/// Renders selection marks in the contraption
pub struct SelectionRenderer {
    model: Rc<Model>,
    /// Scale (vec2) and offset (vec3)
    instance_data: Vec<f32>,
    instance_buffer: VertexBuffer,
}

impl SelectionRenderer {
    pub const Z: f64 = -0.2;

    pub fn new(gl: &Context) -> Self {
        Self {
            model: Rc::clone(&MODELS.borrow()[&ModelType::SelectionMarkInstanced]),
            instance_data: vec![],
            instance_buffer: VertexBuffer::new(gl).unwrap(),
        }
    }

    pub fn render(
        &mut self,
        selection: impl IntoIterator<Item = (XY, WH)>,
        camera: &Camera,
        offset: XY,
        z: f64,
    ) {
        self.instance_data.clear();

        let mut count = 0;

        for (Vec2i { x, y }, (w, h)) in selection.into_iter() {
            let w = w as isize;
            let h = h as isize;

            #[rustfmt::skip]
            self.instance_data.extend_from_slice(
                &[
                     1.0,  1.0, (offset.x + x    ) as f32, (offset.y + y    ) as f32, z as f32,
                    -1.0,  1.0, (offset.x + x + w) as f32, (offset.y + y    ) as f32, z as f32,
                    -1.0, -1.0, (offset.x + x + w) as f32, (offset.y + y + h) as f32, z as f32,
                     1.0, -1.0, (offset.x + x    ) as f32, (offset.y + y + h) as f32, z as f32,
                ]
            );

            count += 4;
        }

        if count == 0 {
            return;
        }

        self.instance_buffer.set_data(&self.instance_data);

        self.model
            .prepare_render_instanced(&self.instance_buffer, &["v_scale", "v_offset"])
            .render_position(vec3(0.0, 0.0, 0.0), camera, count);
    }
}

impl Agent {
    pub fn render(&self, camera: &Camera) {
        let dir = self.direction().cast::<f64>().unwrap();

        let transform = Mat4::from_cols(
            -dir.right_ccw().extend(0.0).extend(0.0),
            dir.extend(0.0).extend(0.0),
            vec4(0.0, 0.0, 1.0, 0.0),
            (self.position()).extend(-0.1).extend(1.0),
        );
        MODELS.borrow()[&ModelType::Agent]
            .prepare_render()
            .render(transform, camera);
    }
}
