extern crate three_d;
extern crate fnv;

mod grid;

use wasm_bindgen::prelude::*;
use web_sys::console;

use three_d::{Camera, Window, Program, VertexBuffer, ElementBuffer, Screen};

macro_rules! log {
    ( $($t:tt)* ) => {
        web_sys::console::log_1(&format!( $($t)* ).into());
    };
}

// When the `wee_alloc` feature is enabled, this uses `wee_alloc` as the global
// allocator.
//
// If you don't want to use `wee_alloc`, you can safely delete this.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;


// This is like the `main` function, except for JavaScript.
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    // This provides better error messages in debug mode.
    // It's disabled in release mode so it doesn't bloat up the file size.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    use three_d::{vec3, vec4};

    let mut window = Window::new_default("Gadget Up! 2").unwrap();
    let gl = window.gl();

    // dummy values, wish Camera::new were public
    let mut camera = Camera::new_orthographic(&gl, vec3(0.0, 0.0, 0.0), vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), 1.0, 1.0, 1.0);

    let program = Program::from_source(&gl,
        include_str!("../assets/shaders/color.vert"),
        include_str!("../assets/shaders/color.frag")).unwrap();

    let positions: Vec<f32> = vec![
        0.5, 0.5, 0.0,
        -0.5, 0.5, 0.0,
        -0.5, -0.5, 0.0,
        0.5, -0.5, 0.0,
    ];
    let position_buffer = VertexBuffer::new_with_static_f32(&gl, &positions).unwrap();

    let colors: Vec<f32> = vec![
        0.0, 1.0, 1.0,
        0.0, 1.0, 1.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 1.0,
    ];
    let color_buffer = VertexBuffer::new_with_static_f32(&gl, &colors).unwrap();

    let elements = ElementBuffer::new_with_u32(&gl, &[0, 1, 2, 2, 3, 0]).unwrap();

    let mut frame = 0;
    let original_width = crate::window().inner_width().unwrap().as_f64().unwrap() as usize;
    let original_height = crate::window().inner_height().unwrap().as_f64().unwrap() as usize;

    window.render_loop(move |frame_input| {
        let width = crate::window().inner_width().unwrap().as_f64().unwrap() as f32;
        let height = crate::window().inner_height().unwrap().as_f64().unwrap() as f32;

        log!("width: {}, height: {}", width, height);

        camera.set_view(vec3(0.0, 0.0, 0.0), vec3(0.0, 0.0, -1.0), vec3(0.0, 1.0, 0.0));
        camera.set_orthographic_projection(width / height * 2.0, 2.0, 1.0);

        Screen::write(&gl, 0, 0, original_width, original_height, Some(&vec4((frame % 60) as f32 / 60.0, 0.0, 0.0, 1.0)), Some(1.0), &|| {
            program.use_attribute_vec3_float(&position_buffer, "position").unwrap();
            program.use_attribute_vec3_float(&color_buffer, "color").unwrap();

            let world_view_projection = camera.get_projection() * camera.get_view();
            program.add_uniform_mat4("worldViewProjectionMatrix", &world_view_projection).unwrap();

            program.draw_elements(&elements);
        }).unwrap();

        frame += 1;
    }).unwrap();

    Ok(())
}

fn window() -> web_sys::Window {
    web_sys::window().expect("No window!")
}
