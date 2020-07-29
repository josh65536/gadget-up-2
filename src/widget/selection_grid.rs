use conrod_core::widget::{self, Widget};
use conrod_core::{builder_method, builder_methods, widget_ids};
use conrod_core::{Borderable, Color, Colorable, Positionable, Sizeable};
use conrod_derive::{WidgetCommon, WidgetStyle};

use super::button::Button;
use super::triangles3d::Triangles3d;
use crate::gadget::Gadget;

type WH = (usize, usize);

widget_ids! {
    pub struct Ids {
        rect, matrix, select_rect,
    }
}

/// A grid for making a selection of a gadget
#[derive(WidgetCommon)]
pub struct SelectionGrid<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    style: Style,
    size: WH,
    gadgets: &'a [Gadget],
    /// Index of selected gadget
    selected: Option<usize>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    #[conrod(default = "theme.border_width")]
    pub border: Option<f64>,
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// Padding on the outside
    #[conrod(default = "0.0")]
    pub outer_padding: Option<f64>,
}

impl<'a> SelectionGrid<'a> {
    pub fn new(width: usize, gadgets: &'a [Gadget], selected: Option<usize>) -> Self {
        Self {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            size: (width, (gadgets.len() + width - 1) / width),
            gadgets,
            selected,
        }
    }

    /// Sets the width of the outer padding,
    /// and sets the height so they look the same.
    pub fn outer_padding(self, pad: f64) -> Self {
        Self {
            style: Style {
                outer_padding: Some(pad),
                ..self.style
            },
            ..self
        }
    }
}

impl<'a> Colorable for SelectionGrid<'a> {
    builder_method! (color { style.color = Some(Color) });
}

impl<'a> Borderable for SelectionGrid<'a> {
    builder_methods! {
        border { style.border = Some(f64) }
        border_color { style.border_color = Some(Color) }
    }
}

impl<'a> Widget for SelectionGrid<'a> {
    type State = Ids;
    type Style = Style;
    type Event = Option<usize>;

    fn init_state(&self, mut id_gen: widget::id::Generator) -> Self::State {
        Ids {
            rect: id_gen.next(),
            matrix: id_gen.next(),
            select_rect: id_gen.next(),
        }
    }

    fn style(&self) -> Self::Style {
        self.style
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

        let Self {
            size: (size_w, size_h),
            gadgets,
            ..
        } = self;

        let color = style.color(&ui.theme);
        let border = style.border(&ui.theme);
        let border_color = style.border_color(&ui.theme);
        let outer_padding = style.outer_padding(&ui.theme);

        let pad_rect = rect.pad(-outer_padding);

        widget::BorderedRectangle::new(pad_rect.dim())
            .middle_of(id)
            .graphics_for(id)
            .color(color)
            .border(border)
            .border_color(border_color)
            .set(state.rect, ui);

        let h_scale = (size_h as f64 / size_w as f64) / (rect.h() / rect.w());

        let mut elements = widget::Matrix::new(size_w, size_h)
            .middle_of(id)
            .w(rect.w())
            .h(rect.h() * h_scale)
            .set(state.matrix, ui);

        let mut event = None;

        while let Some(element) = elements.next(ui) {
            let widget::matrix::Element { row, col, .. } = element;

            let i = row * size_w + col;
            if let Some(gadget) = gadgets.get(i) {
                //let button = widget::Button::image(image_map[name].image_id())
                //    .and_then(image_map[name].source(), widget::Button::source_rectangle);

                //for _ in element.set(button, ui) {
                //    event = Some(name.to_owned());
                //}

                let button = Button::triangles(Triangles3d::from_gadget(gadget))
                    .padding(5.0)
                    .tooltip_text(gadget.name());

                for _ in element.set(button, ui) {
                    event = Some(i);
                }

                if Some(i) == self.selected {
                    let _rect = widget::Rectangle::outline_styled(
                        [element.w, element.h],
                        widget::line::Style::solid()
                            .thickness(4.0)
                            .color(Color::Rgba(0.5, 0.0, 0.0, 1.0)),
                    )
                    .x_y_relative_to(state.matrix, element.rel_x, element.rel_y)
                    .graphics_for(element.widget_id)
                    .set(state.select_rect, ui);
                }
            }
        }

        event
    }
}
