use super::{Camera, ShaderMap, ShaderType, TrianglesEx};
use crate::gadget::{Gadget, GadgetRenderInfo};
use crate::grid::{Grid, WH, XY};
use crate::log;
use crate::shape::{Rectangle, Shape};

use golem::Dimension::{D3, D4};
use golem::{Attribute, AttributeType, Uniform, UniformType, UniformValue};
use golem::{Context, ShaderDescription, ShaderProgram};
use golem::{ElementBuffer, GeometryMode, VertexBuffer};
use itertools::izip;
use std::rc::Rc;

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
    pub fn new(gl: &Rc<Context>, shader_map: &ShaderMap) -> Self {
        Self {
            program: Rc::clone(&shader_map[&ShaderType::Offset]),
            gl: Rc::clone(gl),
            triangles: TrianglesEx::default(),
            vertex_buffer: VertexBuffer::new(gl).unwrap(),
            index_buffer: ElementBuffer::new(gl).unwrap(),
        }
    }

    pub fn render_gadget(&mut self, gadget: &Gadget, position: XY, _size: WH) {
        let x = position.x as f32;
        let y = position.y as f32;

        self.triangles.append(gadget.renderer().triangles().clone().with_extra([x, y, 0.0]));
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

            let rect = Rectangle::new(0.0, 1.0, 0.0, 1.0, GadgetRenderInfo::RECTANGLE_Z);
            rect.append_to(&mut self.positions, &mut self.indexes);

            self.offsets
                .extend(&[x, y, 0.0, x, y, 0.0, x, y, 0.0, x, y, 0.0]);
            self.colors.extend(&[
                0.6, 0.8, 1.0, 1.0, 0.7, 0.9, 1.0, 1.0, 0.9, 1.0, 1.0, 1.0, 0.8, 1.0, 1.0, 1.0,
            ]);
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

        self.vertex_buffer.set_data(&self.triangles.iter_vertex_items().collect::<Vec<_>>());
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
