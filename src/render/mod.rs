mod camera;
mod gadget;
mod model;
mod shader;
mod ui;

pub use camera::Camera;
pub use gadget::{GadgetRenderer, GridItemRenderer};
pub use model::{Model, Triangles, Vertex, VertexEx, TrianglesEx};
pub use model::{ModelMap, ModelType, model_map, TrianglesMap, TrianglesType, triangles_map};
pub use shader::{shader_map, ShaderMap, ShaderType};
pub use ui::UiRenderer;

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
