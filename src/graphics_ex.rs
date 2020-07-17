use conrod_core::render::{Primitive, PrimitiveKind, PrimitiveWalker};
use std::rc::Rc;
use three_d::gl::Glstruct;
use three_d::{vec3, Camera, ElementBuffer, Program, VertexBuffer};

use crate::log;
use crate::shape::{Rectangle, Shape};

pub struct GraphicsEx {
    gl: Rc<Glstruct>,
    camera: Camera,
    program: Program,
    /// 3 coordinates (XYZ) per vertex
    positions: Vec<f32>,
    /// 3 coordinates (XYZ) per offset
    offsets: Vec<f32>,
    /// 4 components (RGBA) per color
    colors: Vec<f32>,
    /// 3 indexes per triangle
    indexes: Vec<u32>,
}

impl GraphicsEx {
    const UI_Z_BASE: f32 = -0.9;

    pub fn new(gl: &Rc<Glstruct>) -> Self {
        // dummy values
        let camera = Camera::new_orthographic(
            gl,
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, -1.0),
            vec3(0.0, 1.0, 0.0),
            1.0,
            1.0,
            1.0,
        );

        let program = Program::from_source(
            gl,
            include_str!("../assets/shaders/color.vert"),
            include_str!("../assets/shaders/color.frag"),
        )
        .unwrap();

        Self {
            gl: Rc::clone(gl),
            camera,
            program,
            positions: vec![],
            offsets: vec![],
            colors: vec![],
            indexes: vec![],
        }
    }

    /// Starts the drawing process
    pub fn draw_begin(&mut self, width: f32, height: f32) {
        self.positions.clear();
        self.offsets.clear();
        self.indexes.clear();
        self.indexes.clear();
        self.camera.set_orthographic_projection(width, height, 1.0);
    }

    /// Finishes the drawing process
    pub fn draw_end(&mut self) {
        let world_view_projection = self.camera.get_projection() * self.camera.get_view();
        self.program
            .add_uniform_mat4("worldViewProjectionMatrix", &world_view_projection)
            .unwrap();

        let positions = VertexBuffer::new_with_static_f32(&self.gl, &self.positions).unwrap();
        let offsets = VertexBuffer::new_with_static_f32(&self.gl, &self.offsets).unwrap();
        let colors = VertexBuffer::new_with_static_f32(&self.gl, &self.colors).unwrap();
        let elements = ElementBuffer::new_with_u32(&self.gl, &self.indexes).unwrap();

        self.program
            .use_attribute_vec3_float(&positions, "position")
            .unwrap();
        self.program
            .use_attribute_vec3_float(&offsets, "offset")
            .unwrap();
        self.program
            .use_attribute_vec4_float(&colors, "color")
            .unwrap();

        self.program.draw_elements(&elements);
    }

    pub fn primitive(&mut self, p: Primitive) {
        let Primitive { id, kind, rect, .. } = p;

        let (x, y, w, h) = rect.x_y_w_h();
        let x = x as f32;
        let y = y as f32;
        let w = w as f32;
        let h = h as f32;

        match kind {
            PrimitiveKind::Rectangle { color } => {
                let rect = Rectangle::new(-w / 2.0, w / 2.0, -h / 2.0, h / 2.0, 0.0);

                rect.append_to(&mut self.positions, &mut self.indexes);

                self.offsets
                    .extend([x, y, GraphicsEx::UI_Z_BASE].iter().cycle().take(4 * 3));
                let rgba = color.to_rgb();
                self.colors
                    .extend([rgba.0, rgba.1, rgba.2, rgba.3].iter().cycle().take(4 * 4));
            }

            PrimitiveKind::TrianglesSingleColor {
                color: rgba,
                triangles,
            } => {
                let p_len = self.positions.len() as u32 / 3;
                self.positions.extend(triangles.iter().flat_map(|t| {
                    vec![
                        t[0][0] as f32,
                        t[0][1] as f32,
                        0.0,
                        t[1][0] as f32,
                        t[1][1] as f32,
                        0.0,
                        t[2][0] as f32,
                        t[2][1] as f32,
                        0.0,
                    ]
                }));
                let t_len = triangles.len() as u32 * 3;
                self.indexes.extend(p_len..(p_len + t_len));

                self.offsets.extend(
                    [0.0, 0.0, GraphicsEx::UI_Z_BASE]
                        .iter()
                        .cycle()
                        .take(t_len as usize * 3),
                );
                self.colors.extend(
                    [rgba.0, rgba.1, rgba.2, rgba.3]
                        .iter()
                        .cycle()
                        .take(t_len as usize * 4),
                );
            }

            PrimitiveKind::TrianglesMultiColor { triangles } => {
                let p_len = self.positions.len() as u32 / 3;
                self.positions.extend(triangles.iter().flat_map(|t| {
                    vec![
                        t[0].0[0] as f32,
                        t[0].0[1] as f32,
                        0.0,
                        t[1].0[0] as f32,
                        t[1].0[1] as f32,
                        0.0,
                        t[2].0[0] as f32,
                        t[2].0[1] as f32,
                        0.0,
                    ]
                }));
                let t_len = triangles.len() as u32 * 3;
                self.indexes.extend(p_len..(p_len + t_len));

                self.offsets.extend(
                    [0.0, 0.0, GraphicsEx::UI_Z_BASE]
                        .iter()
                        .cycle()
                        .take(t_len as usize * 3),
                );
                self.colors.extend(triangles.iter().flat_map(|t| {
                    vec![
                        t[0].1 .0, t[0].1 .1, t[0].1 .2, t[0].1 .3, t[1].1 .0, t[1].1 .1,
                        t[1].1 .2, t[1].1 .3, t[2].1 .0, t[2].1 .1, t[2].1 .2, t[2].1 .3,
                    ]
                }));
            }

            PrimitiveKind::Image {
                image_id,
                color,
                source_rect,
            } => {
                unimplemented!("Images are not supported");
            }

            PrimitiveKind::Text {
                color,
                text,
                font_id,
            } => {
                unimplemented!("Text is not supported");
            }

            PrimitiveKind::Other(widget) => {}
        }
    }
}
