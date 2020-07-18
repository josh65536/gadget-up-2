use conrod_core::{event, input, Point, Scalar};
use three_d::{Event, MouseButton, State};

fn convert_button(button: MouseButton) -> input::MouseButton {
    match button {
        MouseButton::Left => input::MouseButton::Left,
        MouseButton::Middle => input::MouseButton::Middle,
        MouseButton::Right => input::MouseButton::Right,
    }
}

pub fn convert(event: Event, win_w: Scalar, win_h: Scalar) -> Option<event::Input> {
    // Translate the coordinates from top-left-origin-with-y-down to centre-origin-with-y-up.
    let translate_coords = |(x, y): (f64, f64)| (x - win_w / 2.0, -(y - win_h / 2.0));

    match event {
        Event::MouseClick { state, button, .. } => match state {
            State::Pressed => Some(event::Input::Press(input::Button::Mouse(convert_button(
                button,
            )))),
            State::Released => Some(event::Input::Release(input::Button::Mouse(convert_button(
                button,
            )))),
        },

        Event::CursorMoved { position } => {
            let (x, y) = translate_coords(position);
            Some(event::Input::Motion(input::Motion::MouseCursor { x, y }))
        }

        Event::MouseWheel { delta } => Some(event::Input::Motion(input::Motion::Scroll {
            x: 0.0,
            y: delta,
        })),

        _ => None,
    }
}
