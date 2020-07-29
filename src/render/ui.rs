use cgmath::{vec2, vec3, vec4};
use conrod_core::graph::Node;
use conrod_core::render::{Primitive, PrimitiveKind};
use conrod_core::text::GlyphCache;
use conrod_core::utils;
use conrod_core::{Ui, Widget};
use golem::Dimension::{D3, D4};
use golem::{Attribute, AttributeType, Uniform, UniformType, UniformValue};
use golem::{ColorFormat, Context, ShaderDescription, ShaderProgram};
use golem::{ElementBuffer, GeometryMode, VertexBuffer};
use itertools::izip;
use ref_thread_local::RefThreadLocal;
use std::rc::Rc;

use super::texture::{GLYPH_CACHE_HEIGHT, GLYPH_CACHE_WIDTH};
use super::texture::{GLYPH_CACHE_OFFSET_X, GLYPH_CACHE_OFFSET_Y};
use super::texture::{MAIN_TEXTURE_HEIGHT, MAIN_TEXTURE_WIDTH};
use super::{Camera, TrianglesEx, TrianglesType, VertexEx, TRIANGLESES};
use super::{ShaderType, TextureType, SHADERS, TEXTURES};
use crate::log;
use crate::shape::{Rectangle, Shape};
use crate::widget::triangles3d::Triangles3d;

pub struct UiRenderer<'a> {
    gl: Rc<Context>,
    glyph_cache: GlyphCache<'a>,
    pub camera: Camera,
    program: Rc<ShaderProgram>,
    pub triangles: TrianglesEx<[f32; 5]>,
    vertex_buffer: VertexBuffer,
    index_buffer: ElementBuffer,
    width: f64,
    height: f64,
    floating: bool,
}

