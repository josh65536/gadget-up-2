extern crate bitvec;
extern crate cgmath;
extern crate conrod_core;
extern crate conrod_derive;
extern crate conrod_winit;
extern crate fnv;
extern crate glow;
extern crate itertools;
extern crate percent_encoding;
extern crate ref_thread_local;
extern crate ron;
extern crate serde;
extern crate winit;

mod bit_serde;
mod bitfield;
mod gadget;
mod grid;
mod math;
mod preset_gadgets;
mod render;
mod shape;
mod static_map;
mod ui;
mod widget;

use cgmath::{vec2, vec3};
use conrod_core::text::{font, Font};
use conrod_core::{Ui, UiBuilder};
use fnv::FnvHashSet;
use golem::blend::{BlendChannel, BlendEquation, BlendFactor, BlendFunction};
use golem::blend::{BlendInput, BlendMode, BlendOperation};
use golem::depth::{DepthTestFunction, DepthTestMode};
use golem::Context;
use golem::ShaderProgram;
use percent_encoding::{percent_decode_str, utf8_percent_encode, AsciiSet};
use ref_thread_local::RefThreadLocal;
use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalSize};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event::{ModifiersState, MouseScrollDelta};
use winit::event_loop::{ControlFlow, EventLoop};
#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowExtWebSys;
use winit::window::WindowBuilder;

use gadget::{Agent, Gadget, GadgetDef, State};
use grid::Grid;
use math::Vec2;
use render::{Camera, GadgetRenderer, Model, SelectionRenderer, UiRenderer};
use render::{ModelType, ShaderType, TrianglesType, MODELS, SHADERS, TRIANGLESES};
use render::{TextureType, TEXTURES};
use ui::{LeftMouseAction, Mode, WidgetIds};

#[macro_export]
macro_rules! log {
    ( $($t:tt)* ) => {
        // To get rid of the unnecessary rust-analyzer error
        unsafe {
            web_sys::console::log_1(&format!( $($t)* ).into());
        }
    };
}

#[macro_export]
macro_rules! elog {
    ( $($t:tt)* ) => {
        // To get rid of the unnecessary rust-analyzer error
        unsafe {
            web_sys::console::error_1(&format!( $($t)* ).into());
        }
    };
}

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub struct Fonts {
    regular: font::Id,
    italic: font::Id,
    bold: font::Id,
    bold_italic: font::Id,
}

/// An undoable action.
/// Stores the information needed to undo the action.
pub enum UndoAction {
    GadgetInsert { position: grid::XY },
    GadgetRemove { gadget: Gadget, position: grid::XY },
    AgentMove { position: Vec2, direction: grid::XY },
    GadgetChangeState { position: grid::XY, state: State },
    Batch(Vec<UndoAction>),
}

// To allow std::mem::take to work
impl Default for UndoAction {
    fn default() -> Self {
        UndoAction::Batch(vec![])
    }
}

pub struct UndoStack {
    undo: Vec<UndoAction>,
    redo: Vec<UndoAction>,
}

/// An undo stack.
/// Invariant: If an action is batched, so are the ones that come before it.
impl UndoStack {
    pub fn new() -> Self {
        Self {
            undo: vec![],
            redo: vec![],
        }
    }

