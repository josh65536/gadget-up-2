use crate::camera::Camera;
use crate::gadget::{Gadget, GadgetRenderInfo};
use crate::grid::{Grid, WH, XY};
use crate::log;
use crate::math::ToArray;
use crate::shape::{Rectangle, Shape};

use golem::Dimension::{D2, D3, D4};
use golem::{Attribute, AttributeType, Uniform, UniformType, UniformValue};
use golem::{Context, ShaderDescription, ShaderProgram};
use golem::{ElementBuffer, GeometryMode, VertexBuffer};
use itertools::izip;
use std::rc::Rc;

/// Takes a grid and renders it. Assumes the grid is on the XY plane,
/// with X in the grid pointing in the X direction
///  and Y in the grid pointing in the Y direction.
/// Also assumes that the camera is looking directly at the grid, with no rotation.
pub fn render_grid<T, R>(grid: &Grid<T>, camera: &Camera, r: &mut R)
where
    R: GridItemRenderer<Item = T>,
{
    let center = camera.position();
    let ortho = camera.get_projection();

    let width = 2.0 / ortho.x.x as f64;
    let height = 2.0 / ortho.y.y as f64;

    let min_x = center.x as f64 - width / 2.0;
    let max_x = center.x as f64 + width / 2.0;
    let min_y = center.y as f64 - height / 2.0;
    let max_y = center.y as f64 + height / 2.0;

    r.begin();

    for xy in grid.get_empty_in_bounds(min_x, max_x, min_y, max_y) {
        r.render(None, xy, (1, 1));
    }

    for (t, xy, wh) in grid.get_in_bounds(min_x, max_x, min_y, max_y) {
        r.render(Some(t), *xy, *wh);
    }

    r.end(camera);
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
    program: ShaderProgram,
    gl: Rc<Context>,
    positions: Vec<f32>,
    offsets: Vec<f32>,
    colors: Vec<f32>,
    indexes: Vec<u32>,
    vertex_buffer: VertexBuffer,
    index_buffer: ElementBuffer,
}

impl GadgetRenderer {
    pub fn new(gl: &Rc<Context>) -> Self {
        let program = ShaderProgram::new(
            gl,
            ShaderDescription {
                vertex_input: &[
                    Attribute::new("v_position", AttributeType::Vector(D3)),
                    Attribute::new("v_offset", AttributeType::Vector(D3)),
                    Attribute::new("v_color", AttributeType::Vector(D4)),
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
        )
        .unwrap();

        Self {
            program,
            gl: Rc::clone(gl),
            positions: vec![],
            offsets: vec![],
            colors: vec![],
            indexes: vec![],
            vertex_buffer: VertexBuffer::new(gl).unwrap(),
            index_buffer: ElementBuffer::new(gl).unwrap(),
        }
    }

    pub fn render_gadget(&mut self, gadget: &Gadget, position: XY, size: WH) {
        let x = position.x as f32;
        let y = position.y as f32;

        let renderer = gadget.renderer();

        renderer.append_to(&mut self.positions, &mut self.indexes);
        self.colors.extend(renderer.colors());
        self.offsets
            .extend([x, y, 0.0].iter().cycle().take(renderer.num_vertices() * 3));
    }
}

impl GridItemRenderer for GadgetRenderer {
    type Item = Gadget;

    /// Start the rendering of the grid
    fn begin(&mut self) {
        self.positions.clear();
        self.offsets.clear();
        self.colors.clear();
        self.indexes.clear();
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
                UniformValue::Matrix4(world_view_projection.cast::<f32>().unwrap().to_array()),
            )
            .unwrap();

        let vertices = izip!(
            self.positions.chunks(3),
            self.offsets.chunks(3),
            self.colors.chunks(4)
        )
        .flat_map(|(p, o, c)| p.iter().chain(o.iter()).chain(c.iter()))
        .copied()
        .collect::<Vec<_>>();

        self.vertex_buffer.set_data(&vertices);
        self.index_buffer.set_data(&self.indexes);

        unsafe {
            self.program.draw(
                &self.vertex_buffer,
                &self.index_buffer,
                0..self.indexes.len(),
                GeometryMode::Triangles,
            );
        }
    }
}
