use cgmath::prelude::*;
use cgmath::{vec2, vec4, Point3, Vector2};

use conrod_core::event::{Event as ConrodEvent, Input as ConrodInput, Motion as MotionEvent, Ui};
use conrod_core::input::widget::Mouse;
use conrod_core::input::{ModifierKey, Motion};
use conrod_core::widget::bordered_rectangle;
use conrod_core::widget::{self, BorderedRectangle, Widget};
use conrod_core::widget_ids;
use conrod_core::{color, Point, Rect};
use conrod_core::{Positionable, Sizeable};
use conrod_derive::{WidgetCommon, WidgetStyle};

use crate::bitfield;
use crate::grid::XY;
use crate::log;
use crate::math::Vec2;
use crate::render::Camera;
use crate::ui::{LeftMouseAction, Mode};

widget_ids! {
    pub struct Ids {
        selection_rect
    }
}

/// Helper widget for handling inputs in the contraption screen
#[derive(WidgetCommon)]
pub struct ContraptionScreen<'a> {
    #[conrod(common_builder)]
    common: widget::CommonBuilder,
    /// Camera that was used to render grid
    camera: &'a Camera,
    style: Style,
    mode: Mode,
    left_mouse_action: LeftMouseAction,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, WidgetStyle)]
pub struct Style {}

