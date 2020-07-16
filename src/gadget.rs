use cgmath::vec2;
use fnv::{FnvHashMap, FnvHashSet};
use std::cell::{Ref, RefCell};
use std::rc::Rc;
use three_d::Vec2;

use crate::math::Vector2Ex;
use crate::grid::WH;
use crate::log;
use crate::shape::{Circle, Path, Rectangle, Shape};

pub type Port = u32;
pub type State = u32;

/// Type for (state, port) combinations
pub type SP = (Port, State);

/// Type for (port, port) traversals
pub type PP = (Port, Port);

/// Type for ((state, port), (state, port)) traversals
pub type SPSP = (SP, SP);

/// Definition of a gadget, including ports, states, and transitions
pub struct GadgetDef {
    num_ports: usize,
    num_states: usize,
    traversals: FnvHashSet<SPSP>,
}

impl GadgetDef {
    /// Constructs the "nope" gadget
    pub fn new(num_ports: usize, num_states: usize) -> Self {
        Self {
            num_ports,
            num_states,
            traversals: FnvHashSet::default(),
        }
    }

    pub fn from_traversals<I: IntoIterator<Item = SPSP>>(
        num_ports: usize,
        num_states: usize,
        traversals: I,
    ) -> Self {
        Self {
            num_ports,
            num_states,
            traversals: traversals.into_iter().collect(),
        }
    }

    pub fn num_ports(&self) -> usize {
        self.num_ports
    }

    pub fn num_states(&self) -> usize {
        self.num_states
    }

    pub fn traversals(&self) -> impl Iterator<Item = &SPSP> {
        self.traversals.iter()
    }

    /// Gets all the port-to-port traversals allowed in some state
    pub fn port_traversals_in_state(&self, state: State) -> FnvHashSet<PP> {
        self.traversals.iter().filter(|((s, _), _)| *s == state).map(|((_, p0), (_, p1))| (*p0, *p1)).collect()
    }
}

pub struct Gadget {
    def: Rc<GadgetDef>,
    size: WH,
    /// Ports are located at midpoints of unit segments along the perimeter,
    /// starting from the bottom left and going counterclockwise.
    port_map: Vec<Option<Port>>,
    state: State,
    render: RefCell<GadgetRenderInfo>,
}

impl Gadget {
    /// Constructs a new `Gadget` with a gadget definition, a size,
    /// and a port map.
    ///
    /// Ports are located at midpoints of unit segments along the perimeter,
    /// starting from the bottom left and going counterclockwise. In the port map,
    /// a `None` represents the absence of a port.
    pub fn new(def: Rc<GadgetDef>, size: WH, port_map: Vec<Option<Port>>, state: State) -> Self {
        let res = Self {
            def,
            size,
            port_map,
            state,
            render: RefCell::new(GadgetRenderInfo::new()),
        };
        res.update_render();
        res
    }

    pub fn def(&self) -> &Rc<GadgetDef> {
        &self.def
    }

    pub fn size(&self) -> WH {
        self.size
    }

    pub fn state(&self) -> State {
        self.state
    }

    fn potential_port_positions(&self) -> Vec<Vec2> {
        (0..self.size.0)
            .map(|i| vec2(0.5 + i as f32, 0.0))
            .chain((0..self.size.1).map(|i| vec2(self.size.0 as f32, 0.5 + i as f32)))
            .chain(
                (0..self.size.0)
                    .rev()
                    .map(|i| vec2(0.5 + i as f32, self.size.1 as f32)),
            )
            .chain((0..self.size.1).rev().map(|i| vec2(0.0, 0.5 + i as f32)))
            .collect()
    }

    /// Gets the positions of the ports of this gadget in port order.
    /// The positions are relative to the bottom-left corner.
    pub fn port_positions(&self) -> Vec<Vec2> {
        let mut vec = Vec::new();
        vec.resize(self.def.num_ports, vec2(0.0, 0.0));

        let x: f32 = 0.0;
        let y: f32 = 0.0;

        for (port, position) in self.port_map.iter().zip(self.potential_port_positions()) {
            if let Some(port) = port {
                vec[*port as usize] = position;
            }
        }

        vec
    }

    /// Updates the rendering information
    pub fn update_render(&self) {
        self.render.borrow_mut().update(self);
    }

    pub fn renderer(&self) -> Ref<GadgetRenderInfo> {
        self.render.borrow()
    }
}

pub struct GadgetRenderInfo {
    /// 3 coordinates per position
    positions: Vec<f32>,
    /// 3 components per color
    colors: Vec<f32>,
    /// 3 indexes per triangle
    indexes: Vec<u32>,
    paths: FnvHashMap<PP, Path>,
}

impl GadgetRenderInfo {
    pub const RECTANGLE_Z: f32 = -0.001;
    const OUTLINE_Z: f32 = -0.05;
    const PATH_Z: f32 = -0.075;
    const PORT_Z: f32 = -0.1;

