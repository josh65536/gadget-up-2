use super::{Camera, ShaderType, TrianglesEx, TrianglesType, SHADERS, TRIANGLESES};
use super::{MODELS, ModelType, Triangles, Vertex, Model};
use crate::gadget::{Gadget, PP, Agent};
use crate::grid::{Grid, WH, XY};
use crate::log;
use crate::shape::{Rectangle, Shape, Path, Circle};
use crate::math::{Vec2, Vector2Ex, Mat4};

use cgmath::{vec2, vec3, vec4};
use golem::Dimension::{D3, D4};
use golem::{Attribute, AttributeType, Uniform, UniformType, UniformValue};
use golem::{Context, ShaderDescription, ShaderProgram};
use golem::{ElementBuffer, GeometryMode, VertexBuffer};
use itertools::izip;
use ref_thread_local::RefThreadLocal;
use std::rc::Rc;
use std::cell::{Ref, RefCell};
use fnv::FnvHashMap;

pub struct GadgetRenderInfo {
    triangles: Triangles,
    paths: FnvHashMap<PP, Path>,
    /// Cached model
    model: RefCell<Option<Model>>,
}

impl GadgetRenderInfo {
    pub const RECTANGLE_Z: f64 = -0.001;
    const OUTLINE_Z: f64 = -0.002;
    const PATH_Z: f64 = -0.003;
    const PORT_Z: f64 = -0.004;

    pub fn triangles(&self) -> &Triangles {
        &self.triangles
    }

    /// Returns the model for this gadget, if it changed
    pub fn model(&self, gl: &Context) -> Ref<Model> {
        {
            let mut model = self.model.borrow_mut();

            if model.is_none() {
                *model = Some(Model::new(gl, &SHADERS.borrow()[ShaderType::Basic], &self.triangles));
            }
        }
        Ref::map(self.model.borrow(), |m| m.as_ref().unwrap())
    }

    pub(crate) fn new() -> Self {
        Self {
            triangles: Triangles::new(vec![], vec![]),
            paths: FnvHashMap::default(),
            model: RefCell::new(None),
        }
    }

    fn has_outline(&self, gadget: &Gadget) -> bool {
        gadget.def().num_states() > 1
    }