bitfield! {
    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub struct Input(u32) {
        left, is_left, set_left: 0,
        middle, is_middle, set_middle: 1,
        right, is_right, set_right: 2,
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
    position: Vec2,
    prev_position: Vec2,
    position_raw: Point,
    prev_position_raw: Point,
    ids: Ids,
    /// To make sure we don't send a million tile paint events of the same value
    last_tile_event: Option<Event>,
    /// Selection start in real coordinates
    selection_start: Vec2,
}

impl<'a> ContraptionScreen<'a> {
    /// Constructs a new StageWidget with dimensions [width, height]
    /// and a selection cursor image
    pub fn new(mode: Mode, left_mouse_action: LeftMouseAction, camera: &'a Camera) -> Self {
        Self {
            common: widget::CommonBuilder::default(),
            style: Style::default(),
            camera,
            mode,
            left_mouse_action,
        }
    }

    fn screen_to_world(mut position: Point, camera: &Camera, w: f64, h: f64) -> Vec2 {
        position[0] /= w * 0.5;
        position[1] /= h * 0.5;
        let position = camera.position()
            + camera.view_offset_at_screen(vec4(position[0], position[1], 0.0, 1.0));
        vec2(position.x, position.y)
    }

    fn world_to_screen(position: Vec2, camera: &Camera, w: f64, h: f64) -> Point {
        let mut position = camera.get_projection().transform_point(
            camera
                .get_view()
                .transform_point(Point3::new(position.x, position.y, 0.0)),
        );
        position.x *= w * 0.5;
        position.y *= h * 0.5;
        [position.x, position.y]
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
        state.prev_position_raw = state.position_raw;
        state.prev_position = state.position;

        if let Some(mouse) = mouse {
            state.input_raw.set_left(mouse.buttons.left().is_down());
            state.input_raw.set_middle(mouse.buttons.middle().is_down());
            state.input_raw.set_right(mouse.buttons.right().is_down());

            state.position_raw = mouse.rel_xy();
            state.position = Self::screen_to_world(state.position_raw, camera, w, h);
        } else {
            state.input_raw.set_left(false);
            state.input_raw.set_middle(false);
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

        state.input.set_middle(state.input_raw.is_middle());

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

        let Self { camera: _, .. } = self;

        let mut events = vec![];

        state.update(|state| {
            if state.pressed.is_left() {
                state.last_tile_event = None;
            }

            if let Some(mouse) = ui.widget_input(id).mouse() {
                if mouse.is_over() {
                    let (_w, _h) = rect.w_h();

                    let x = state.position[0].floor() as isize;
                    let y = state.position[1].floor() as isize;

                    if state.input.is_left() {
                        let event = Event::TilePaint(vec2(x, y));
                        if state.last_tile_event != Some(event.clone()) {
                            state.last_tile_event = Some(event.clone());
                            events.push(event);
                        }
                    }
                }
            }

            if state.released.is_left() {
                events.push(Event::TilePaintFinish);
            }
        });

        events
    }

    fn update_place_agent(self, args: widget::UpdateArgs<Self>) -> <Self as Widget>::Event {
        let id = args.id;
        let state = args.state;
        let rect = args.rect;
        let ui = args.ui;

        let Self { camera: _, .. } = self;

        let mut events = vec![];

        state.update(|state| {
            if let Some(mouse) = ui.widget_input(id).mouse() {
                if mouse.is_over() {
                    let (_w, _h) = rect.w_h();

                    let Vec2 { mut x, y } = state.position;
                    x -= 0.5;

                    let mut xx = x - y;
                    let mut yy = x + y;

                    xx = xx.round();
                    yy = yy.round();

                    let x = (xx + yy) * 0.5 + 0.5;
                    let y = (-xx + yy) * 0.5;

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

    fn update_select(self, args: widget::UpdateArgs<Self>) -> <Self as Widget>::Event {
        let id = args.id;
        let state = args.state;
        let rect = args.rect;
        let ui = args.ui;

        let Self { camera, .. } = self;

        let mut events = vec![];

        state.update(|state| {
            if let Some(mouse) = ui.widget_input(id).mouse() {
                if state.pressed.is_left() {
                    state.selection_start = state.position;
                    events.push(Event::SelectStart(state.selection_start));
                }

                if state.input.is_left() {
                    let corner_0 =
                        Self::world_to_screen(state.selection_start, camera, rect.w(), rect.h());
                    let corner_1 =
                        Self::world_to_screen(state.position, camera, rect.w(), rect.h());
                    let selection_rect = Rect::from_corners(corner_0, corner_1);

                    BorderedRectangle::new([selection_rect.w(), selection_rect.h()])
                        .with_style(bordered_rectangle::Style {
                            color: Some(color::TRANSPARENT),
                            border: None,
                            border_color: Some(color::BLACK),
                        })
                        .xy(selection_rect.xy())
                        .graphics_for(id)
                        .set(state.ids.selection_rect, ui);
                }

                if state.released.is_left() {
                    let modifiers = ui.global_input().current.modifiers;

                    events.push(Event::Select(
                        Rect::from_corners(state.selection_start.into(), state.position.into()),
                        match modifiers {
                            ModifierKey::SHIFT => SelectFunc::Add,
                            ModifierKey::CTRL => SelectFunc::Xor,
                            ModifierKey::ALT => SelectFunc::Subtract,
                            _ => SelectFunc::Replace,
                        },
                    ));
                }
            }
        });

        events
    }

    fn update_gadget_move(self, args: widget::UpdateArgs<Self>) -> <Self as Widget>::Event {
        let id = args.id;
        let state = args.state;
        let rect = args.rect;
        let ui = args.ui;

        let Self { camera: _, .. } = self;

        let mut events = vec![];

        if state.released.is_left() {
            events.push(Event::GadgetMoveFinish);
        }

        events
    }

    fn update_gadget_paste(self, args: widget::UpdateArgs<Self>) -> <Self as Widget>::Event {
        let id = args.id;
        let state = args.state;
        let rect = args.rect;
        let ui = args.ui;

        let Self { camera: _, .. } = self;

        let mut events = vec![];

        state.update(|state| {
            if let Some(mouse) = ui.widget_input(id).mouse() {
                if mouse.is_over() {
                    let (_w, _h) = rect.w_h();

                    let x = state.position[0].floor() as isize;
                    let y = state.position[1].floor() as isize;

                    if state.pressed.is_left() {
                        events.push(Event::GadgetPaste(vec2(x, y)));
                    }
                }
            }
        });

        events
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SelectFunc {
    /// Replace the current selection with the new one
    Replace,
    /// Add the new selection to the current selection
    Add,
    /// Remove the new selection from the current selection
    Subtract,
    /// Xor the new selection with the current selection
    Xor,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// Tile at (X, Y) is painted
    TilePaint(XY),
    /// Tile painting is done
    TilePaintFinish,
    /// Agent is placed at (X, Y)
    AgentPlace(Vec2),
    /// Mouse moved over (X, Y) in agent place mode
    AgentHover(Vec2),
    /// Screen panned by a difference of (X, Y)
    Pan(Vec2),
    /// Screen zoomed at (X, Y) by some amount
    Zoom(Vec2, f64),
    /// Attempted to start a selection at (X, Y)
    SelectStart(Vec2),
    /// Rectangle selection made
    Select(Rect, SelectFunc),
    /// Finished moving gadgets
    GadgetMoveFinish,
    /// Pasted copied selection
    GadgetPaste(XY),
    /// Communicates the position of the mouse in the grid
    MousePosition(Vec2),
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
            position: vec2(0.0, 0.0),
            prev_position: vec2(0.0, 0.0),
            position_raw: [0.0, 0.0],
            prev_position_raw: [0.0, 0.0],
            ids: Ids::new(id_gen),
            last_tile_event: None,
            selection_start: vec2(0.0, 0.0),
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

        let mut vec = vec![Event::MousePosition(args.state.position)];

        for event in args.ui.global_input().events() {
            if let ConrodEvent::Raw(ConrodInput::Motion(Motion::Scroll { x, y })) = event {
                vec.push(Event::Zoom(args.state.position, -*y / 64.0));
                break;
            }

            if let ConrodEvent::Raw(ConrodInput::Motion(Motion::MouseCursor { .. })) = event {
                if args.state.input.is_left()
                    && self.left_mouse_action == LeftMouseAction::Zoom
                    && args.ui.widget_input(args.id).mouse().is_some()
                {
                    vec.push(Event::Zoom(
                        Self::screen_to_world([0.0, 0.0], self.camera, rect.w(), rect.h()),
                        (args.state.prev_position_raw[1] - args.state.position_raw[1]) / 16.0,
                    ));
                    break;
                }
            }
        }

        if (args.state.input.is_middle()
            || (self.left_mouse_action == LeftMouseAction::Pan
                && args.state.input.is_left()
                && args.ui.widget_input(args.id).mouse().is_some()))
            && args.state.position != args.state.prev_position
        {
            vec.push(Event::Pan(
                vec2(args.state.prev_position[0], args.state.prev_position[1])
                    - vec2(args.state.position[0], args.state.position[1]),
            ));
            args.state.update(|state| {
                state.position = state.prev_position;
            })
        }

        vec.append(&mut match self.mode {
            Mode::TilePaint => self.update_paint_tile(args),
            Mode::AgentPlace => self.update_place_agent(args),
            Mode::Select => self.update_select(args),
            Mode::GadgetMove => self.update_gadget_move(args),
            Mode::GadgetPaste => self.update_gadget_paste(args),
            _ => vec![],
        });

        vec
    }
}
