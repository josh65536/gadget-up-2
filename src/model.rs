use std::rc::Rc;
use three_d::gl::Glstruct;
use three_d::{Camera, ElementBuffer, Mat4, Program, Vec3, VertexBuffer};

use crate::log;

/// A simple model.
pub struct Model {
    gl: Rc<Glstruct>,
    program: Program,
    positions: VertexBuffer,
    colors: VertexBuffer,
    indexes: ElementBuffer,
}

impl Model {
    /// 3 coordinates per position, 4 components per color, 3 indexes per triangle
    pub fn new(
        gl: &Rc<Glstruct>,
        positions: &Vec<f32>,
        colors: &Vec<f32>,
        indexes: &Vec<u32>,
    ) -> Self {
        let program = Program::from_source(
            gl,
            include_str!("../assets/shaders/basic.vert"),
            include_str!("../assets/shaders/basic.frag"),
        )
        .unwrap();

        let positions = VertexBuffer::new_with_static_f32(gl, &positions).unwrap();
        let colors = VertexBuffer::new_with_static_f32(gl, &colors).unwrap();
        let indexes = ElementBuffer::new_with_u32(gl, &indexes).unwrap();

        Self {
            gl: Rc::clone(gl),
            program,
            positions,
            colors,
            indexes,
        }
    }

    pub fn render(&self, transform: Mat4, camera: &Camera) {
        let transform = camera.get_projection() * camera.get_view() * transform;
        self.program
            .add_uniform_mat4("worldViewProjectionMatrix", &transform)
            .unwrap();

        self.program
            .use_attribute_vec3_float(&self.positions, "position")
            .unwrap();
        self.program
            .use_attribute_vec4_float(&self.colors, "color")
            .unwrap();

        self.program.draw_elements(&self.indexes);
    }

    pub fn render_position(&self, position: Vec3, camera: &Camera) {
        self.render(Mat4::from_translation(position), camera);
    }
}