    /// Undoes a single action and returns the inverse of that action,
    /// if the original action is still valid
    fn undo_action(&mut self, app: &mut App, action: UndoAction) -> Option<UndoAction> {
        match action {
            UndoAction::GadgetInsert { position } => {
                let (gadget, xy, _) = app
                    .grid
                    .remove(position)
                    .expect("A GadgetInsert action was inserted when no gadget was inserted");
                Some(UndoAction::GadgetRemove {
                    gadget,
                    position: xy,
                })
            }

            UndoAction::GadgetRemove { gadget, position } => {
                let size = gadget.size();
                app.grid.insert(gadget, position, size);
                Some(UndoAction::GadgetInsert { position })
            }

            UndoAction::AgentMove {
                position,
                direction,
            } => {
                if let Some(agent) = app.agent.as_mut() {
                    let old_position = agent.position();
                    let old_direction = agent.direction();

                    agent.set_position(position);
                    // Note that set_position also makes sure the direction is valid for that position
                    if agent.direction() != direction {
                        agent.flip();
                    }

                    Some(UndoAction::AgentMove {
                        position: old_position,
                        direction: old_direction,
                    })
                } else {
                    // We are no longer in play mode, so this action should get removed
                    None
                }
            }

            UndoAction::GadgetChangeState { position, state } => {
                let (gadget, _, _) = app
                    .grid
                    .get_mut(position)
                    .expect("GadgetChangeState requires the gadget to be there");
                let old_state = gadget.state();
                gadget.set_state(state);
                Some(UndoAction::GadgetChangeState {
                    position,
                    state: old_state,
                })
            }

            UndoAction::Batch(mut actions) => {
                let mut rev_actions = vec![];

                for action in actions.into_iter().rev() {
                    rev_actions.extend(self.undo_action(app, action));
                }

                Some(UndoAction::Batch(rev_actions))
            }
        }
    }

    pub fn undo(&mut self, app: &mut App) {
        // Just in case there were unbatched actions at the top of the stack
        self.batch();

        if let Some(action) = self.undo.pop() {
            let action = self.undo_action(app, action);
            self.redo.extend(action);
        }
    }

    pub fn redo(&mut self, app: &mut App) {
        // Must preserve the invariant!
        self.batch();

        if let Some(action) = self.redo.pop() {
            let action = self.undo_action(app, action);
            self.undo.extend(action);
        }
    }

    /// Adds an action to the undo stack, clearing the redo stack
    pub fn push(&mut self, action: UndoAction) {
        self.redo.clear();
        self.undo.push(action);
    }

    /// Ends the current list of undo actions, making it a batch,
    /// if there are any unbatched actions at the top
    pub fn batch(&mut self) {
        // Take advantage of the invariant
        if let Some(first_unbatched) = self.undo.iter().position(|action| {
            if let UndoAction::Batch(_) = action {
                false
            } else {
                true
            }
        }) {
            let vec = self.undo.drain(first_unbatched..).collect::<Vec<_>>();
            self.undo.push(UndoAction::Batch(vec));
        }
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }

    /// Batch all the actions in `other` and push that batch onto this stack
    pub fn append_as_batch(&mut self, other: &mut UndoStack) {
        let vec = std::mem::take(&mut other.undo);

        if vec.len() > 0 {
            self.push(UndoAction::Batch(vec));
        }
    }

    pub fn is_undo_empty(&self) -> bool {
        self.undo.is_empty()
    }

    pub fn is_redo_empty(&self) -> bool {
        self.redo.is_empty()
    }
}

pub struct App<'a> {
    gl: Rc<Context>,
    camera: Camera,
    center: Vec2,
    height: f64,
    grid: Grid<Gadget>,
    grid_mouse_position: Vec2,
    int_mouse_position: grid::XY,
    gadget_renderer: GadgetRenderer,
    /// A list of gadgets that can be selected from the selector
    gadget_select: Vec<Gadget>,
    gadget_selection: Option<usize>,
    /// The gadget currently being used to paint tiles
    gadget_tile: Option<Gadget>,
    agent: Option<Agent>,
    gadget_select_rep: Gadget,
    /// A list of gadget positions in the contraption that are selected,
    /// along with cached sizes
    selection: FnvHashSet<(grid::XY, grid::WH)>,
    selection_renderer: SelectionRenderer,
    /// The grid for moved gadgets
    moving: Grid<Gadget>,
    /// The grid to paste
    paste: Grid<Gadget>,
    paste_renderer: GadgetRenderer,
    mode: Mode,
    left_mouse_action: LeftMouseAction,
    ids: WidgetIds,
    ui_renderer: UiRenderer<'a>,
    fonts: Fonts,
    // One for editing, and one for playing
    undo_stacks: [Option<UndoStack>; 2],
    undo_stack_index: usize,
    modifiers: ModifiersState,
}

impl<'a> App<'a> {
    const HEIGHT_MIN: f64 = 1.0;
    const HEIGHT_MAX: f64 = 32.0;
    const WIDTH_MAX: f64 = 128.0;

