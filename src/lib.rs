extern crate cgmath;
extern crate fnv;
extern crate graphics;
extern crate itertools;
extern crate three_d;

mod graphics_ex;
mod grid;
mod render;

use std::rc::Rc;
use three_d::{Camera, Event, Screen, Window};
use three_d::{vec3, vec4};
use three_d::gl::Glstruct;
use wasm_bindgen::prelude::*;
use web_sys::console;

use grid::Grid;
use render::GadgetRenderer;

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
    grid: Grid<()>,
    gadget_renderer: GadgetRenderer,
}

impl App {
    pub fn new(gl: &Rc<Glstruct>) -> Self {
        let camera = Camera::new_orthographic(
            &gl,
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, -1.0),
            vec3(0.0, 1.0, 0.0),
            1.0,
            1.0,
            1.0,
        );

        Self {
            gl: Rc::clone(gl),
            camera,
            grid: Grid::new(),
            gadget_renderer: GadgetRenderer::new(&gl)
        }
    }

    pub fn render(&mut self, width: f32, height: f32) {
        self.camera.set_view(
            vec3(0.0, 0.0, 0.0),
            vec3(0.0, 0.0, -1.0),
            vec3(0.0, 1.0, 0.0),
        );
        self.camera.set_orthographic_projection(width / height * 16.0, 16.0, 1.0);

        render::render_grid(&self.grid, &self.camera, &mut self.gadget_renderer);
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

    let mut app = App::new(&gl);

    window
        .render_loop(move |frame_input| {
            let width = crate::window().inner_width().unwrap().as_f64().unwrap() as f32;
            let height = crate::window().inner_height().unwrap().as_f64().unwrap() as f32;

            //log!("width: {}, height: {}", width, height);

            Screen::write(
                &gl,
                0,
                0,
                original_width,
                original_height,
                Some(&vec4(0.0, 0.0, 0.0, 1.0)),
                Some(1.0),
                &mut || {
                    app.render(width, height);
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
