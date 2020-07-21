extern crate cgmath;
extern crate conrod_core;
extern crate conrod_derive;
extern crate conrod_winit;
extern crate fnv;
extern crate itertools;
extern crate winit;
extern crate glow;

mod camera;
mod bitfield;
mod gadget;
mod graphics_ex;
mod grid;
mod math;
mod model;
mod preset_gadgets;
mod render;
mod shape;
mod ui;
mod widget;

use cgmath::{vec2, vec3};
use conrod_core::{Ui, UiBuilder};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;
use winit::event_loop::{EventLoop, ControlFlow};
use winit::event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode};
use winit::event::MouseScrollDelta;
use winit::dpi::LogicalPosition;
use winit::window::WindowBuilder;
use golem::{Context, GolemError, VertexBuffer, ElementBuffer};
use golem::{ShaderProgram, ShaderDescription, GeometryMode};
use golem::{Attribute, AttributeType, Uniform, UniformType, UniformValue};
use golem::depth::{DepthTestFunction, DepthTestMode};
use golem::Dimension::{D2, D3, D4};

use camera::Camera;
use gadget::{Gadget, GadgetDef, Agent};
use graphics_ex::GraphicsEx;
use grid::Grid;
use render::GadgetRenderer;
use ui::{WidgetIds, Mode};
use model::Model;
use math::Vec2;

