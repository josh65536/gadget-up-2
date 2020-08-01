use conrod_core::builder_methods;
use conrod_core::position::{self, Align, Place};
use conrod_core::text;
use conrod_core::widget::bordered_rectangle;
use conrod_core::widget::{self, BorderedRectangle, Common, CommonBuilder, Text};
use conrod_core::widget_ids;
use conrod_core::Colorable;
use conrod_core::{Color, FontSize, Positionable, Scalar, Sizeable, Widget};
use conrod_derive::WidgetStyle;
use std::ops::{Deref, DerefMut};

use super::triangles3d::Triangles3d;

pub struct Button<'a, S> {
    inner: widget::Button<'a, S>,
    style: Style,
    tooltip_text: Option<&'a str>,
    current: bool,
    enabled: bool,
}

impl<'a, S> Common for Button<'a, S> {
    fn common(&self) -> &CommonBuilder {
        self.inner.common()
    }

    fn common_mut(&mut self) -> &mut CommonBuilder {
        self.inner.common_mut()
    }
}

impl<'a, S> Deref for Button<'a, S> {
    type Target = widget::Button<'a, S>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, S> DerefMut for Button<'a, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
/// Unique styling for the Button.
#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {
    /// Color of the Button's pressable area.
    #[conrod(default = "theme.shape_color")]
    pub color: Option<Color>,
    /// Width of the border surrounding the button
    #[conrod(default = "theme.border_width")]
    pub border: Option<Scalar>,
    /// The color of the border.
    #[conrod(default = "theme.border_color")]
    pub border_color: Option<Color>,
    /// The color of the Button's label.
    #[conrod(default = "theme.label_color")]
    pub label_color: Option<Color>,
    /// The font size of the Button's label.
    #[conrod(default = "theme.font_size_medium")]
    pub label_font_size: Option<FontSize>,
    /// The ID of the font used to display the label.
    #[conrod(default = "theme.font_id")]
    pub label_font_id: Option<Option<text::font::Id>>,
    /// The label's typographic alignment over the *x* axis.
    #[conrod(default = "text::Justify::Center")]
    pub label_justify: Option<text::Justify>,
    /// The position of the title bar's `Label` widget over the *x* axis.
    #[conrod(default = "position::Relative::Align(Align::Middle)")]
    pub label_x: Option<position::Relative>,
    /// The position of the title bar's `Label` widget over the *y* axis.
    #[conrod(default = "position::Relative::Align(Align::Middle)")]
    pub label_y: Option<position::Relative>,
    /// The color of the tooltip rectangle.
    #[conrod(default = "Color::Rgba(0.75, 1.0, 0.5, 1.0)")]
    pub tooltip_rect_color: Option<Color>,
    /// The color of the tooltip rectangle border.
    #[conrod(default = "Color::Rgba(0.0, 0.0, 0.0, 1.0)")]
    pub tooltip_border_color: Option<Color>,
}

widget_ids! {
    pub struct TrianglesIds {
        triangles,
        tooltip_rect,
        tooltip_text,
        select_rect,
        hover_rect,
        disabled,
    }
}

pub struct Triangles {
    triangles: Triangles3d,
    padding: f64,
}

impl<'a, S> Button<'a, S> {
    builder_methods! {
        pub tooltip_rect_color { style.tooltip_rect_color = Some(Color) }
        pub tooltip_border_color { style.tooltip_border_color = Some(Color) }
    }
}

impl<'a> Button<'a, Triangles> {
    pub fn triangles(triangles: Triangles3d) -> Self {
        Self {
            inner: widget::Button::new_internal(Triangles {
                triangles,
                padding: 0.0,
            }),
            style: Style::default(),
            tooltip_text: None,
            current: false,
            enabled: true,
        }
    }

    pub fn padding(mut self, padding: f64) -> Self {
        self.show.padding = padding;
        self
    }

    pub fn tooltip_text(mut self, text: &'a str) -> Self {
        self.tooltip_text = Some(text);
        self
    }

    pub fn current(mut self, current: bool) -> Self {
        self.current = current;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

impl<'a> Widget for Button<'a, Triangles> {
    type State = TrianglesIds;
    type Style = Style;
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
        let widget::Button { show, .. } = self.inner;
        let tooltip_text = self.tooltip_text;
        let Triangles { triangles, padding } = show;

        let (x, y, w, h) = rect.x_y_w_h();

        triangles
            .x_y(x, y)
            .w_h(w - 2.0 * padding, h - 2.0 * padding)
            .parent(id)
            .graphics_for(id)
            .set(state.triangles, ui);

        if let Some(tooltip_text) = tooltip_text {
            if let Some(mouse) = ui.widget_input(id).mouse() {
                if mouse.is_over() {
                    let text = Text::new(tooltip_text).font_size(12);

                    let mut wh = text.get_wh(ui).unwrap_or([10.0, 10.0]);
                    wh[0] += 6.0;
                    wh[1] += 6.0;

                    BorderedRectangle::new(wh)
                        .with_style(bordered_rectangle::Style {
                            color: Some(style.tooltip_rect_color(&ui.theme)),
                            border: None,
                            border_color: Some(style.tooltip_border_color(&ui.theme)),
                        })
                        .x_place_on(id, Place::Start(ui.w_of(id).map(|x| x / 2.0)))
                        .y_place_on(id, Place::End(ui.h_of(id).map(|x| x / 2.0)))
                        .graphics_for(id)
                        .floating(true)
                        //.and_then(ui.widget_graph().depth_parent(id), Widget::parent)
                        //.depth(-1.0)
                        .set(state.tooltip_rect, ui);

                    text.middle_of(state.tooltip_rect)
                        .graphics_for(id)
                        .set(state.tooltip_text, ui);
                }
            }
        }

        if self.current {
            widget::Rectangle::outline_styled(
                [rect.w(), rect.h()],
                widget::line::Style::solid()
                    .thickness(4.0)
                    .color(Color::Rgba(0.5, 0.0, 0.0, 1.0)),
            )
            .middle_of(id)
            .wh_of(id)
            .graphics_for(id)
            .set(state.select_rect, ui);
        }

        if !self.enabled {
            widget::Line::new([rect.left(), rect.bottom()], [rect.right(), rect.top()])
                .thickness(4.0)
                .color(Color::Rgba(0.5, 0.0, 0.0, 1.0))
                .set(state.disabled, ui);
        }

        let (_interaction, times_triggered) =
            widget::button::interaction_and_times_triggered(id, ui);

        widget::button::TimesClicked(if self.enabled { times_triggered } else { 0 })
    }
}
