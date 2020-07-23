use cgmath::vec2;
use conrod_core::position::{Align, Place, Relative};
use conrod_core::render::PrimitiveWalker;
use conrod_core::widget::{self, bordered_rectangle, matrix, BorderedRectangle, Matrix, List};
use conrod_core::widget::Canvas;
use conrod_core::widget_ids;
use conrod_core::Ui;
use conrod_core::{Color, Colorable, Positionable, Sizeable, Widget, Borderable, Theme};
use conrod_core::color;
use ref_thread_local::RefThreadLocal;

use crate::log;
use crate::gadget::Agent;
use crate::render::{Model, ModelType, ShaderType, TrianglesEx, TrianglesType, MODELS};
use crate::render::{SHADERS, TRIANGLESES};
use crate::widget::{screen, Button, ContraptionScreen, SelectionGrid, Triangles3d};
use crate::App;

widget_ids! {
    pub struct WidgetIds {
        rect, contraption_screen, menu, menu_list, gadget_select, agent,
        canvas, header, body, left_sidebar,
    }
}

pub fn theme() -> Theme {
    Theme {
        background_color: color::TRANSPARENT,
        shape_color: color::TRANSPARENT,
        border_color: color::TRANSPARENT,
        ..Theme::default()
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
        for id in [
            self.ids.canvas, self.ids.header, self.ids.body, self.ids.left_sidebar,
            self.ids.rect, self.ids.contraption_screen, self.ids.menu, self.ids.menu_list,
            self.ids.gadget_select, self.ids.agent
        ].iter() {
            log!("Id: {:?}, Capturing: {}", id, ui.widget_input(*id).mouse().is_some());
        }
        
        let mut ui = ui.set_widgets();

        // Contraption screen
        for event in ContraptionScreen::new(self.mode, &self.camera)
            .middle_of(ui.window)
            .wh_of(ui.window)
            //.x_y(0.0, 0.0)
            //.wh_of(ui.window)
            .set(self.ids.contraption_screen, &mut ui)
        {
            match event {
                screen::Event::TilePaint(xy) => {
                    if let Some(gadget) = &self.gadget_tile {
                        // Nope gadget is special
                        if gadget.def().num_states() == 1
                            && gadget.def().num_ports() == 0
                            && gadget.size() == (1, 1)
                        {
                            self.grid.remove(xy);
                        } else {
                            self.grid.insert(gadget.clone(), xy, gadget.size());
                        }
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

                screen::Event::Pan(xy) => {
                    self.center += xy;
                }
            }
        }

        let new_canvas = || {
            Canvas::new().graphics_for(self.ids.contraption_screen)
        };

        new_canvas().flow_down(&[
            (self.ids.header, new_canvas().length(40.0)),
            (self.ids.body, new_canvas().flow_right(&[
                (self.ids.left_sidebar, new_canvas().length(260.0))
            ]))
        ]).set(self.ids.canvas, &mut ui);

        // Menu
        BorderedRectangle::new([1.0, 1.0])
            .with_style(bordered_rectangle::Style {
                color: Some(Color::Rgba(0.9, 0.9, 0.9, 1.0)),
                border: None,
                border_color: Some(color::BLACK),
            })
            .middle_of(self.ids.header)
            .wh_of(self.ids.header)
            .set(self.ids.menu, &mut ui);

        let (mut items, _) = List::flow_right(2)
            .middle_of(self.ids.menu)
            .wh_of(self.ids.menu)
            .set(self.ids.menu_list, &mut ui);
            
        for _ in items.next(&ui).unwrap().set(
            Button::triangles(Triangles3d::from_gadget(&self.gadget_select_rep))
                    .padding(3.0)
                    .w(ui.h_of(self.ids.menu_list).expect("No menu list!"))
                    .h_of(self.ids.menu_list),
            &mut ui
        ) {
            self.set_mode(Mode::TilePaint);
        }

        for _ in items.next(&ui).unwrap().set(
            Button::triangles(Triangles3d::new(
                (*TRIANGLESES.borrow()[TrianglesType::Agent])
                    .clone()
                    .with_default_extra(),
                vec2(0.0, 0.0),
                0.3,
                0.3,
            ))
                .padding(3.0)
                .w(ui.h_of(self.ids.menu_list).expect("No menu list!"))
                .h_of(self.ids.menu_list),
            &mut ui
        ) {
            self.set_mode(Mode::AgentPlace);
            self.agent = Some(Agent::new(
                vec2(0.5, 0.0),
                vec2(0, 1),
            ));
        }

        // Gadget selector
        if self.mode != Mode::Play {
            let selection = SelectionGrid::new(4, &self.gadget_select, self.gadget_selection)
                .color(Color::Rgba(0.8, 0.9, 0.8, 1.0))
                .border_color(color::BLACK)
                .outer_padding(5.0)
                .middle_of(self.ids.left_sidebar)
                .padded_wh_of(self.ids.left_sidebar, 10.0)
                .set(self.ids.rect, &mut ui);

            if let Some(selection) = selection {
                self.set_mode(Mode::TilePaint);
                self.gadget_selection = Some(selection);

                let gadget = self.gadget_select[selection].clone();
                self.gadget_tile = Some(gadget);
            }
        }
    }

    pub fn render_ui(&mut self, ui: &mut Ui, width: f64, height: f64) {
        self.ui_renderer.draw_begin(width, height);

        let mut primitives = ui.draw();
        while let Some(primitive) = PrimitiveWalker::next_primitive(&mut primitives) {
            self.ui_renderer.primitive(primitive);
        }

        self.ui_renderer.draw_end();
    }
}
