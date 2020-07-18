use conrod_core::position::{Align, Place, Relative};
use conrod_core::render::PrimitiveWalker;
use conrod_core::widget::{self, bordered_rectangle, BorderedRectangle, matrix, Matrix};
use conrod_core::widget_ids;
use conrod_core::{Color, Colorable, Positionable, Sizeable, Widget};
use conrod_core::{Ui, UiBuilder};
use cgmath::vec2;

use crate::log;
use crate::gadget::Agent;
use crate::widget::{screen, ContraptionScreen, SelectionGrid, Triangles3d, Button};
use crate::App;

widget_ids! {
    pub struct WidgetIds {
        rect, contraption_screen, menu, agent,
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
// Mode of editing
pub enum Mode {
    None,
    TilePaint,
    AgentPlace,
    Play,
}

impl App {
    const MENU_HEIGHT: f64 = 40.0;

    pub fn set_mode(&mut self, mode: Mode) {
        if mode != self.mode {
            // clear some fields
            if mode != Mode::TilePaint {
                self.gadget_selection = None;
                self.gadget_tile = None;
            }

            if mode != Mode::AgentPlace && mode != Mode::Play {
                self.agent = None;
            }

            self.mode = mode;
        }
    }

    pub fn update_ui(&mut self, ui: &mut Ui) {
        let mut ui = ui.set_widgets();

        for event in ContraptionScreen::new(self.mode, &self.camera)
            .x_y(0.0, 0.0)
            .wh_of(ui.window)
            .set(self.ids.contraption_screen, &mut ui)
        {
            match event {
                screen::Event::TilePaint(xy) => {
                    if let Some(gadget) = &self.gadget_tile {
                        self.grid.insert(gadget.clone(), xy, gadget.size());
                    }
                }

                screen::Event::TileHover(xy) => {
                    self.gadget_tile_xy = xy;
                }

                screen::Event::AgentPlace(xy) => {
                    if self.agent.is_some() {
                        self.set_mode(Mode::Play);
                    }

                    if let Some(agent) = &mut self.agent {
                        agent.set_position(xy);
                    }
                }

                screen::Event::AgentHover(xy) => {
                    if let Some(agent) = &mut self.agent {
                        agent.set_position(xy);
                    }
                }

                _ => {}
            }
        }

        // Menu
        BorderedRectangle::new([1.0, 1.0])
            .with_style(bordered_rectangle::Style {
                color: Some(Color::Rgba(0.9, 0.9, 0.9, 1.0)),
                border: None,
                border_color: None,
            })
            .w_of(ui.window)
            .h(App::MENU_HEIGHT)
            .x(0.0)
            .y_position_relative_to(ui.window, Relative::Align(Align::End))
            .set(self.ids.menu, &mut ui);


        //while let Some(element) = Matrix::new(2, 1)
        //    .x(3.0)
        //    .y(0.0)
        //    .w((App::MENU_HEIGHT - 6.0) * 2.0)
        //    .h(34.0)
        //    .set(self.ids.menu_mtx, &mut ui).next(&mut ui)
        //{
        {
            let positions: Vec<f32> = vec![
                0.1, -0.1, 0.0, 0.1, 0.0, 0.0, 0.0, 0.1, 0.0, -0.1, 0.0, 0.0, -0.1, -0.1, 0.0,
            ];
            let colors: Vec<f32> = vec![
                0.0, 0.8, 0.0, 1.0, 0.0, 0.6, 0.0, 1.0, 0.0, 0.4, 0.0, 1.0, 0.0, 0.6, 0.0, 1.0, 0.0,
                0.8, 0.0, 1.0,
            ];
            let indexes: Vec<u32> = vec![0, 1, 2, 0, 2, 4, 2, 3, 4];

            for _ in Button::triangles(Triangles3d::new(positions, colors, indexes, vec2(0.0, 0.0), 0.2, 0.2))
                .x_position_relative_to(self.ids.menu, Relative::Place(Place::Start(Some(3.0))))
                .y_position_relative_to(self.ids.menu, Relative::Place(Place::Start(Some(3.0))))
                .w(App::MENU_HEIGHT - 6.0)
                .h(34.0)
                .set(self.ids.agent, &mut ui)
            {
                self.set_mode(Mode::AgentPlace);
                self.agent = Some(Agent::new(self.agent_position, vec2(0, 1), &self.agent_model));
            }
        }
        //}

        // Gadget selector
        if self.mode != Mode::Play {
            let selection = SelectionGrid::new(4, &self.gadget_select, self.gadget_selection)
                .color(Color::Rgba(0.8, 0.9, 0.8, 1.0))
                .outer_padding(5.0)
                .x_position_relative_to(ui.window, Relative::Place(Place::Start(Some(10.0))))
                .y_position_relative_to(ui.window, Relative::Place(Place::Start(Some(10.0))))
                .w(300.0)
                .padded_h_of(ui.window, App::MENU_HEIGHT / 2.0 + 10.0)
                .set(self.ids.rect, &mut ui);

            if let Some(selection) = selection {
                self.set_mode(Mode::TilePaint);
                self.gadget_selection = Some(selection);
                self.gadget_tile = Some(self.gadget_select[selection].clone());
            }
        }
    }

    pub fn render_ui(&mut self, ui: &mut Ui, width: f32, height: f32) {
        self.ui_renderer.draw_begin(width, height);

        let mut primitives = ui.draw();
        while let Some(primitive) = PrimitiveWalker::next_primitive(&mut primitives) {
            self.ui_renderer.primitive(primitive);
        }

        self.ui_renderer.draw_end();
    }
}
