use cgmath::{vec3, vec4};
use conrod_core::render::{Primitive, PrimitiveKind};
use conrod_core::Widget;
use golem::Dimension::{D3, D4};
use golem::{Attribute, AttributeType, Uniform, UniformType, UniformValue};
use golem::{Context, ShaderDescription, ShaderProgram};
use golem::{ElementBuffer, GeometryMode, VertexBuffer};
use itertools::izip;
use ref_thread_local::RefThreadLocal;
use std::rc::Rc;

use super::{Camera, TrianglesEx, TrianglesType, VertexEx, TRIANGLESES};
use super::{SHADERS, ShaderType};
use crate::log;
use crate::shape::{Rectangle, Shape};
use crate::widget::triangles3d::Triangles3d;

pub struct UiRenderer {
    gl: Rc<Context>,
    pub camera: Camera,
    program: Rc<ShaderProgram>,
    pub triangles: TrianglesEx<[f32; 5]>,
    vertex_buffer: VertexBuffer,
    index_buffer: ElementBuffer,
}

impl UiRenderer {
    pub const UI_Z_BASE: f64 = -0.9;

    pub fn new(gl: &Rc<Context>) -> Self {
        // dummy values
        let camera = Camera::new_orthographic(
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, -1.0),
            vec3(0.0, 1.0, 0.0),
            1.0,
            1.0,
            1.0,
        );

        Self {
            gl: Rc::clone(gl),
            camera,
            program: Rc::clone(&SHADERS.borrow()[ShaderType::ScaleOffset]),
            triangles: TrianglesEx::default(),
            vertex_buffer: VertexBuffer::new(gl).unwrap(),
            index_buffer: ElementBuffer::new(gl).unwrap(),
        }
    }

    /// Starts the drawing process
    pub fn draw_begin(&mut self, width: f64, height: f64) {
        self.triangles.clear();
        self.camera.set_orthographic_projection(width, height, 1.0);
    }

    /// Finishes the drawing process
    pub fn draw_end(&mut self) {
        let world_view_projection = self.camera.get_projection() * self.camera.get_view();

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

    pub fn primitive(&mut self, p: Primitive) {
        let Primitive {
            id: _, kind, rect, ..
        } = p;

        let (x, y, w, h) = rect.x_y_w_h();

        match kind {
            PrimitiveKind::Rectangle { color } => {
                let rgba = color.to_rgb();

                self.triangles.append(
                    Rectangle::new(-w / 2.0, w / 2.0, -h / 2.0, h / 2.0, 0.0)
                        .triangles(vec4(rgba.0, rgba.1, rgba.2, rgba.3))
                        .with_extra([1.0, 1.0, x as f32, y as f32, UiRenderer::UI_Z_BASE as f32]),
                );
            }

            PrimitiveKind::TrianglesSingleColor {
                color: rgba,
                triangles,
            } => {
                let color = vec4(rgba.0, rgba.1, rgba.2, rgba.3);
                let extra = [1.0, 1.0, 0.0, 0.0, 0.0];

                self.triangles.append(TrianglesEx::new(
                    triangles
                        .iter()
                        .flat_map(|t| {
                            vec![
                                VertexEx::new(
                                    vec3(
                                        t[0][0] as f32,
                                        t[0][1] as f32,
                                        UiRenderer::UI_Z_BASE as f32,
                                    ),
                                    color,
                                    extra,
                                ),
                                VertexEx::new(
                                    vec3(
                                        t[1][0] as f32,
                                        t[1][1] as f32,
                                        UiRenderer::UI_Z_BASE as f32,
                                    ),
                                    color,
                                    extra,
                                ),
                                VertexEx::new(
                                    vec3(
                                        t[2][0] as f32,
                                        t[2][1] as f32,
                                        UiRenderer::UI_Z_BASE as f32,
                                    ),
                                    color,
                                    extra,
                                ),
                            ]
                            .into_iter()
                        })
                        .collect(),
                    (0..(triangles.len() as u32 * 3)).collect(),
                ));
            }

            PrimitiveKind::TrianglesMultiColor { triangles } => {
                let extra = [1.0, 1.0, 0.0, 0.0, 0.0];

                self.triangles.append(TrianglesEx::new(
                    triangles
                        .iter()
                        .flat_map(|t| {
                            vec![
                                VertexEx::new(
                                    vec3(
                                        t[0].0[0] as f32,
                                        t[0].0[1] as f32,
                                        UiRenderer::UI_Z_BASE as f32,
                                    ),
                                    vec4(t[0].1 .0, t[0].1 .1, t[0].1 .2, t[0].1 .3),
                                    extra,
                                ),
                                VertexEx::new(
                                    vec3(
                                        t[1].0[0] as f32,
                                        t[1].0[1] as f32,
                                        UiRenderer::UI_Z_BASE as f32,
                                    ),
                                    vec4(t[0].1 .0, t[0].1 .1, t[0].1 .2, t[0].1 .3),
                                    extra,
                                ),
                                VertexEx::new(
                                    vec3(
                                        t[2].0[0] as f32,
                                        t[2].0[1] as f32,
                                        UiRenderer::UI_Z_BASE as f32,
                                    ),
                                    vec4(t[0].1 .0, t[0].1 .1, t[0].1 .2, t[0].1 .3),
                                    extra,
                                ),
                            ]
                            .into_iter()
                        })
                        .collect(),
                    (0..(triangles.len() as u32 * 3)).collect(),
                ));
            }

            PrimitiveKind::Image {
                image_id: _,
                color: _,
                source_rect: _,
            } => {
                unimplemented!("Images are not supported");
            }

            PrimitiveKind::Text {
                color: _,
                text: _,
                font_id: _,
            } => {
                unimplemented!("Text is not supported");
            }

            PrimitiveKind::Other(widget) => {
                if widget.type_id == std::any::TypeId::of::<<Triangles3d as Widget>::State>() {
                    if let Some(ss) = widget.unique_widget_state::<Triangles3d>() {
                        ss.state.render(self);
                    }
                }
            }
        }
    }
}
