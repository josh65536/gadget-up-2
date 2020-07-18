use conrod_core::widget::{self, Common, CommonBuilder};
use conrod_core::widget_ids;
use conrod_core::{Positionable, Sizeable, Widget};
use std::ops::{Deref, DerefMut};

use super::triangles3d::Triangles3d;

pub struct Button<'a, S>(widget::Button<'a, S>);

impl<'a, S> Common for Button<'a, S> {
    fn common(&self) -> &CommonBuilder {
        self.0.common()
    }

    fn common_mut(&mut self) -> &mut CommonBuilder {
        self.0.common_mut()
    }
}

impl<'a, S> Deref for Button<'a, S> {
    type Target = widget::Button<'a, S>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, S> DerefMut for Button<'a, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

widget_ids! {
    pub struct TrianglesIds {
        triangles,
    }
}

pub struct Triangles {
    triangles: Triangles3d,
    padding: f64,
}

impl<'a> Button<'a, Triangles> {
    pub fn triangles(triangles: Triangles3d) -> Self {
        Self(widget::Button::new_internal(Triangles {
            triangles,
            padding: 0.0,
        }))
    }

    pub fn padding(mut self, padding: f64) -> Self {
        self.show.padding = padding;
        self
    }
}

impl<'a> Widget for Button<'a, Triangles> {
    type State = TrianglesIds;
    type Style = widget::button::Style;
    type Event = widget::button::TimesClicked;

    fn init_state(&self, id: widget::id::Generator) -> Self::State {
        TrianglesIds::new(id)
    }

    fn style(&self) -> Self::Style {
        self.style.clone()
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget::UpdateArgs {
            id,
            state,
            style,
            rect,
            ui,
            ..
        } = args;
        let widget::Button { show, .. } = self.0;
        let Triangles { triangles, padding } = show;

        let (x, y, w, h) = rect.x_y_w_h();

        triangles
            .x_y(x, y)
            .w_h(w - 2.0 * padding, h - 2.0 * padding)
            .parent(id)
            .graphics_for(id)
            .set(state.triangles, ui);

        let (interaction, times_triggered) =
            widget::button::interaction_and_times_triggered(id, ui);

        widget::button::TimesClicked(times_triggered)
    }
}