    /// Gets the path a robot takes to go from p0 to p1
    fn port_path(ports: PP, port_positions: &Vec<Vec2>) -> Path {
        let positions = [
            port_positions[ports.0 as usize],
            port_positions[ports.1 as usize],
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

        Path::from_bezier3(
            [positions[0], bezier[0], bezier[1], positions[1]],
            GadgetRenderInfo::PATH_Z,
            0.05,
        )
    }

    /// Updates the rendering information so
    /// that it is correct when rendering
    pub(crate) fn update(&mut self, gadget: &Gadget) {
        self.triangles.clear();
        self.paths.clear();
        *self.model.borrow_mut() = None;

        // Surrounding rectangle
        self.triangles.append({
            let mut triangles = (*TRIANGLESES.borrow()[TrianglesType::GadgetRectangle]).clone();

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
        for ports in gadget.def().port_traversals_in_state(gadget.state()) {
            let path = GadgetRenderInfo::port_path(ports, &port_positions);

            self.paths.insert(ports, path);
        }

        for ((p0, p1), path) in &self.paths {
            let directed = self.paths.get(&(*p1, *p0)).is_none();

            // No redundant path drawing!
            if p0 <= p1 || directed {
                self.triangles
                    .append(path.triangles(vec4(0.0, 0.0, 0.0, 1.0)));
            }

            if directed {
                let dir = path.end_direction();
                let end = port_positions[*p1 as usize];

                let v0 = end + dir * -0.2 + dir.right_ccw() * -0.1;
                let v2 = end + dir * -0.2 + dir.right_ccw() * 0.1;

                self.triangles.append(Triangles::new(
                    vec![
                        Vertex::new(
                            vec3(v0.x as f32, v0.y as f32, GadgetRenderInfo::PATH_Z as f32),
                            vec4(0.0, 0.0, 0.0, 1.0),
                            [],
                        ),
                        Vertex::new(
                            vec3(end.x as f32, end.y as f32, GadgetRenderInfo::PATH_Z as f32),
                            vec4(0.0, 0.0, 0.0, 1.0),
                            [],
                        ),
                        Vertex::new(
                            vec3(v2.x as f32, v2.y as f32, GadgetRenderInfo::PATH_Z as f32),
                            vec4(0.0, 0.0, 0.0, 1.0),
                            [],
                        ),
                    ],
                    vec![0, 1, 2],
                ));
            }
        }
    }
}

impl Clone for GadgetRenderInfo {
    fn clone(&self) -> Self {
        Self {
            triangles: self.triangles.clone(),
            paths: self.paths.clone(),
            model: RefCell::new(None),
        }
    }
}

pub trait GridItemRenderer {
    type Item;

    /// Start the rendering of the grid
    fn begin(&mut self);

    /// Render a specific item
    fn render(&mut self, item: Option<&Self::Item>, position: XY, size: WH);

    /// Finalize the rendering of the grid
    fn end(&mut self, camera: &Camera);
}

pub struct GadgetRenderer {
    program: Rc<ShaderProgram>,
    gl: Rc<Context>,
    /// Extra attributes: offset (vec3)
    triangles: TrianglesEx<[f32; 3]>,
    vertex_buffer: VertexBuffer,
    index_buffer: ElementBuffer,
}

impl GadgetRenderer {
    pub fn new(gl: &Rc<Context>) -> Self {
        Self {
            program: Rc::clone(&SHADERS.borrow()[ShaderType::Offset]),
            gl: Rc::clone(gl),
            triangles: TrianglesEx::default(),
            vertex_buffer: VertexBuffer::new(gl).unwrap(),
            index_buffer: ElementBuffer::new(gl).unwrap(),
        }
    }

    pub fn render_gadget(&mut self, gadget: &Gadget, position: XY, _size: WH) {
        let x = position.x as f32;
        let y = position.y as f32;

        self.triangles.append(
            gadget
                .renderer()
                .triangles()
                .clone()
                .with_extra([x, y, 0.0]),
        );
    }
}

impl GridItemRenderer for GadgetRenderer {
    type Item = Gadget;

    /// Start the rendering of the grid
    fn begin(&mut self) {
        self.triangles.clear();
    }

    /// Render a specific item
    fn render(&mut self, item: Option<&Self::Item>, position: XY, size: WH) {
        if let Some(gadget) = item {
            self.render_gadget(gadget, position, size);
        } else {
            let x = position.x as f32;
            let y = position.y as f32;

            self.triangles.append(
                (*TRIANGLESES.borrow()[TrianglesType::GadgetRectangle])
                    .clone()
                    .with_extra([x, y, GadgetRenderInfo::RECTANGLE_Z as f32]),
            );
        }
    }

    /// Finalize the rendering of the grid
    fn end(&mut self, camera: &Camera) {
        let world_view_projection = camera.get_projection() * camera.get_view();

        self.program.bind();
        self.program
            .set_uniform(
                "transform",
                UniformValue::Matrix4(*world_view_projection.cast::<f32>().unwrap().as_ref()),
            )
            .unwrap();

        self.vertex_buffer
            .set_data(&self.triangles.iter_vertex_items().collect::<Vec<_>>());
        self.index_buffer.set_data(&self.triangles.indexes());

        unsafe {
            self.program
                .draw(
                    &self.vertex_buffer,
                    &self.index_buffer,
                    0..self.triangles.indexes().len(),
                    GeometryMode::Triangles,
                )
                .unwrap();
        }
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
        MODELS.borrow()[ModelType::Agent].prepare_render().render(transform, camera);
    }
}