    pub fn new(gl: Rc<Context>, ui: &mut Ui, _width: u32, _height: u32) -> Self {
        let camera = Camera::new_orthographic(
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, -1.0),
            vec3(0.0, 1.0, 0.0),
            1.0,
            1.0,
            1.0,
        );

        let grid = load_grid_from_url().unwrap_or_else(|| Grid::new());

        //let def = GadgetDef::from_traversals(2, 2, vec![((0, 0), (1, 1)), ((1, 1), (0, 0))]);

        //let gadget = Gadget::new(
        //    Rc::new(def),
        //    (3, 2),
        //    vec![
        //        None,
        //        Some(0),
        //        None,
        //        None,
        //        None,
        //        None,
        //        Some(1),
        //        None,
        //        None,
        //        None,
        //    ],
        //    0,
        //);
        //let size = gadget.size();
        //grid.insert(gadget, (1, 2), size);

        let def = GadgetDef::new(2, 0);

        let gadget_select_rep = Gadget::new(&Rc::new(def), (1, 1), vec![], State(0));

        let widget_ids = WidgetIds::new(ui.widget_id_generator());

        SHADERS.borrow_mut().init(&gl);
        TRIANGLESES.borrow_mut().init(());
        TEXTURES.borrow_mut().init(&gl);
        MODELS.borrow_mut().init(&gl);

        let gadget_renderer = GadgetRenderer::new(&gl);
        let paste_renderer = GadgetRenderer::new(&gl);
        let ui_renderer = UiRenderer::new(&gl);
        let selection_renderer = SelectionRenderer::new(&gl);

        let fonts = Fonts {
            regular: ui.fonts.insert(
                Font::from_bytes(&include_bytes!("../assets/fonts/Ubuntu-R.ttf")[..])
                    .expect("Cannot load regular font"),
            ),
            italic: ui.fonts.insert(
                Font::from_bytes(&include_bytes!("../assets/fonts/Ubuntu-RI.ttf")[..])
                    .expect("Cannot load italic font"),
            ),
            bold: ui.fonts.insert(
                Font::from_bytes(&include_bytes!("../assets/fonts/Ubuntu-B.ttf")[..])
                    .expect("Cannot load bold font"),
            ),
            bold_italic: ui.fonts.insert(
                Font::from_bytes(&include_bytes!("../assets/fonts/Ubuntu-BI.ttf")[..])
                    .expect("Cannot load bold italic font"),
            ),
        };

        ui.theme.font_id = Some(fonts.regular);

