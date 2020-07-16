use crate::grid::{Grid, WH, XY};

use std::rc::Rc;
use three_d::core::Error;
use three_d::gl::Glstruct;
use three_d::{Camera, ElementBuffer, Program, VertexBuffer};

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
        r.render(None, xy, [1, 1]);
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
    program: Program,
    gl: Rc<Glstruct>,
    vertices: Vec<f32>,
    offsets: Vec<f32>,
    colors: Vec<f32>,
    indexes: Vec<u32>,
}

impl GadgetRenderer {
    pub fn new(gl: &Rc<Glstruct>) -> Self {
        let program = Program::from_source(
            gl,
            include_str!("../assets/shaders/color.vert"),
            include_str!("../assets/shaders/color.frag"),
        )
        .unwrap();

        Self {
            program,
            gl: Rc::clone(gl),
            vertices: vec![],
            offsets: vec![],
            colors: vec![],
            indexes: vec![],
        }
    }
}

impl GridItemRenderer for GadgetRenderer {
    type Item = ();

    /// Start the rendering of the grid
    fn begin(&mut self) {
        self.vertices.clear();
        self.offsets.clear();
        self.colors.clear();
        self.indexes.clear();
    }

    /// Render a specific item
    fn render(&mut self, item: Option<&Self::Item>, position: XY, size: WH) {
        let idx = self.vertices.len() as u32 / 3;

        let x = position[0] as f32;
        let y = position[1] as f32;

        self.indexes
            .extend(&[idx + 0, idx + 1, idx + 2, idx + 2, idx + 3, idx + 0]);
        self.offsets.extend(&[x, y, x, y, x, y, x, y]);
        self.vertices
            .extend(&[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0]);
        self.colors
            .extend(&[0.6, 0.8, 1.0, 0.7, 0.9, 1.0, 0.9, 1.0, 1.0, 0.8, 1.0, 1.0]);
    }

    /// Finalize the rendering of the grid
    fn end(&mut self, camera: &Camera) {
        let world_view_projection = camera.get_projection() * camera.get_view();
        self.program
            .add_uniform_mat4("worldViewProjectionMatrix", &world_view_projection)
            .unwrap();

        let positions = VertexBuffer::new_with_static_f32(&self.gl, &self.vertices).unwrap();
        let offsets = VertexBuffer::new_with_static_f32(&self.gl, &self.offsets).unwrap();
        let colors = VertexBuffer::new_with_static_f32(&self.gl, &self.colors).unwrap();
        let elements = ElementBuffer::new_with_u32(&self.gl, &self.indexes).unwrap();

        self.program
            .use_attribute_vec3_float(&positions, "position")
            .unwrap();
        self.program
            .use_attribute_vec2_float(&offsets, "offset")
            .unwrap();
        self.program
            .use_attribute_vec3_float(&colors, "color")
            .unwrap();

        self.program.draw_elements(&elements);
    }
}
