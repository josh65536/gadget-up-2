use fnv::FnvHashMap;
use golem::Dimension::{D2, D3, D4};
use golem::{Attribute, AttributeType, Uniform, UniformType, UniformValue};
use golem::{Context, ShaderDescription, ShaderProgram};
use golem::{ElementBuffer, GeometryMode, VertexBuffer};
use std::rc::Rc;

/// The type of shader to use
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum ShaderType {
    /// Just position and color
    Basic,
    /// Add a offset (vec3)
    Offset,
    /// Add a scale (vec2) and an offset (vec3)
    ScaleOffset,
}

pub type ShaderMap = FnvHashMap<ShaderType, Rc<ShaderProgram>>;

pub fn shader_map(gl: &Context) -> ShaderMap {
    [
        (
            ShaderType::Basic,
            Rc::new(ShaderProgram::new(
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
            ).unwrap())
        ),
        (
            ShaderType::Offset,
            Rc::new(ShaderProgram::new(
                gl,
                ShaderDescription {
                    vertex_input: &[
                        Attribute::new("v_position", AttributeType::Vector(D3)),
                        Attribute::new("v_color", AttributeType::Vector(D4)),
                        Attribute::new("v_offset", AttributeType::Vector(D3)),
                    ],
                    fragment_input: &[Attribute::new("f_color", AttributeType::Vector(D4))],
                    uniforms: &[Uniform::new("transform", UniformType::Matrix(D4))],
                    vertex_shader: r#"void main() {
                        f_color = v_color;
                        gl_Position = transform * vec4(v_position + v_offset, 1.0);
                    }"#,
                    fragment_shader: r#"void main() {
                        gl_FragColor = f_color;
                    }"#,
                },
            ).unwrap())
        ),
        (
            ShaderType::Offset,
            Rc::new(ShaderProgram::new(
                gl,
                ShaderDescription {
                    vertex_input: &[
                        Attribute::new("v_position", AttributeType::Vector(D3)),
                        Attribute::new("v_color", AttributeType::Vector(D4)),
                        Attribute::new("v_scale", AttributeType::Vector(D2)),
                        Attribute::new("v_offset", AttributeType::Vector(D3)),
                    ],
                    fragment_input: &[Attribute::new("f_color", AttributeType::Vector(D4))],
                    uniforms: &[Uniform::new("transform", UniformType::Matrix(D4))],
                    vertex_shader: r#"void main() {
                        f_color = v_color;
                        gl_Position = transform * vec4(v_position * vec3(v_scale, 1.0) + v_offset, 1.0);
                    }"#,
                    fragment_shader: r#"void main() {
                        gl_FragColor = f_color;
                    }"#,
                },
            ).unwrap())
        ),
    ].iter().cloned().collect()
}