#[macro_export]
macro_rules! log {
    ( $($t:tt)* ) => {
        // To get rid of the unnecessary rust-analyzer error
        unsafe {
            web_sys::console::log_1(&format!( $($t)* ).into());
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

pub struct App {
    gl: Rc<Context>,
    camera: Camera,
    center: Vec2,
    height: f64,
    grid: Grid<Gadget>,
    gadget_renderer: GadgetRenderer,
    /// A list of gadgets that can be selected from the selector
    gadget_select: Vec<Gadget>,
    gadget_selection: Option<usize>,
    /// The gadget currently being used to paint tiles
    gadget_tile: Option<Gadget>,
    gadget_tile_xy: grid::XY,
    agent: Option<Agent>,
    agent_position: Vec2,
    agent_model: Rc<Model>,
    gadget_select_rep: Gadget,
    mode: Mode,
    ids: WidgetIds,
    ui_renderer: GraphicsEx,
}

impl App {
    const HEIGHT_MIN: f64 = 1.0;
    const HEIGHT_MAX: f64 = 32.0;

    pub fn new(gl: &Rc<Context>, ui: &mut Ui, width: u32, height: u32) -> Self {
        let camera = Camera::new_orthographic(
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, -1.0),
            vec3(0.0, 1.0, 0.0),
            1.0,
            1.0,
            1.0,
        );

        let mut grid = Grid::new();

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

        let def = GadgetDef::new(
            2,
            0,
        );

        let gadget_select_rep = Gadget::new(
            &Rc::new(def),
            (1, 1),
            vec![],
            0,
        );

        let widget_ids = WidgetIds::new(ui.widget_id_generator());

        Self {
            gl: Rc::clone(gl),
            camera,
            center: vec2(0.0, 0.0),
            height: 10.0,
            grid,
            gadget_renderer: GadgetRenderer::new(&gl),
            gadget_select: preset_gadgets::preset_gadgets(),
            gadget_selection: None,
            gadget_tile: None,
            gadget_tile_xy: vec2(0, 0),
            agent: None,
            agent_position: vec2(0.0, 0.0),
            agent_model: Rc::new(Agent::new_shared_model(gl)),
            gadget_select_rep,
            mode: Mode::None,
            ids: widget_ids,
            ui_renderer: GraphicsEx::new(&gl),
        }
    }

    pub fn update(&mut self, ui: &mut Ui) {
        self.camera.set_orthographic_projection(
            ui.win_w / ui.win_h * self.height,
            self.height,
            1.0,
        );

        self.update_ui(ui);

        let cx = self.center.x;
        let cy = self.center.y;
        self.camera.set_view(vec3(cx, cy, 0.0), vec3(cx, cy, -1.0), vec3(0.0, 1.0, 0.0));
    }

    pub fn render(&mut self, ui: &mut Ui, width: f64, height: f64) {
        render::render_grid(&self.grid, &self.camera, &mut self.gadget_renderer);

        if let Some(gadget) = &self.gadget_tile {
            gadget.renderer().model(&self.gl).render_position(
                vec3(
                    self.gadget_tile_xy.x as f64,
                    self.gadget_tile_xy.y as f64,
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
            Event::WindowEvent { event: WindowEvent::MouseWheel {delta: MouseScrollDelta::PixelDelta(
                LogicalPosition {y: delta, ..}
            ), ..}, ..} => {
                self.height = (self.height + *delta)
                    .max(Self::HEIGHT_MIN)
                    .min(Self::HEIGHT_MAX)
            }

            Event::WindowEvent { event: WindowEvent::KeyboardInput {input: KeyboardInput {
                virtual_keycode: Some(keycode), state, ..
            }, ..}, ..} => {

                match keycode {
                    VirtualKeyCode::R | VirtualKeyCode::T => {
                        if let ElementState::Pressed = state {
                            if let Some(gadget) = &mut self.gadget_tile {
                                gadget.rotate_ports(if *keycode == VirtualKeyCode::R {1} else {-1});
                            }

                            if self.mode == Mode::AgentPlace {
                                if let Some(agent) = &mut self.agent {
                                    agent.rotate(1);
                                }
                            }
                        }
                    },

                    VirtualKeyCode::F => {
                        if let ElementState::Pressed = state {
                            if let Some(gadget) = &mut self.gadget_tile {
                                gadget.flip_ports();
                            }
                        }
                    },

                    VirtualKeyCode::C => {
                        if let ElementState::Pressed = state {
                            if let Some(gadget) = &mut self.gadget_tile {
                                gadget.cycle_state();
                            }
                        }
                    },

                    _ => {}
                }

                if self.mode == Mode::Play {
                    if let ElementState::Pressed = state {
                        let dir = match keycode {
                            VirtualKeyCode::W | VirtualKeyCode::Up => Some(vec2(0, 1)),
                            VirtualKeyCode::A | VirtualKeyCode::Left => Some(vec2(-1, 0)),
                            VirtualKeyCode::S | VirtualKeyCode::Down => Some(vec2(0, -1)),
                            VirtualKeyCode::D | VirtualKeyCode::Right => Some(vec2(1, 0)),
                            _ => None,
                        };

                        if let Some(dir) = dir {
                            self.agent.as_mut().unwrap().advance(&mut self.grid, dir);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    
    let event_loop = EventLoop::new();

    let window = WindowBuilder::new().with_title("Gadget Up! 2").build(&event_loop).unwrap();
    let gl = web_sys::window().unwrap().document().unwrap().get_element_by_id("canvas").unwrap()
        .dyn_into::<web_sys::HtmlCanvasElement>().map_err(|_| ()).unwrap()
        .get_context("webgl").expect("init webgl fail 1").expect("init webgl fail 2")
        .dyn_into::<web_sys::WebGlRenderingContext>().unwrap();

    // This is a hack to get 'rustc' to stop complaining
    // about this function 'not existing' and move on to
    // more interesting errors.
    // 
    // The `transmute_copy` will not be executed,
    // though there is a `fake_panic` that doesn't
    // return and doesn't say it doesn't return, just in case
    #[cfg(target_arch = "wasm32")]
    let gl = glow::Context::from_webgl1_context(gl);
    #[cfg(not(target_arch = "wasm32"))]
    fake_panic();
    #[cfg(not(target_arch = "wasm32"))]
    let gl = unsafe { std::mem::transmute_copy(&()) };

    let gl = Context::from_glow(gl).unwrap();

    gl.set_depth_test_mode(Some(DepthTestMode {
        depth_mask: true,
        function: DepthTestFunction::GreaterOrEqual,
        range_near: -1.0,
        range_far: 0.0,
    }));

    let mut frame = 0;
    let original_width = crate::window().inner_width().unwrap().as_f64().unwrap() as usize;
    let original_height = crate::window().inner_height().unwrap().as_f64().unwrap() as usize;

    let mut ui = UiBuilder::new([original_width as f64, original_height as f64]).build();
    let mut app = App::new(&Rc::new(gl), &mut ui, original_width as u32, original_height as u32);

    event_loop.run(move |event, _, ctrl| {
        let width = crate::window().inner_width().unwrap().as_f64().unwrap();
        let height = crate::window().inner_height().unwrap().as_f64().unwrap();

        *ctrl = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *ctrl = ControlFlow::Exit,

            Event::MainEventsCleared => {
                ui.win_w = width;
                ui.win_h = height;

                app.update(&mut ui);
                app.render(&mut ui, width, height);

                frame += 1;
                window.request_redraw();
            },

            event => {
                app.handle_input(&event);

                if let Some(event) = conrod_winit::v021_convert_event!(event, &window) {
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