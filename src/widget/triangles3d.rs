use conrod_core::widget;
use conrod_core::Widget;
use conrod_derive::WidgetCommon;
use cgmath::vec2;

use crate::math::Vec2;
use crate::gadget::Gadget;
use crate::graphics_ex::GraphicsEx;
use crate::log;
use crate::shape::Shape;

/// Triangles, but in 3D
#[derive(Debug, WidgetCommon)]
pub struct Triangles3d {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    positions: Vec<f32>,
    colors: Vec<f32>,
    indexes: Vec<u32>,
    // These get mapped onto the bounding rectangle.
    src_center: Vec2,
    src_width: f64,
    src_height: f64,
}

impl Triangles3d {

    pub fn new(positions: Vec<f32>, colors: Vec<f32>, indexes: Vec<u32>, src_center: Vec2, src_width: f64, src_height: f64) -> Self {
        Self {
            common: widget::CommonBuilder::default(),
            positions,
            colors,
            indexes,
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
            positions: gadget.renderer().positions(),
            colors: gadget.renderer().colors().clone(),
            indexes: gadget.renderer().indexes(),
            src_center: vec2(width / 2.0, height / 2.0),
            src_width: width,
            src_height: height,
        }
    }
}

pub struct State {
    positions: Vec<f32>,
    colors: Vec<f32>,
    indexes: Vec<u32>,
    offset: Vec2,
}

impl State {
    fn new() -> Self {
        Self {
            positions: vec![],
            colors: vec![],
            indexes: vec![],
            offset: vec2(0.0, 0.0),
        }
    }

    pub fn render(&self, g: &mut GraphicsEx) {
        let old_len = g.positions.len() as u32 / 3;

        g.positions.extend(self.positions.iter());
        g.offsets.extend(
            [self.offset.x as f32, self.offset.y as f32, GraphicsEx::UI_Z_BASE as f32]
                .iter()
                .cycle()
                .take(self.positions.len()),
        );
        g.colors.extend(self.colors.iter());
        g.indexes.extend(self.indexes.iter().map(|i| *i + old_len));
    }
}

impl Widget for Triangles3d {
    type State = State;
    type Style = ();
    type Event = ();

    fn init_state(&self, id: widget::id::Generator) -> Self::State {
        State::new()
    }

    fn style(&self) -> Self::Style {
        ()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs { state, rect, .. } = args;

        let Self {
            positions,
            colors,
            indexes,
            src_center,
            src_width,
            src_height,
            ..
        } = self;

        let (x, y, w, h) = rect.x_y_w_h();
        let scale = (w / src_width).min(h / src_height);
        let offset = vec2(x, y) - src_center * scale;

        state.update(|state| {
            state.positions = positions
                .into_iter()
                .zip([true, true, false].iter().cycle())
                .map(|(v, change)| if *change { v * scale as f32 } else { v })
                .collect();
            state.colors = colors;
            state.indexes = indexes;
            state.offset = offset;
        })
    }
}