    fn new() -> Self {
        Self {
            positions: vec![],
            colors: vec![],
            indexes: vec![],
            paths: FnvHashMap::default(),
        }
    }

    fn has_outline(&self, gadget: &Gadget) -> bool {
        gadget.def().num_states() > 1
    }

    /// Gets the path a robot takes to go from p0 to p1
    fn port_path(ports: PP, port_positions: &Vec<Vec2>) -> Path {
        let positions = [
            port_positions[ports.0 as usize],
            port_positions[ports.1 as usize],
        ];
        let mut bezier = [vec2(0f32, 0f32), vec2(0f32, 0f32)];

        let offset = 0.25f32;

        for (pos, bez) in positions.iter().zip(bezier.iter_mut()) {
            *bez = pos + if pos.x.floor() == pos.x { // on vertical edge
                if pos.x == 0.0 { // on left edge
                    vec2(offset, 0.0)
                } else { // on right edge
                    vec2(-offset, 0.0)
                }
            } else { // on horizontal edge
                if pos.y == 0.0 { // on bottom edge
                    vec2(0.0, offset)
                } else { // on top edge
                    vec2(0.0, -offset)
                }
            }
        }

        // Same-port traversal; make it look like a loop
        if bezier[0] == bezier[1] {
            let dv = (bezier[0] - positions[0]).right_ccw();

            bezier[0] += dv;
            bezier[1] -= dv;
        }

        Path::from_bezier3(
            [positions[0], bezier[0], bezier[1], positions[1]],
            GadgetRenderInfo::PATH_Z,
            0.08,
        )
    }

    /// Updates the rendering information so
    /// that it is correct when rendering
    fn update(&mut self, gadget: &Gadget) {
        self.positions.clear();
        self.indexes.clear();

        // Surrounding rectangle
        let rect = Rectangle::new(0.0, gadget.size().0 as f32, 0.0, gadget.size().1 as f32, GadgetRenderInfo::RECTANGLE_Z);
        rect.append_to(&mut self.positions, &mut self.indexes);
        self.colors
            .extend(&[0.6, 0.8, 1.0, 0.7, 0.9, 1.0, 0.9, 1.0, 1.0, 0.8, 1.0, 1.0]);

        // Port circles
        let port_positions = gadget.port_positions();
        for vec in port_positions.iter() {
            let circle = Circle::new(vec.x, vec.y, GadgetRenderInfo::PORT_Z, 0.08);
            circle.append_to(&mut self.positions, &mut self.indexes);
            self.colors.extend(
                [0.0, 0.0, 0.5]
                    .iter()
                    .cycle()
                    .take(circle.num_vertices() * 3),
            );
        }

        // Outline
        if self.has_outline(gadget) {
            let path = Path::new(
                vec![
                    vec2(0.0, 0.0),
                    vec2(0.0, gadget.size().1 as f32),
                    vec2(gadget.size().0 as f32, gadget.size().1 as f32),
                    vec2(gadget.size().0 as f32, 0.0),
                ],
                GadgetRenderInfo::OUTLINE_Z,
                0.05,
                true,
            );
            path.append_to(&mut self.positions, &mut self.indexes);
            self.colors
                .extend([0.0, 0.0, 0.0].iter().cycle().take(path.num_vertices() * 3));
        }

        // Paths
        for ports in gadget.def().port_traversals_in_state(gadget.state()) {
            let path = GadgetRenderInfo::port_path(ports, &port_positions);

            path.append_to(&mut self.positions, &mut self.indexes);
            self.colors.extend(
                [0.0, 0.0, 0.0]
                    .iter()
                    .cycle()
                    .take(path.num_vertices() * 3),
            );

            self.paths.insert(ports, path);
        }
    }

    pub fn colors(&self) -> &Vec<f32> {
        &self.colors
    }
}

impl Shape for GadgetRenderInfo {
    fn num_vertices(&self) -> usize {
        self.positions().len() / 3
    }

    fn positions(&self) -> Vec<f32> {
        self.positions.clone()
    }

    fn indexes(&self) -> Vec<u32> {
        self.indexes.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nope() {
        let def = GadgetDef::new(3, 4);
        assert_eq!(3, def.num_ports());
        assert_eq!(4, def.num_states());
        assert_eq!(0, def.traversals().count());
    }

    #[test]
    fn test_from_traversals() {
        let def = GadgetDef::from_traversals(2, 2, vec![((0, 0), (1, 1)), ((1, 1), (0, 0))]);
        assert_eq!(2, def.num_ports());
        assert_eq!(2, def.num_states());

        let mut expected = FnvHashSet::default();
        expected.insert(((0, 0), (1, 1)));
        expected.insert(((1, 1), (0, 0)));
        let result = def.traversals().copied().collect::<FnvHashSet<_>>();
        assert_eq!(result, expected);
    }
}
