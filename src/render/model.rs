use cgmath::{vec2, vec3, vec4, Vector2, Vector3, Vector4};
use fnv::FnvHashMap;
use golem::Dimension::{D3, D4};
use golem::{Attribute, AttributeType, Uniform, UniformType, UniformValue};
use golem::{Context, ShaderDescription, ShaderProgram};
use golem::{ElementBuffer, GeometryMode, VertexBuffer};
use ref_thread_local::{ref_thread_local, RefThreadLocal};
use std::rc::Rc;

use super::{Camera, ShaderType, SHADERS};
use crate::log;
use crate::math::{Mat4, Vec3};
use crate::static_map::StaticMap;

pub type Vertex = VertexEx<[f32; 0]>;

/// Stores the information for a single vertex.
#[derive(Clone, Debug)]
pub struct VertexEx<T: AsRef<[f32]> + Copy> {
    // Edit the num_floats function if fields are modified!
    pub position: Vector3<f32>,
    // Z coordinate is 0 for no texture and 1 for texture
    pub tex_coord: Vector3<f32>,
    pub color: Vector4<f32>,
    pub extra: T,
}

impl<T: AsRef<[f32]> + Copy> VertexEx<T> {
    pub fn new(
        position: Vector3<f32>,
        tex_coord: Vector3<f32>,
        color: Vector4<f32>,
        extra: T,
    ) -> Self {
        Self {
            position,
            tex_coord,
            color,
            extra,
        }
    }

    /// Gets an iterator of attribute items,
    /// putting position, then color, then extra items
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = f32> + 'a {
        AsRef::<[f32; 3]>::as_ref(&self.position)
            .iter()
            .chain(AsRef::<[f32; 3]>::as_ref(&self.tex_coord).iter())
            .chain(AsRef::<[f32; 4]>::as_ref(&self.color).iter())
            .chain(self.extra.as_ref().iter())
            .copied()
    }

    /// Gets the number of f32's this vertex takes
    pub fn num_floats(&self) -> usize {
        3 + 3 + 4 + self.extra.as_ref().len()
    }
}

impl Vertex {
    /// Gets a new vertex with extra attribute items added
    pub fn with_extra<U: AsRef<[f32]> + Copy>(self, extra: U) -> VertexEx<U> {
        VertexEx {
            position: self.position,
            tex_coord: self.tex_coord,
            color: self.color,
            extra,
        }
    }

    /// Gets a new vertex with default extra attribute items added
    pub fn with_default_extra<U: AsRef<[f32]> + Copy + Default>(self) -> VertexEx<U> {
        self.with_extra(U::default())
    }
}

pub type Triangles = TrianglesEx<[f32; 0]>;

/// Stores the information for multiple triangles.
#[derive(Clone, Debug, Default)]
pub struct TrianglesEx<T: AsRef<[f32]> + Copy> {
    vertices: Vec<VertexEx<T>>,
    indexes: Vec<u32>,
}

impl<T: AsRef<[f32]> + Copy> TrianglesEx<T> {
    pub fn new(vertices: Vec<VertexEx<T>>, indexes: Vec<u32>) -> Self {
        Self { vertices, indexes }
    }

    /// Gets an iterator of attribute items for each vertex in order
    pub fn iter_vertex_items<'a>(&'a self) -> impl Iterator<Item = f32> + 'a {
        self.vertices.iter().flat_map(move |v| v.iter())
    }

    pub fn vertices(&self) -> &[VertexEx<T>] {
        &self.vertices
    }

    pub fn vertices_mut(&mut self) -> &mut [VertexEx<T>] {
        &mut self.vertices
    }

    pub fn indexes(&self) -> &[u32] {
        &self.indexes
    }

    /// Takes ownership of the other set of triangles because
    /// this will often be called with temporary triangle structures
    pub fn append(&mut self, other: TrianglesEx<T>) {
        let base_index = self.vertices.len() as u32;

        let TrianglesEx { vertices, indexes } = other;
        self.vertices.extend(vertices.into_iter());
        self.indexes
            .extend(indexes.into_iter().map(|i| i + base_index));
    }

    /// Clears the triangle list
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indexes.clear();
    }

    ///// Appends the triangles to a list of vertex attributes and indexes,
    ///// adding extra vertex attributes to each vertex after their position and color.
    ///// Assumes that `extra` is the same length as the extra attribute items
    ///// that have already been added to `vertices`.
    pub fn append_to(&self, vertices: &mut Vec<f32>, indexes: &mut Vec<u32>) {
        let base_index =
            (vertices.len() / self.vertices.first().map_or(1, |v| v.num_floats())) as u32;

        vertices.extend(self.iter_vertex_items());
        indexes.extend(self.indexes.iter().map(|i| *i + base_index));
    }
}

impl Triangles {
    /// Converts this to a triangle list where `extra` has been
    /// added to each vertex's attribute items
    pub fn with_extra<U: AsRef<[f32]> + Copy>(self, extra: U) -> TrianglesEx<U> {
        TrianglesEx {
            vertices: self
                .vertices
                .into_iter()
                .map(|v| v.with_extra(extra))
                .collect(),
            indexes: self.indexes,
        }
    }

