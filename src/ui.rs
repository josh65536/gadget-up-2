use conrod_core::render::PrimitiveWalker;
use conrod_core::widget;
use conrod_core::widget_ids;
use conrod_core::{Color, Positionable, Widget, Sizeable};
use conrod_core::position::{Relative, Align};
use conrod_core::{Ui, UiBuilder};

use crate::log;
use crate::App;

widget_ids! {
    pub struct WidgetIds {
        rect,
    }
}

impl App {
    pub fn update_ui(&mut self) {
        let mut ui = self.ui.set_widgets();

        // Gadget selector
        widget::Rectangle::fill_with([100.0, 100.0], Color::Rgba(0.9, 0.9, 0.9, 1.0))
            .x_position_relative_to(ui.window, Relative::Align(Align::Start))
            .y(0.0)
            .w(300.0)
            .padded_h_of(ui.window, 10.0)
            .set(self.ids.rect, &mut ui);
    }

    pub fn render_ui(&mut self, width: f32, height: f32) {
        self.ui_renderer.draw_begin(width, height);

        let mut primitives = self.ui.draw();
        while let Some(primitive) = PrimitiveWalker::next_primitive(&mut primitives) {
            self.ui_renderer.primitive(primitive);
        }

        self.ui_renderer.draw_end();
    }
}
