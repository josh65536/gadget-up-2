use graphics::types::Color;
use graphics::{DrawState, Graphics, ImageSize};
use three_d::Camera;

pub struct GraphicsEx {
    /// Only for 3d
    camera: Camera,
    /// 3 coordinates per vertex
    vertices: Vec<f32>,
    /// 3 components per color
    colors: Vec<f32>,
    /// 3 indexes per triangle
    indexes: Vec<u32>,
}

impl Graphics for GraphicsEx {
    type Texture = DummyTexture;

    fn clear_color(&mut self, color: Color) {
        unimplemented!();
    }

    fn clear_stencil(&mut self, value: u8) {
        unimplemented!();
    }

    fn tri_list<F>(&mut self, draw_state: &DrawState, color: &[f32; 4], f: F)
    where
        F: FnMut(&mut dyn FnMut(&[[f32; 2]])),
    {
    }

    fn tri_list_uv<F>(
        &mut self,
        draw_state: &DrawState,
        color: &[f32; 4],
        texture: &Self::Texture,
        f: F,
    ) where
        F: FnMut(&mut dyn FnMut(&[[f32; 2]], &[[f32; 2]])),
    {
    }
}

pub struct DummyTexture;

impl ImageSize for DummyTexture {
    fn get_size(&self) -> (u32, u32) {
        (1, 1)
    }
}