    /// Converts this to a triangle list where default extra items have been
    /// added to each vertex's attribute items
    pub fn with_default_extra<U: AsRef<[f32]> + Copy + Default>(self) -> TrianglesEx<U> {
        self.with_extra(U::default())
    }
}

/// A simple model.
pub struct Model {
    program: Rc<ShaderProgram>,
    vertex_buffer: VertexBuffer,
    index_buffer: ElementBuffer,
    num_indexes: usize,
}

impl Model {
    /// Construct a model out of triangles
    pub fn new<T: AsRef<[f32]> + Copy>(
        gl: &Context,
        program: &Rc<ShaderProgram>,
        triangles: &TrianglesEx<T>,
    ) -> Self {
        let vertices = triangles.iter_vertex_items().collect::<Vec<_>>();

        let mut vertex_buffer = VertexBuffer::new(gl).unwrap();
        vertex_buffer.set_data(&vertices);

        let mut index_buffer = ElementBuffer::new(gl).unwrap();
        index_buffer.set_data(&triangles.indexes());

        Self {
            program: Rc::clone(program),
            vertex_buffer,
            index_buffer,
            num_indexes: triangles.indexes().len(),
        }
    }

    pub fn prepare_render(&self) -> RenderingModel {
        self.program.bind_if_not_bound();

        self.program
            .prepare_draw(&self.vertex_buffer, &self.index_buffer)
            .unwrap();

        RenderingModel(self)
    }

    fn render(&self, transform: Mat4, camera: &Camera) {
        let transform: Mat4 = camera.get_projection() * camera.get_view() * transform;

        self.program
            .set_uniform(
                "transform",
                UniformValue::Matrix4(*transform.cast::<f32>().unwrap().as_ref()),
            )
            .unwrap();

        unsafe {
            self.program
                .draw_prepared(0..self.num_indexes, GeometryMode::Triangles)
        }
    }
}

/// An RAII guard for a model that has been prepared to render.
/// If render is called multiple times while holding the same
/// guard, unnecessary preparation calls will not happen.
pub struct RenderingModel<'a>(&'a Model);

impl<'a> RenderingModel<'a> {
    pub fn render(&self, transform: Mat4, camera: &Camera) {
        self.0.render(transform, camera)
    }

    pub fn render_position(&self, position: Vec3, camera: &Camera) {
        self.render(Mat4::from_translation(position), camera);
    }
}

/// Names of triangles structures
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TrianglesType {
    Agent,
    GadgetRectangle,
}

type TrianglesMap = FnvHashMap<TrianglesType, Rc<Triangles>>;

#[rustfmt::skip]
fn triangles_map(_: &()) -> TrianglesMap {
    [
        (
            TrianglesType::Agent,
            Rc::new(Triangles::new(
                vec![
                    Vertex::new(vec3( 0.15, -0.15, 0.), vec3(0., 0., 0.), vec4(0., 0.8, 0., 1.), []),
                    Vertex::new(vec3( 0.15,  0.,   0.), vec3(0., 0., 0.), vec4(0., 0.6, 0., 1.), []),
                    Vertex::new(vec3( 0.,    0.15, 0.), vec3(0., 0., 0.), vec4(0., 0.4, 0., 1.), []),
                    Vertex::new(vec3(-0.15,  0.,   0.), vec3(0., 0., 0.), vec4(0., 0.6, 0., 1.), []),
                    Vertex::new(vec3(-0.15, -0.15, 0.), vec3(0., 0., 0.), vec4(0., 0.8, 0., 1.), []),
                ],
                vec![0, 1, 2, 0, 2, 4, 2, 3, 4],
            )),
        ),
        (
            TrianglesType::GadgetRectangle,
            Rc::new(Triangles::new(
                vec![
                    Vertex::new(vec3(0., 0., 0.), vec3(0., 0., 0.), vec4(0.6, 0.8, 1., 1.), []),
                    Vertex::new(vec3(1., 0., 0.), vec3(0., 0., 0.), vec4(0.7, 0.9, 1., 1.), []),
                    Vertex::new(vec3(1., 1., 0.), vec3(0., 0., 0.), vec4(0.9, 1.0, 1., 1.), []),
                    Vertex::new(vec3(0., 1., 0.), vec3(0., 0., 0.), vec4(0.8, 1.0, 1., 1.), []),
                ],
                vec![0, 1, 2, 2, 3, 0],
            )),
        ),
    ]
    .iter()
    .cloned()
    .collect()
}

ref_thread_local!(
    pub static managed TRIANGLESES: StaticMap<TrianglesType, Rc<Triangles>, fn(&()) -> TrianglesMap, ()> = StaticMap::new(
        triangles_map
    );
);

/// Names of models
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ModelType {
    Agent,
}

type ModelMap = FnvHashMap<ModelType, Rc<Model>>;

ref_thread_local!(
    pub static managed MODELS: StaticMap<ModelType, Rc<Model>, fn(&Context) -> ModelMap, Context> = StaticMap::new(
        model_map
    );
);

fn model_map(gl: &Context) -> ModelMap {
    [(
        ModelType::Agent,
        Rc::new(Model::new(
            gl,
            &SHADERS.borrow()[ShaderType::Basic],
            &TRIANGLESES.borrow()[TrianglesType::Agent],
        )),
    )]
    .iter()
    .cloned()
    .collect()
}
