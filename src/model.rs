use golem::Dimension::{D3, D4};
use golem::{Attribute, AttributeType, Uniform, UniformType, UniformValue};
use golem::{Context, ShaderDescription, ShaderProgram};
use golem::{ElementBuffer, GeometryMode, VertexBuffer};
use std::rc::Rc;

use crate::camera::Camera;
use crate::log;
use crate::math::{Mat4, ToArray, Vec3};

/// A simple model.
pub struct Model {
    gl: Rc<Context>,
    program: ShaderProgram,
    vertices: VertexBuffer,
    indexes: ElementBuffer,
    num_indexes: usize,
}

impl Model {
    /// 3 coordinates per position, 4 components per color, 3 indexes per triangle
    pub fn new(
        gl: &Rc<Context>,
        positions: &Vec<f32>,
        colors: &Vec<f32>,
        indexes: &Vec<u32>,
    ) -> Self {
        let program = ShaderProgram::new(
            gl,
            ShaderDescription {
                vertex_input: &[
                    Attribute::new("v_position", AttributeType::Vector(D3)),
                    Attribute::new("v_color", AttributeType::Vector(D4)),
                ],
                fragment_input: &[Attribute::new("f_color", AttributeType::Vector(D4))],
                uniforms: &[Uniform::new("transform", UniformType::Matrix(D4))],
                vertex_shader: r#"void main() {
                    f_color = v_color;
                    gl_Position = transform * vec4(v_position, 1.0);
                }"#,
                fragment_shader: r#"void main() {
                    gl_FragColor = f_color;
                }"#,
            },
        )
        .unwrap();

        let vertices = positions
            .chunks(3)
            .zip(colors.chunks(4))
            .flat_map(|(p, c)| p.iter().chain(c.iter()))
            .copied()
            .collect::<Vec<_>>();

        let mut vertex_buffer = VertexBuffer::new(gl).unwrap();
        vertex_buffer.set_data(&vertices);

        let mut index_buffer = ElementBuffer::new(gl).unwrap();
        index_buffer.set_data(&indexes);

        Self {
            gl: Rc::clone(gl),
            program,
            vertices: vertex_buffer,
            indexes: index_buffer,
            num_indexes: indexes.len(),
        }
    }

    pub fn prepare_render(&self) -> RenderingModel {
        self.program.bind_if_not_bound();

        self.program.prepare_draw(&self.vertices, &self.indexes).unwrap();

        RenderingModel(self)
    }

    fn render(&self, transform: Mat4, camera: &Camera) {
        let transform: Mat4 = camera.get_projection() * camera.get_view() * transform;

        self.program
            .set_uniform(
                "transform",
                UniformValue::Matrix4(transform.cast::<f32>().unwrap().to_array()),
            )
            .unwrap();

        unsafe {
            self.program
                .draw_prepared(
                    0..self.num_indexes,
                    GeometryMode::Triangles,
                )
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