extern crate cgmath;
extern crate conrod_core;
extern crate conrod_derive;
extern crate fnv;
extern crate itertools;
extern crate three_d;

mod bitfield;
mod gadget;
mod graphics_ex;
mod grid;
mod input;
mod math;
mod model;
mod preset_gadgets;
mod render;
mod shape;
mod ui;
mod widget;

use cgmath::{vec2, Vector2};
use conrod_core::{Ui, UiBuilder};
use std::cell::RefCell;
use std::rc::Rc;
use three_d::gl::Glstruct;
use three_d::state;
use three_d::{Vec2, vec3, vec4};
use three_d::{Camera, Event, Screen, Window};
use wasm_bindgen::prelude::*;
use web_sys::console;

use gadget::{Gadget, GadgetDef, Agent};
use graphics_ex::GraphicsEx;
use grid::Grid;
use render::GadgetRenderer;
use ui::{WidgetIds, Mode};
use model::Model;

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
    gl: Rc<Glstruct>,
    camera: Camera,
    center: Vector2<f64>,
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

    pub fn new(gl: &Rc<Glstruct>, ui: &mut Ui, width: u32, height: u32) -> Self {
        let camera = Camera::new_orthographic(
            &gl,
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
            (ui.win_w / ui.win_h * self.height) as f32,
            self.height as f32,
            1.0,
        );

        self.update_ui(ui);

        let cx = self.center.x as f32;
        let cy = self.center.y as f32;
        self.camera.set_view(vec3(cx, cy, 0.0), vec3(cx, cy, -1.0), vec3(0.0, 1.0, 0.0));
    }

    pub fn render(&mut self, ui: &mut Ui, width: f32, height: f32) {
        render::render_grid(&self.grid, &self.camera, &mut self.gadget_renderer);

        if let Some(gadget) = &self.gadget_tile {
            gadget.renderer().model(&self.gl).render_position(
                vec3(
                    self.gadget_tile_xy.x as f32,
                    self.gadget_tile_xy.y as f32,
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

    pub fn handle_input(&mut self, event: &Event) {
        match event {
            Event::MouseWheel { delta } => {
                self.height = (self.height + 4.0 * delta)
                    .max(Self::HEIGHT_MIN)
                    .min(Self::HEIGHT_MAX)
            }

            Event::Key { state, kind } => {
                if kind == "R" || kind == "T" {
                    if let three_d::State::Pressed = state {
                        if let Some(gadget) = &mut self.gadget_tile {
                            gadget.rotate_ports(if kind == "R" {1} else {-1});
                        }

                        if self.mode == Mode::AgentPlace {
                            if let Some(agent) = &mut self.agent {
                                agent.rotate(1);
                            }
                        }
                    }
                } else if kind == "F" {
                    if let three_d::State::Pressed = state {
                        if let Some(gadget) = &mut self.gadget_tile {
                            gadget.flip_ports();
                        }
                    }
                } else if kind == "C" {
                    if let three_d::State::Pressed = state {
                        if let Some(gadget) = &mut self.gadget_tile {
                            gadget.cycle_state();
                        }
                    }
                }

                if self.mode == Mode::Play {
                    if let three_d::State::Pressed = state {
                        let dir = if kind == "W" || kind == "ArrowUp" {
                            Some(vec2(0, 1))
                        } else if kind == "A" || kind == "ArrowLeft" {
                            Some(vec2(-1, 0))
                        } else if kind == "S" || kind == "ArrowDown" {
                            Some(vec2(0, -1))
                        } else if kind == "D" || kind == "ArrowRight" {
                            Some(vec2(1, 0))
                        } else {
                            None
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

    let mut window = Window::new_default("Gadget Up! 2").unwrap();
    let gl = window.gl();

    let mut frame = 0;
    let original_width = crate::window().inner_width().unwrap().as_f64().unwrap() as usize;
    let original_height = crate::window().inner_height().unwrap().as_f64().unwrap() as usize;

    let mut ui = UiBuilder::new([original_width as f64, original_height as f64]).build();
    let mut app = App::new(&gl, &mut ui, original_width as u32, original_height as u32);

    state::depth_write(&gl, true);
    state::depth_test(&gl, three_d::DepthTestType::GreaterOrEqual);
    state::blend(&gl, three_d::BlendType::SrcAlphaOneMinusSrcAlpha);

    window
        .render_loop(move |frame_input| {
            let width = crate::window().inner_width().unwrap().as_f64().unwrap() as f32;
            let height = crate::window().inner_height().unwrap().as_f64().unwrap() as f32;

            ui.win_w = width as f64;
            ui.win_h = height as f64;

            for event in frame_input.events.iter() {
                app.handle_input(event);

                if let Some(event) = input::convert(event.clone(), width as f64, height as f64) {
                    ui.handle_event(event);
                }
            }

            //log!("width: {}, height: {}", width, height);

            app.update(&mut ui);

            Screen::write(
                &gl,
                0,
                0,
                original_width,
                original_height,
                Some(&vec4(0.0, 0.0, 0.0, 1.0)),
                Some(-1.0),
                &mut || {
                    app.render(&mut ui, width, height);
                },
            )
            .unwrap();

            frame += 1;
        })
        .unwrap();

    Ok(())
}

fn window() -> web_sys::Window {
    web_sys::window().expect("No window!")
}