impl<'a> UiRenderer<'a> {
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
            glyph_cache: GlyphCache::builder()
                .dimensions(GLYPH_CACHE_WIDTH as u32, GLYPH_CACHE_HEIGHT as u32)
                .build(),
            camera,
            program: Rc::clone(&SHADERS.borrow()[ShaderType::ScaleOffset]),
            triangles: TrianglesEx::default(),
            vertex_buffer: VertexBuffer::new(gl).unwrap(),
            index_buffer: ElementBuffer::new(gl).unwrap(),
            width: 1.0,
            height: 1.0,
            floating: false,
        }
    }

    /// Starts the drawing process
    pub fn draw_begin(&mut self, width: f64, height: f64) {
        self.triangles.clear();
        self.camera.set_orthographic_projection(width, height, 1.0);
        self.width = width;
        self.height = height;
        self.floating = false;
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

        self.program
            .set_uniform("image", UniformValue::Int(1))
            .unwrap();
        TEXTURES.borrow()[TextureType::Main].set_active(std::num::NonZeroU32::new(1).unwrap());

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

    pub fn primitive(&mut self, p: Primitive, ui: &Ui) {
        let Primitive { id, kind, rect, .. } = p;

        let (x, y, w, h) = rect.x_y_w_h();

        let mut z = Self::UI_Z_BASE as f32 + if self.floating { -0.01 } else { 0.0 };

        // Because the model widgets render with different depth,
        // add a hack here for floating widgets.
        //if let Some(Node::Widget(widget)) = ui.widget_graph().node(id) {
        //    log!("This is");
        //    if widget.maybe_floating.is_some() {
        //        log!("a floating widget!");
        //        z -= 0.01;
        //        log!("Kind: {:?}", std::mem::discriminant(&kind));
        //    }
        //}

        match kind {
            PrimitiveKind::Rectangle { color } => {
                let rgba = color.to_rgb();

                self.triangles.append(
                    Rectangle::new(-w / 2.0, w / 2.0, -h / 2.0, h / 2.0, 0.0)
                        .triangles(vec4(rgba.0, rgba.1, rgba.2, rgba.3))
                        .with_extra([1.0, 1.0, x as f32, y as f32, z]),
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
                                    vec3(t[0][0] as f32, t[0][1] as f32, z),
                                    vec3(0.0, 0.0, 0.0),
                                    color,
                                    extra,
                                ),
                                VertexEx::new(
                                    vec3(t[1][0] as f32, t[1][1] as f32, z),
                                    vec3(0.0, 0.0, 0.0),
                                    color,
                                    extra,
                                ),
                                VertexEx::new(
                                    vec3(t[2][0] as f32, t[2][1] as f32, z),
                                    vec3(0.0, 0.0, 0.0),
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
                                    vec3(t[0].0[0] as f32, t[0].0[1] as f32, z),
                                    vec3(0.0, 0.0, 0.0),
                                    vec4(t[0].1 .0, t[0].1 .1, t[0].1 .2, t[0].1 .3),
                                    extra,
                                ),
                                VertexEx::new(
                                    vec3(t[1].0[0] as f32, t[1].0[1] as f32, z),
                                    vec3(0.0, 0.0, 0.0),
                                    vec4(t[0].1 .0, t[0].1 .1, t[0].1 .2, t[0].1 .3),
                                    extra,
                                ),
                                VertexEx::new(
                                    vec3(t[2].0[0] as f32, t[2].0[1] as f32, z),
                                    vec3(0.0, 0.0, 0.0),
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
                color,
                text,
                font_id,
            } => {
                let glyphs = text.positioned_glyphs(1.0);

                for glyph in glyphs.iter() {
                    self.glyph_cache.queue_glyph(font_id.index(), glyph.clone())
                }

                self.glyph_cache
                    .cache_queued(|rect, data| {
                        let data = data
                            .iter()
                            .flat_map(|c| vec![*c; 4].into_iter())
                            .collect::<Vec<_>>();

                        TEXTURES.borrow_mut()[TextureType::Main].set_subimage(
                            &data,
                            rect.min.x + GLYPH_CACHE_OFFSET_X as u32,
                            rect.min.y + GLYPH_CACHE_OFFSET_Y as u32,
                            rect.width(),
                            rect.height(),
                            ColorFormat::RGBA,
                        );
                    })
                    .unwrap();

                const UV_X_MIN: f32 = GLYPH_CACHE_OFFSET_X as f32 / MAIN_TEXTURE_WIDTH as f32;
                const UV_X_MAX: f32 =
                    (GLYPH_CACHE_OFFSET_X + GLYPH_CACHE_WIDTH) as f32 / MAIN_TEXTURE_WIDTH as f32;
                const UV_Y_MIN: f32 = GLYPH_CACHE_OFFSET_Y as f32 / MAIN_TEXTURE_HEIGHT as f32;
                const UV_Y_MAX: f32 =
                    (GLYPH_CACHE_OFFSET_Y + GLYPH_CACHE_HEIGHT) as f32 / MAIN_TEXTURE_HEIGHT as f32;

                let rgba = color.to_rgb();
                let color = vec4(rgba.0, rgba.1, rgba.2, rgba.3);

                let extra = [
                    1.0,
                    -1.0,
                    -self.width as f32 / 2.0,
                    self.height as f32 / 2.0,
                    0.0,
                ];

                for glyph in glyphs {
                    if let Ok(Some((uv_rect, pos))) =
                        self.glyph_cache.rect_for(font_id.index(), glyph)
                    {
                        let tx_min = utils::map_range(uv_rect.min.x, 0.0, 1.0, UV_X_MIN, UV_X_MAX);
                        let tx_max = utils::map_range(uv_rect.max.x, 0.0, 1.0, UV_X_MIN, UV_X_MAX);
                        let ty_min = utils::map_range(uv_rect.min.y, 0.0, 1.0, UV_Y_MIN, UV_Y_MAX);
                        let ty_max = utils::map_range(uv_rect.max.y, 0.0, 1.0, UV_Y_MIN, UV_Y_MAX);

                        self.triangles.append(TrianglesEx::new(
                            vec![
                                VertexEx::new(
                                    vec3(pos.min.x as f32, pos.min.y as f32, z),
                                    vec3(tx_min, ty_min, 1.0),
                                    color,
                                    extra,
                                ),
                                VertexEx::new(
                                    vec3(pos.max.x as f32, pos.min.y as f32, z),
                                    vec3(tx_max, ty_min, 1.0),
                                    color,
                                    extra,
                                ),
                                VertexEx::new(
                                    vec3(pos.max.x as f32, pos.max.y as f32, z),
                                    vec3(tx_max, ty_max, 1.0),
                                    color,
                                    extra,
                                ),
                                VertexEx::new(
                                    vec3(pos.min.x as f32, pos.max.y as f32, z),
                                    vec3(tx_min, ty_max, 1.0),
                                    color,
                                    extra,
                                ),
                            ],
                            vec![0, 1, 2, 2, 3, 0],
                        ));
                    }
                }
            }

            PrimitiveKind::Other(widget) => {
                // Floating widgets render after normal ones.
                // Hack here to move them closer to the camera because of 3D models in the UI
                if widget.maybe_floating.is_some() {
                    self.floating = true;
                }

                if widget.type_id == std::any::TypeId::of::<<Triangles3d as Widget>::State>() {
                    if let Some(ss) = widget.unique_widget_state::<Triangles3d>() {
                        ss.state.render(self);
                    }
                }
            }
        }
    }
}