        Self {
            gl,
            camera,
            center: vec2(0.0, 0.0),
            height: 10.0,
            grid,
            grid_mouse_position: vec2(0.0, 0.0),
            int_mouse_position: vec2(0, 0),
            gadget_renderer,
            gadget_select: preset_gadgets::preset_gadgets(),
            gadget_selection: None,
            gadget_tile: None,
            agent: None,
            gadget_select_rep,
            selection: FnvHashSet::default(),
            selection_renderer,
            moving: Grid::new(),
            paste: Grid::new(),
            paste_renderer,
            mode: Mode::None,
            left_mouse_action: LeftMouseAction::None,
            ids: widget_ids,
            ui_renderer,
            fonts,
            undo_stacks: [Some(UndoStack::new()), Some(UndoStack::new())],
            undo_stack_index: 0,
            modifiers: ModifiersState::default(),
        }
    }

    // Convenience functions that assume the logic is correct
    pub fn undo_stack_mut(&mut self) -> &mut UndoStack {
        self.undo_stacks[self.undo_stack_index]
            .as_mut()
            .expect("Tried to get undo stack while undoing/redoing")
    }

    pub fn undo_stack_take(&mut self) -> UndoStack {
        self.undo_stacks[self.undo_stack_index]
            .take()
            .expect("Tride to take undo stack while undoing/redoing")
    }

    /// Some things will no longer be valid after an undo.
    pub fn invalidate_before_undo(&mut self) {
        // A selected gadget may be deleted
        self.selection.clear();
    }

    pub fn undo(&mut self) {
        self.invalidate_before_undo();

        let mut stack = self.undo_stack_take();
        stack.undo(self);
        self.undo_stacks[self.undo_stack_index] = Some(stack);
    }

    pub fn redo(&mut self) {
        self.invalidate_before_undo();

        let mut stack = self.undo_stack_take();
        stack.redo(self);
        self.undo_stacks[self.undo_stack_index] = Some(stack);
    }

    pub fn add_gadget_to_grid(&mut self, gadget: Gadget, position: grid::XY) {
        let size = gadget.size();

        let removed = self.grid.insert(gadget, position, size);
        for (gadget, xy, _) in removed.into_iter() {
            self.undo_stack_mut().push(UndoAction::GadgetRemove {
                gadget,
                position: xy,
            });
        }

        self.undo_stack_mut()
            .push(UndoAction::GadgetInsert { position });
    }

    pub fn remove_gadget_from_grid(&mut self, position: grid::XY) {
        if let Some((gadget, xy, _)) = self.grid.remove(position) {
            self.undo_stack_mut().push(UndoAction::GadgetRemove {
                gadget,
                position: xy,
            });
        }
    }

    pub fn remove_selected_gadgets(&mut self) {
        for (xy, _) in self.selection.iter().copied().collect::<Vec<_>>() {
            self.remove_gadget_from_grid(xy);
        }
        self.selection.clear();
    }

    pub fn copy_selected_gadgets(&mut self, center: bool) -> Grid<Gadget> {
        let imm = self
            .grid
            .iter()
            .filter(|(_, xy, wh)| self.selection.contains(&(*xy, *wh)))
            .cloned()
            .collect::<Grid<_>>();

        if center {
            imm.center()
        } else {
            imm.translate(-self.int_mouse_position)
        }
    }

    pub fn clamp_height(&mut self, ui: &Ui) {
        self.height = self
            .height
            .max(Self::HEIGHT_MIN)
            .min(Self::HEIGHT_MAX)
            .min(Self::WIDTH_MAX * ui.win_h / ui.win_w);
    }

    /// Rotates the active thing by a certain number of counterclockwise right turns
    pub fn rotate_active(&mut self, num_turns: i32) {
        if let Some(gadget) = &mut self.gadget_tile {
            gadget.rotate(num_turns);
        }

        if self.mode == Mode::AgentPlace {
            if let Some(agent) = &mut self.agent {
                agent.flip();
            }
        }

        if self.mode == Mode::GadgetMove {
            self.moving =
                std::mem::take(&mut self.moving).rotate(vec2(0.5, 0.5), num_turns as isize);
        }

        if self.mode == Mode::GadgetPaste {
            self.paste = std::mem::take(&mut self.paste).rotate(vec2(0.5, 0.5), num_turns as isize);
        }
    }

    pub fn flip_x_active(&mut self) {
        if let Some(gadget) = &mut self.gadget_tile {
            gadget.flip_ports_x();
        }

        if self.mode == Mode::GadgetMove {
            self.moving = std::mem::take(&mut self.moving).flip_x(0.5);
        }

        if self.mode == Mode::GadgetPaste {
            self.paste = std::mem::take(&mut self.paste).flip_x(0.5);
        }
    }

    pub fn flip_y_active(&mut self) {
        if let Some(gadget) = &mut self.gadget_tile {
            gadget.flip_ports_y();
        }

        if self.mode == Mode::GadgetMove {
            self.moving = std::mem::take(&mut self.moving).flip_y(0.5);
        }

        if self.mode == Mode::GadgetPaste {
            self.paste = std::mem::take(&mut self.paste).flip_y(0.5);
        }
    }

    pub fn twist_active(&mut self) {
        if let Some(gadget) = &mut self.gadget_tile {
            gadget.twist_bottom_right();
        }
    }

    pub fn cycle_state_active(&mut self) {
        if let Some(gadget) = &mut self.gadget_tile {
            gadget.cycle_state();
        }
    }

    pub fn pan(&mut self, xy: Vec2) {
        self.center += xy;
    }

    pub fn zoom(&mut self, center: Vec2, amount: f64, ui: &Ui) {
        let prev_height = self.height;
        self.height += amount;
        self.clamp_height(&ui);
        self.center = center + (self.center - center) * self.height / prev_height;
    }

    pub fn cut(&mut self, center: bool) {
        if self.selection.len() > 0 {
            self.paste = self.copy_selected_gadgets(center);
            self.remove_selected_gadgets();
            self.set_mode(Mode::GadgetPaste);
        }
    }

    pub fn copy(&mut self, center: bool) {
        if self.selection.len() > 0 {
            self.paste = self.copy_selected_gadgets(center);
            self.set_mode(Mode::GadgetPaste);
        }
    }

    pub fn paste(&mut self) {
        if self.mode != Mode::GadgetMove && !self.paste.is_empty() {
            self.set_mode(Mode::GadgetPaste);
        }
    }

    pub fn update(&mut self, ui: &mut Ui) {
        self.clamp_height(ui);

        self.update_ui(ui);

        self.camera.set_orthographic_projection(
            ui.win_w / ui.win_h * self.height,
            self.height,
            1.0,
        );

        let cx = self.center.x;
        let cy = self.center.y;
        self.camera
            .set_view(vec3(cx, cy, 0.0), vec3(cx, cy, -1.0), vec3(0.0, 1.0, 0.0));
    }

    pub fn render(&mut self, ui: &mut Ui, width: f64, height: f64) {
        self.gl.clear();

        self.selection_renderer.render(
            self.selection.iter().copied(),
            &self.camera,
            vec2(0, 0),
            SelectionRenderer::Z,
        );

        render::render_grid(
            &self.grid,
            &self.camera,
            &mut self.gadget_renderer,
            vec2(0, 0),
            0.0,
            true,
        );

        if self.mode == Mode::GadgetMove {
            render::render_grid(
                &self.moving,
                &self.camera,
                &mut self.paste_renderer,
                self.int_mouse_position,
                -0.25,
                false,
            );

            self.selection_renderer.render(
                self.moving.iter().map(|(_, xy, wh)| (*xy, *wh)),
                &self.camera,
                self.int_mouse_position,
                SelectionRenderer::Z - 0.1,
            );
        }

        if self.mode == Mode::GadgetPaste {
            render::render_grid(
                &self.paste,
                &self.camera,
                &mut self.paste_renderer,
                self.int_mouse_position,
                -0.25,
                false,
            );
        }

        if let Some(gadget) = &self.gadget_tile {
            gadget
                .renderer()
                .model(&self.gl)
                .prepare_render()
                .render_position(
                    vec3(
                        self.int_mouse_position.x as f64,
                        self.int_mouse_position.y as f64,
                        -0.25,
                    ),
                    &self.camera,
                );
        }

        if let Some(agent) = &self.agent {
            agent.render(&self.camera);
        }

        self.render_ui(ui, width, height);
    }

    pub fn handle_input(&mut self, event: &Event<()>) {
        match event {
            //Event::WindowEvent {
            //    event:
            //        WindowEvent::MouseWheel {
            //            delta: MouseScrollDelta::PixelDelta(LogicalPosition { y: delta, .. }),
            //            ..
            //        },
            //    ..
            //} => {
            //    self.height = self.height + *delta / 64.0
            //}

            //TODO: Implement the event in winit for the web
            //Event::WindowEvent {
            //    event: WindowEvent::ModifiersChanged(state),
            //    ..
            //} => {
            //    self.modifiers = *state;
            //    log!("Modifiers: {:?}", self.modifiers);
            //}
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                modifiers,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                if modifiers.ctrl() {
                    if let ElementState::Pressed = state {
                        match keycode {
                            VirtualKeyCode::Z => {
                                self.undo();
                            }

                            VirtualKeyCode::Y => {
                                self.redo();
                            }

                            VirtualKeyCode::X => {
                                self.cut(false);
                            }

                            VirtualKeyCode::C => {
                                self.copy(false);
                            }

                            VirtualKeyCode::V => {
                                self.paste();
                            }

                            VirtualKeyCode::S => {
                                crate::save_grid_in_url(&self.grid);
                            }

                            VirtualKeyCode::A => {
                                if self.mode != Mode::GadgetMove {
                                    self.set_mode(Mode::Select);
                                    self.selection
                                        .extend(self.grid.iter().map(|(_, xy, wh)| (*xy, *wh)));
                                }
                            }

                            _ => {}
                        }
                    }
                } else {
                    if let ElementState::Pressed = state {
                        match keycode {
                            VirtualKeyCode::R | VirtualKeyCode::T => {
                                let num_turns = if *keycode == VirtualKeyCode::R { 1 } else { -1 };
                                self.rotate_active(num_turns);
                            }

                            VirtualKeyCode::X => {
                                self.flip_x_active();
                            }

                            VirtualKeyCode::Y => {
                                self.flip_y_active();
                            }

                            VirtualKeyCode::U => {
                                self.twist_active();
                            }

                            VirtualKeyCode::C => {
                                self.cycle_state_active();
                            }

                            VirtualKeyCode::Delete | VirtualKeyCode::Back => {
                                self.remove_selected_gadgets();
                                self.undo_stack_mut().batch();
                            }

                            VirtualKeyCode::Escape => {
                                if self.mode == Mode::GadgetPaste {
                                    self.set_mode(Mode::Select);
                                }
                            }

                            _ => {}
                        }
                    }

                    if self.mode == Mode::Play {
                        if *state == ElementState::Pressed && modifiers.is_empty() {
                            let dir = match keycode {
                                VirtualKeyCode::W | VirtualKeyCode::Up => Some(vec2(0, 1)),
                                VirtualKeyCode::A | VirtualKeyCode::Left => Some(vec2(-1, 0)),
                                VirtualKeyCode::S | VirtualKeyCode::Down => Some(vec2(0, -1)),
                                VirtualKeyCode::D | VirtualKeyCode::Right => Some(vec2(1, 0)),
                                _ => None,
                            };

                            if let Some(dir) = dir {
                                let agent = self.agent.as_mut().unwrap();
                                // Borrowing rules require that self.undo_stack is obtained directly
                                let undo_stack = self.undo_stacks[self.undo_stack_index]
                                    .as_mut()
                                    .expect("Tried to get undo stack while undoing/redoing");
                                let prev_position = agent.position();
                                let prev_direction = agent.direction();

                                let result = agent.advance(&mut self.grid, dir);

                                if agent.position() != prev_position
                                    || agent.direction() != prev_direction
                                {
                                    undo_stack.push(UndoAction::AgentMove {
                                        position: prev_position,
                                        direction: prev_direction,
                                    })
                                }

                                if let Some((_, xy, state)) = result {
                                    undo_stack.push(UndoAction::GadgetChangeState {
                                        position: xy,
                                        state,
                                    });
                                }

                                undo_stack.batch();

                                save_grid_in_url(&self.grid);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Characters that are special in the fragment portion of a URL,
/// as defined in https://tools.ietf.org/rfc/rfc3986.txt, page 49
const SPECIAL_CHARS: AsciiSet = percent_encoding::NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~')
    .remove(b'!')
    .remove(b'$')
    .remove(b'&')
    .remove(b'\'')
    .remove(b'(')
    .remove(b')')
    .remove(b'*')
    .remove(b'+')
    .remove(b',')
    .remove(b';')
    .remove(b'=')
    .remove(b':')
    .remove(b'@')
    .remove(b'/')
    .remove(b'?');

/// Attempts to save the grid as part of the URL's hash map, and returs whether it saved
pub fn save_grid_in_url(grid: &Grid<Gadget>) -> bool {
    let (base64, padding) = bit_serde::to_base64(grid)
        .map_err(|e| {
            elog!("Grid failed to save: {}", e);
            e
        })
        .unwrap_or_else(|e| (String::new(), 0));

    if base64.is_empty() {
        window().location().set_hash("").unwrap();
        return false;
    }

    let string = format!("{}{}", base64, padding);
    window().location().set_hash(&string).map_or_else(
        |e| {
            elog!("Grid failed to save: {:?}", e);
            false
        },
        |()| true,
    )
}

pub fn load_grid_from_url() -> Option<Grid<Gadget>> {
    // Avoid panicking here
    let mut string = window().location().hash().ok()?;

    if string.len() == 0 {
        return None;
    }

    string = string.replace("#", "");

    let padding = string
        .pop()
        .or_else(|| {
            elog!("Failed to load grid: URL hash is empty");
            None
        })?
        .to_string()
        .parse()
        .or_else(|e| {
            elog!("Failed to load grid: {}", e);
            Err(e)
        })
        .ok()?;

    bit_serde::from_base64(&string, padding)
        .or_else(|e| {
            elog!("Failed to load grid: {}", e);
            Err(e)
        })
        .ok()
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let event_loop = EventLoop::new();

    let original_width = crate::window().inner_width().unwrap().as_f64().unwrap();
    let original_height = crate::window().inner_height().unwrap().as_f64().unwrap();

    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(original_width, original_height))
        .with_title("Gadget Up! 2")
        .build(&event_loop)
        .unwrap();

    // This is a hack to get 'rustc' to stop complaining
    // about this function 'not existing' and move on to
    // more interesting errors.
    //
    // The `transmute_copy` will not be executed,
    // though there is a `fake_panic` that doesn't
    // return and doesn't say it doesn't return, just in case
    let gl = {
        #[cfg(target_arch = "wasm32")]
        let gl = {
            let canvas = window.canvas();
            // winit 0.22 sets the width and height via style,
            // which overrides the style "canvas".
            // No thank you.
            let style = canvas.style();
            style.remove_property("width").unwrap();
            style.remove_property("height").unwrap();
            style.set_property("cursor", "auto");

            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .body()
                .unwrap()
                .append_child(&canvas)
                .unwrap();

            let gl = canvas
                .get_context("webgl")
                .expect("init webgl fail 1")
                .expect("init webgl fail 2")
                .dyn_into::<web_sys::WebGlRenderingContext>()
                .unwrap();

            glow::Context::from_webgl1_context(gl)
        };

        #[cfg(not(target_arch = "wasm32"))]
        let gl = {
            fake_panic();
            unsafe { std::mem::transmute_copy(&()) }
        };

        Rc::new(Context::from_glow(gl).unwrap())
    };

    gl.set_clear_color(0.0, 0.0, 0.0, 1.0);
    gl.set_clear_depth(-1.0);

    gl.set_depth_test_mode(Some(DepthTestMode {
        depth_mask: true,
        function: DepthTestFunction::GreaterOrEqual,
        ..DepthTestMode::default()
    }));

    gl.set_blend_mode(Some(BlendMode::default()));

    let mut frame = 0;

    let mut ui = UiBuilder::new([original_width, original_height])
        .theme(ui::theme())
        .build();
    let mut app = App::new(gl, &mut ui, original_width as u32, original_height as u32);

    let mut width = original_width;
    let mut height = original_height;

    event_loop.run(move |event, _, ctrl| {
        *ctrl = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *ctrl = ControlFlow::Exit,

            Event::MainEventsCleared => {
                // These are in logical size already
                width = crate::window().inner_width().unwrap().as_f64().unwrap();
                height = crate::window().inner_height().unwrap().as_f64().unwrap();

                ui.win_w = width;
                ui.win_h = height;

                app.update(&mut ui);
                app.render(&mut ui, width, height);

                frame += 1;
                window.request_redraw();
            }

            event => {
                app.handle_input(&event);

                if let Some(event) = conrod_winit::v021_convert_event_wh!(
                    // but the event is in physical coordinates
                    event,
                    // so convert the size to physical coordinates here
                    LogicalSize::new(width, height).to_physical::<f64>(window.scale_factor()),
                    // and pass in the scale factor
                    window.scale_factor()
                ) {
                    ui.handle_event(event);
                }

                //log!("width: {}, height: {}", width, height);
            }
        }
    });
}

fn window() -> web_sys::Window {
    web_sys::window().expect("No window!")
}

fn fake_panic() {
    unreachable!();
}
