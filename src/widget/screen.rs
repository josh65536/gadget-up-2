use cgmath::{vec2, vec1};
use cgmath::prelude::*;
use conrod_core::image;
use conrod_core::input::widget::Mouse;
use conrod_core::widget::{self, Widget};
use conrod_core::Point;
use conrod_core::{utils, widget_ids};
use conrod_core::{Positionable, Sizeable};
use conrod_derive::{WidgetCommon, WidgetStyle};
use three_d::{vec4, Camera, Vec2};

use crate::grid::XY;
use crate::{bitfield, log};
use crate::ui::Mode;

widget_ids! {
    pub struct Ids {
        cursor,
    }
}

/// Helper widget for handling inputs in the contraption screen
#[derive(WidgetCommon)]
pub struct ContraptionScreen<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    /// Size in tiles
    camera: &'a Camera,
    style: Style,
    mode: Mode,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {}

bitfield! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Input(u32) {
        left, is_left, set_left: 0,
        right, is_right, set_right: 1,
    }
}

pub struct State {
    // Storing custom input because sometimes it will be nice
    // to fake release events
    input: Input,
    prev_input: Input,
    pressed: Input,
    released: Input,
    input_raw: Input,
    prev_input_raw: Input,
    pressed_raw: Input,
    position: Point,
    ids: Ids,
    /// To make sure we don't send a million tile paint events of the same value
    last_tile_event: Option<Event>,
}

impl<'a> ContraptionScreen<'a> {
    /// Constructs a new StageWidget with dimensions [width, height]
    /// and a selection cursor image
    pub fn new(mode: Mode, camera: &'a Camera) -> Self {
        Self {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            camera,
            mode,
        }
    }

    pub fn handle_input(
        &self,
        state: &mut State,
        mouse: Option<&Mouse>,
        camera: &Camera,
        w: f64,
        h: f64,
    ) {
        state.prev_input_raw = state.input_raw;

        if let Some(mouse) = mouse {
            state.input_raw.set_left(mouse.buttons.left().is_down());
            state.input_raw.set_right(mouse.buttons.right().is_down());

            let [mut x, mut y] = mouse.rel_xy();
            x /= w * 0.5;
            y /= h * 0.5;
            let position = camera.position()
                + camera.view_offset_at_screen(vec4(x as f32, y as f32, 0.0, 1.0));
            state.position = [position.x as f64, position.y as f64];
        } else {
            state.input_raw.set_left(false);
            state.input_raw.set_right(false);
        }

        state.pressed_raw = state.input_raw | state.prev_input_raw;
        //state.released_raw = state.pressed_raw ^ state.input_raw;
        state.pressed_raw ^= state.prev_input_raw;

        // Now for potentially filtered input

        state.prev_input = state.input;

        // The left button should release the right button when pressed and vice versa

        if state.pressed_raw.is_left() {
            state.input.set_left(true);
            state.input.set_right(false);
        } else if !state.input_raw.is_left() {
            state.input.set_left(false);
        }

        if state.pressed_raw.is_right() {
            state.input.set_right(true);
            state.input.set_left(false);
        } else if !state.input_raw.is_right() {
            state.input.set_right(false);
        }

        state.pressed = state.input | state.prev_input;
        state.released = state.pressed ^ state.input;
        state.pressed ^= state.prev_input;
    }

    fn update_paint_tile(self, args: widget::UpdateArgs<Self>) -> <Self as Widget>::Event {
        let id = args.id;
        let state = args.state;
        let rect = args.rect;
        let ui = args.ui;

        let Self { camera, .. } = self;

        let mut events = vec![];

        state.update(|state| {
            if state.pressed.is_left() {
                state.last_tile_event = None;
            }

            if let Some(mouse) = ui.widget_input(id).mouse() {
                if mouse.is_over() {
                    let (w, h) = rect.w_h();

                    let x = state.position[0].floor() as i32;
                    let y = state.position[1].floor() as i32;

                    if state.input.is_left() {
                        let event = Event::TilePaint(vec2(x, y));
                        if state.last_tile_event != Some(event.clone()) {
                            state.last_tile_event = Some(event.clone());
                            events.push(event);
                        }
                    }

                    events.push(Event::TileHover(vec2(x, y)));
                }
            }
        });

        events
    }

    fn update_place_agent(self, args: widget::UpdateArgs<Self>) -> <Self as Widget>::Event {
        let id = args.id;
        let state = args.state;
        let rect = args.rect;
        let ui = args.ui;

        let Self { camera, .. } = self;

        let mut events = vec![];

        state.update(|state| {
            if let Some(mouse) = ui.widget_input(id).mouse() {
                if mouse.is_over() {
                    let (w, h) = rect.w_h();

                    let mut x_off: f32 = state.position[0] as f32 * 2.0;
                    let mut y_off: f32 = state.position[1] as f32 * 2.0;

                    let mut x = x_off.round() / 2.0;
                    let mut y = y_off.round() / 2.0;
                    
                    x_off = x_off - x_off.round();
                    y_off = y_off - y_off.round();

                    if x_off.abs() > y_off.abs() {
                        x += vec1(x_off).normalize_to(0.5).x;
                    } else {
                        y += vec1(y_off).normalize_to(0.5).x;
                    }

                    if state.pressed.is_left() {
                        events.push(Event::AgentPlace(vec2(x, y)));
                    } else {
                        events.push(Event::AgentHover(vec2(x, y)));
                    }
                }
            }
        });

        events
    }
}

#[derive(Clone, PartialEq)]
pub enum Event {
    /// Tile at (X, Y) is painted
    TilePaint(XY),
    /// Mouse moved over (X, Y) in tile paint mode
    TileHover(XY),
    /// Tile at (X, Y) is erased
    TileErase(XY),
    /// Agent is placed at (X, Y)
    AgentPlace(Vec2),
    /// Mouse moved over (X, Y) in agent place mode
    AgentHover(Vec2),
}

impl<'a> Widget for ContraptionScreen<'a> {
    type State = State;
    type Style = Style;
    type Event = Vec<Event>;

    fn init_state(&self, id_gen: widget::id::Generator) -> Self::State {
        State {
            input: Input::zero(),
            prev_input: Input::zero(),
            pressed: Input::zero(),
            released: Input::zero(),
            input_raw: Input::zero(),
            prev_input_raw: Input::zero(),
            pressed_raw: Input::zero(),
            position: [0.0, 0.0],
            ids: Ids::new(id_gen),
            last_tile_event: None,
        }
    }

    fn style(&self) -> Self::Style {
        self.style
    }

    fn update(self, args: widget::UpdateArgs<Self>) -> Self::Event {
        let widget_input = args.ui.widget_input(args.id);
        let rect = args.rect;

        args.state.update(|state| {
            self.handle_input(
                state,
                widget_input.mouse().as_ref(),
                self.camera,
                rect.w(),
                rect.h(),
            )
        });

        match self.mode {
            Mode::TilePaint => self.update_paint_tile(args),
            Mode::AgentPlace => self.update_place_agent(args),
            _ => vec![]
        }
    }
}
