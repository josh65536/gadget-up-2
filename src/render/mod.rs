mod camera;
mod gadget;
mod model;
mod shader;
mod texture;
mod ui;

pub use camera::Camera;
pub use gadget::{GadgetRenderInfo, GadgetRenderer, GridItemRenderer, SelectionRenderer};
pub use model::{Model, Triangles, TrianglesEx, Vertex, VertexEx};
pub use model::{ModelType, TrianglesType, MODELS, TRIANGLESES};
pub use shader::{ShaderType, SHADERS};
pub use texture::{TextureType, TEXTURES};
pub use ui::UiRenderer;


use crate::grid::{Grid, GridItem, XY};











/// Takes a grid and renders it. Assumes the grid is on the XY plane,
/// with X in the grid pointing in the X direction
///  and Y in the grid pointing in the Y direction.
/// Also assumes that the camera is looking directly at the grid, with no rotation.
/// A z-index and offset is provided.
/// The background is optionally rendered.
pub fn render_grid<T: GridItem, R>(
    grid: &Grid<T>,
    camera: &Camera,
    r: &mut R,
    xy: XY,
    z: f64,
    render_background: bool,
) where
    R: GridItemRenderer<Item = T>,
{
    let center = camera.position();
    let ortho = camera.get_projection();

    let width = 2.0 / ortho.x.x as f64;
    let height = 2.0 / ortho.y.y as f64;

    let min_x = (center.x - xy.x as f64) - width / 2.0;
    let max_x = (center.x - xy.x as f64) + width / 2.0;
    let min_y = (center.y - xy.y as f64) - height / 2.0;
    let max_y = (center.y - xy.y as f64) + height / 2.0;

    r.begin();

    if render_background {
        for xy in grid.get_empty_in_bounds(min_x, max_x, min_y, max_y) {
            r.render(None, xy, (1, 1), z);
        }
    }

    for (t, xy, wh) in grid.get_in_bounds(min_x, max_x, min_y, max_y) {
        r.render(Some(t), *xy, *wh, z);
    }

    let offset = xy.cast::<f64>().unwrap().extend(0.0);
    let mut camera = camera.clone();
    camera.set_view(
        camera.position() - offset,
        camera.target() - offset,
        *camera.up(),
    );
    r.end(&camera);
}
