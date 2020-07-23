use cgmath::vec2;
use conrod_core::widget;
use conrod_core::Widget;
use conrod_derive::WidgetCommon;

use crate::log;
use crate::gadget::Gadget;
use crate::math::Vec2;
use crate::render::{TrianglesEx, UiRenderer};
use crate::shape::Shape;

/// Triangles, but in 3D
#[derive(Debug, WidgetCommon)]
pub struct Triangles3d {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    /// Extra attributes: scale (vec2), offset (vec3)
    triangles: TrianglesEx<[f32; 5]>,
    // These get mapped onto the bounding rectangle.
    src_center: Vec2,
    src_width: f64,
    src_height: f64,
}

impl Triangles3d {
    pub fn new(
        triangles: TrianglesEx<[f32; 5]>,
        src_center: Vec2,
        src_width: f64,
        src_height: f64,
    ) -> Self {
        Self {
            common: widget::CommonBuilder::default(),
            triangles,
            src_center,
            src_width,
            src_height,
        }
    }

    pub fn from_gadget(gadget: &Gadget) -> Self {
        let width = gadget.size().0 as f64;
        let height = gadget.size().1 as f64;

        Self {
            common: widget::CommonBuilder::default(),
            triangles: gadget.renderer().triangles().clone().with_default_extra(),
            src_center: vec2(width / 2.0, height / 2.0),
            src_width: width,
            src_height: height,
        }
    }
}

pub struct State {
    triangles: TrianglesEx<[f32; 5]>,
}

impl State {
    fn new() -> Self {
        Self {
            triangles: TrianglesEx::default(),
        }
    }

    pub fn render(&self, g: &mut UiRenderer) {
        g.triangles.append(self.triangles.clone());
    }
}

impl Widget for Triangles3d {
    type State = State;
    type Style = ();
    type Event = ();

    fn init_state(&self, _id: widget::id::Generator) -> Self::State {
        State::new()
    }

    fn style(&self) -> Self::Style {
        ()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, rect, .. } = args;

        let Self {
            mut triangles,
            src_center,
            src_width,
            src_height,
            ..
        } = self;

        let (x, y, w, h) = rect.x_y_w_h();
        let scale = (w / src_width).min(h / src_height);
        let offset = vec2(x, y) - src_center * scale;

        for v in triangles.vertices_mut() {
            v.extra = [
                scale as f32,
                scale as f32,
                offset.x as f32,
                offset.y as f32,
                UiRenderer::UI_Z_BASE as f32,
            ];
        }

        state.update(|state| {
            state.triangles = triangles;
        })
    }
}
