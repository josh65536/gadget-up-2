use cgmath::{vec2};
use fnv::{FnvHashMap, FnvHashSet};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::{Cell, Ref, RefCell};
use std::fmt::{self, Debug, Formatter};
use std::rc::Rc;

use crate::grid::{Grid, GridItem, WH, XY};

use crate::math::{Vec2, Vec2i, Vector2Ex};

use crate::render::{GadgetRenderInfo};


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Port(pub usize);

impl Port {
    /// Gets the id of this port
    pub fn id(self) -> usize {
        self.0
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct State(pub usize);

impl State {
    /// Gets the id of this state
    pub fn id(self) -> usize {
        self.0
    }
}

/// Type for (state, port) combinations
pub type SP = (State, Port);

/// Type for (port, port) traversals
pub type PP = (Port, Port);

/// Type for ((state, port), (state, port)) traversals
pub type SPSP = (SP, SP);

/// Helper macro for defining (port, port) combinations.
#[allow(unused_macros)]
macro_rules! sp_multi {
    ($(($s:expr, $p:expr)),* $(,)?) => {
        [$(($crate::gadget::State($s), $crate::gadget::Port($p))),*].iter().copied()
    };
}

/// Helper macro for defining (state, port) traversals.
#[allow(unused_macros)]
macro_rules! pp_multi {
    ($(($p0:expr, $p1:expr)),* $(,)?) => {
        [$(($crate::gadget::Port($p0), $crate::gadget::Port($p1))),*].iter().copied()
    };
}

/// Helper macro for defining traversals.
/// Returns an iterator of ((state, port), (state, port)) traversals.
#[macro_export]
macro_rules! spsp_multi {
    ($((($s0:expr, $p0:expr), ($s1:expr, $p1:expr))),* $(,)?) => {
        [$((
            ($crate::gadget::State($s0), $crate::gadget::Port($p0)),
            ($crate::gadget::State($s1), $crate::gadget::Port($p1)),
        )),*].iter().copied()
    };
}

/// Definition of a gadget, including ports, states, and transitions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GadgetDef {
    num_states: usize,
    num_ports: usize,
    traversals: FnvHashSet<SPSP>,
}

impl GadgetDef {
    /// Constructs the "nope" gadget
    pub fn new(num_states: usize, num_ports: usize) -> Self {
        Self {
            num_states,
            num_ports,
            traversals: FnvHashSet::default(),
        }
    }

    /// Checks if all the invariants are satisfied:
    ///
    /// * At least 1 state exists.
    /// * The states and ports of the traversals are not out of bounds.
    fn is_valid(&self) -> bool {
        self.num_states > 0
            && self.traversals.iter().all(|((s0, p0), (s1, p1))| {
                s0.0 < self.num_states
                    && s1.0 < self.num_states
                    && p0.0 < self.num_ports
                    && p1.0 < self.num_ports
            })
    }

    pub fn from_traversals<I: IntoIterator<Item = SPSP>>(
        num_states: usize,
        num_ports: usize,
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

    /// Gets all the destinations allowed in some state and port
    pub fn targets_from_state_port<'a>(&'a self, sp: SP) -> impl Iterator<Item = SP> + 'a {
        self.traversals
            .iter()
            .filter(move |((s, p), _)| *s == sp.0 && *p == sp.1)
            .map(move |(_, (s, p))| (*s, *p))
    }

    /// Gets all the port-to-port traversals allowed in some state
    pub fn port_traversals_in_state(&self, state: State) -> FnvHashSet<PP> {
        self.traversals
            .iter()
            .filter(|((s, _), _)| *s == state)
            .map(|((_, p0), (_, p1))| (*p0, *p1))
            .collect()
    }
}

/// Gadget that can be serialized and deserialized.
/// Instead of the gadget def, it contains an index
/// into a list of gadget defs.
/// No rendering info is stored.
#[derive(Serialize, Deserialize, Debug)]
pub struct GadgetSerde {
    def: usize,
    size: WH,
    port_map: Vec<usize>,
    state: State,
}

pub struct Gadget {
    name: String,
    def: Rc<GadgetDef>,
    size: WH,
    /// Ports are located at midpoints of unit segments along the perimeter,
    /// starting from the bottom left and going counterclockwise.
    /// This map maps each port to its position index along the perimeter.
    port_map: Vec<usize>,
    state: State,
    render: RefCell<GadgetRenderInfo>,
    dirty: Cell<bool>,
}

impl Gadget {
    /// Constructs a new `Gadget` with a gadget definition, a size,
    /// and a port map.
    ///
    /// Ports are located at midpoints of unit segments along the perimeter,
    /// starting from the bottom left and going counterclockwise.
    /// The port map maps each port to its position index along the perimeter.
    pub fn new(def: &Rc<GadgetDef>, size: WH, port_map: Vec<usize>, state: State) -> Self {
        let res = Self {
            name: String::new(),
            def: Rc::clone(def),
            size,
            port_map,
            state,
            render: RefCell::new(GadgetRenderInfo::new()),
            dirty: Cell::new(true),
        };
        res
    }

    /// Gives this gadget a name and returns it back
    pub fn name_this(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// Gets the gadget's name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the serializable form of this gadget.
    /// Index instead of reference to definition; no render info
    /// If the def is not in the list yet, it is added to the list.
    ///
    /// `defs` is a list of gadget defs.
    /// `defs_inv` is an "inverse Vec" of gadget defs.
    pub fn get_serializable(
        &self,
        defs: &mut Vec<Rc<GadgetDef>>,
        defs_inv: &mut FnvHashMap<*const GadgetDef, usize>,
    ) -> GadgetSerde {
        let index = if let Some(index) = defs_inv.get(&(&*self.def as *const GadgetDef)) {
            *index
        } else {
            defs.push(Rc::clone(&self.def));
            defs_inv.insert(&*self.def as *const GadgetDef, defs.len() - 1);
            defs_inv.len() - 1
        };

        GadgetSerde {
            def: index,
            size: self.size,
            port_map: self.port_map.clone(),
            state: self.state,
        }
    }

    pub fn from_serializable(gadget: GadgetSerde, defs: &Vec<Rc<GadgetDef>>) -> Self {
        Self::new(
            &defs[gadget.def],
            gadget.size,
            gadget.port_map,
            gadget.state,
        )
    }

    pub fn def(&self) -> &Rc<GadgetDef> {
        &self.def
    }

    pub fn size(&self) -> WH {
        self.size
    }

    pub fn perimeter(&self) -> usize {
        2 * self.size.0 + 2 * self.size.1
    }

    pub fn port(&self, index: usize) -> Option<Port> {
        self.port_map
            .iter()
            .position(|n| *n == index)
            .map(|n| Port(n))
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn set_state(&mut self, state: State) {
        self.state = state;
        self.dirty.set(true);
    }

    /// Gets the traversals allowed in the current state, at some port
    /// in back, right, front, left order relative to some facing direction
    pub fn targets_from_state_port_brfl(&self, port: Port, direction: XY) -> [Vec<SP>; 4] {
        let offset = if direction.x == 0 {
            if direction.y > 0 {
                0
            } else {
                2
            }
        } else {
            if direction.x > 0 {
                1
            } else {
                3
            }
        };

        let mut arr = [vec![], vec![], vec![], vec![]];
        let (w, h) = self.size();
        let map = &self.port_map;

        for sp in self.def().targets_from_state_port((self.state(), port)) {
            let (_, port) = sp;
            let idx = map[port.0 as usize];

            if idx < w + h {
                if idx < w {
                    &mut arr[(0 + offset) % 4]
                } else {
                    &mut arr[(1 + offset) % 4]
                }
            } else {
                if idx < w + h + w {
                    &mut arr[(2 + offset) % 4]
                } else {
                    &mut arr[(3 + offset) % 4]
                }
            }
            .push(sp);
        }

        arr
    }

    fn potential_port_positions(&self) -> Vec<Vec2> {
        (0..self.size.0)
            .map(|i| vec2(0.5 + i as f64, 0.0))
            .chain((0..self.size.1).map(|i| vec2(self.size.0 as f64, 0.5 + i as f64)))
            .chain(
                (0..self.size.0)
                    .rev()
                    .map(|i| vec2(0.5 + i as f64, self.size.1 as f64)),
            )
            .chain((0..self.size.1).rev().map(|i| vec2(0.0, 0.5 + i as f64)))
            .collect()
    }

    /// Rotates the ports of the gadget by some number of spaces.
    /// A positive number means counterclockwise,
    /// a negative number means clockwise.
    pub fn rotate_ports(&mut self, num_spaces: i32) {
        self.dirty.set(true);
        let perimeter = self.perimeter();
        let rem = (num_spaces).rem_euclid(perimeter as i32);

        for idx in self.port_map.iter_mut() {
            *idx = (*idx + rem as usize).rem_euclid(perimeter)
        }
    }

    /// Rotates the gadget by some number of 90-degree turns.
    /// A positive number means counterclockwise,
    /// a negative number means clockwise.
    pub fn rotate(&mut self, num_turns: i32) {
        self.dirty.set(true);

        for _ in 0..num_turns.rem_euclid(4) {
            self.rotate_ports(self.size.1 as i32);
            std::mem::swap(&mut self.size.0, &mut self.size.1);
        }
    }

    /// Flips ports across the x axis
    pub fn flip_ports_x(&mut self) {
        self.dirty.set(true);
        let perimeter = self.perimeter() as isize;

        for idx in self.port_map.iter_mut() {
            *idx = (self.size.0 as isize - *idx as isize - 1).rem_euclid(perimeter) as usize
        }
    }

    /// Flips ports across the y axis
    pub fn flip_ports_y(&mut self) {
        self.dirty.set(true);
        let perimeter = self.perimeter() as isize;

        for idx in self.port_map.iter_mut() {
            *idx = (perimeter - self.size.1 as isize - *idx as isize - 1).rem_euclid(perimeter)
                as usize
        }
    }

    /// Twists the bottom-right ports
    pub fn twist_bottom_right(&mut self) {
        self.dirty.set(true);

        let t0 = self.size.0 - 1;
        let t1 = self.size.0;

        for idx in self.port_map.iter_mut() {
            *idx = match *idx {
                _ if *idx == t0 => t1,
                _ if *idx == t1 => t0,
                _ => *idx,
            }
        }
    }

    /// Adds 1 to the state; resetting it to 0 in case of overflow
    pub fn cycle_state(&mut self) {
        self.dirty.set(true);
        self.set_state(State((self.state.0 + 1) % self.def.num_states()));
    }

    /// Gets the positions of the ports of this gadget in port order.
    /// The positions are relative to the bottom-left corner.
    pub fn port_positions(&self) -> Vec<Vec2> {
        let potential = self.potential_port_positions();

        self.port_map.iter().map(|n| potential[*n]).collect()
    }

    /// Updates the rendering information
    pub fn update_render(&self) {
        self.render.borrow_mut().update(self);
    }

    pub fn renderer(&self) -> Ref<GadgetRenderInfo> {
        if self.dirty.get() {
            self.dirty.set(false);
            self.update_render();
        }
        self.render.borrow()
    }
}

impl GridItem for Gadget {
    fn rotate_in_grid(mut self, turns: isize) -> Self {
        self.rotate(turns as i32);
        self
    }

    fn flip_x_in_grid(mut self) -> Self {
        self.flip_ports_x();
        self
    }

    fn flip_y_in_grid(mut self) -> Self {
        self.flip_ports_y();
        self
    }
}

impl Clone for Gadget {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            def: Rc::clone(&self.def),
            size: self.size,
            port_map: self.port_map.clone(),
            state: self.state,
            render: self.render.clone(),
            dirty: self.dirty.clone(),
        }
    }
}

impl Debug for Gadget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Gadget")
            .field("name", &self.name)
            .field("def", &self.def)
            .field("size", &self.size)
            .field("port_map", &self.port_map)
            .field("state", &self.state)
            .finish()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GadgetGridSerde {
    defs: Vec<GadgetDef>,
    gadgets: Vec<(GadgetSerde, (isize, isize))>,
}

impl GadgetGridSerde {
    /// Is a no-op if this is valid,
    /// but returns an error otherwise.
    fn validate<'de, D: Deserializer<'de>>(self) -> Result<Self, D::Error> {
        use serde::de::Error;

        for def in &self.defs {
            if !def.is_valid() {
                return Err(D::Error::custom(&format!(
                    "Gadget def {:?} is not valid",
                    def
                )));
            }
        }

        // Now validate the gadgets
        for (gadget, _) in &self.gadgets {
            // index must be in bounds
            let def = self.defs.get(gadget.def).ok_or_else(|| {
                D::Error::custom(&format!(
                    "Gadget {:?} is not valid because its def is out of bounds",
                    gadget
                ))
            })?;

            // size must be positive
            if gadget.size.0 == 0 || gadget.size.1 == 0 {
                return Err(D::Error::custom(&format!(
                    "Gadget {:?} with def {:?} is not valid because its size is not positive",
                    gadget, def
                )));
            }

            // state must be in bounds
            if gadget.state.0 >= def.num_states() {
                return Err(D::Error::custom(&format!(
                    "Gadget {:?} with def {:?} is not valid because its state is out of bounds",
                    gadget, def
                )));
            }

            // port map must be the right size and in bounds, and also be a 1-to-1 map
            if gadget.port_map.len() != def.num_ports() {
                return Err(D::Error::custom(&format!("Gadget {:?} with def {:?} is not valid because its port map is not the right size", gadget, def)));
            }

            if gadget
                .port_map
                .iter()
                .any(|i| *i >= 2 * gadget.size.0 + 2 * gadget.size.1)
            {
                return Err(D::Error::custom(&format!("Gadget {:?} with def {:?} is not valid because a port position index is out of bounds", gadget, def)));
            }

            if gadget.port_map.iter().collect::<FnvHashSet<_>>().len() != gadget.port_map.len() {
                return Err(D::Error::custom(&format!("Gadget {:?} with def {:?} is not valid because 2 ports map to the same position", gadget, def)));
            }
        }

        Ok(self)
    }
}

impl Serialize for Grid<Gadget> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut defs = vec![];
        let mut defs_inv = FnvHashMap::default();
        let mut gadgets = vec![];

        for (gadget, xy, _wh) in self.iter() {
            let gadget = gadget.get_serializable(&mut defs, &mut defs_inv);
            gadgets.push((gadget, (xy.x, xy.y)));
        }

        let grid_serde = GadgetGridSerde {
            defs: defs.into_iter().map(|def| (*def).clone()).collect(),
            gadgets,
        };

        grid_serde.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Grid<Gadget> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let GadgetGridSerde { defs, gadgets } =
            GadgetGridSerde::deserialize(deserializer)?.validate::<D>()?;

        let defs = defs.into_iter().map(|def| Rc::new(def)).collect();

        let mut grid = Self::new();

        for (gadget, (x, y)) in gadgets.into_iter() {
            let gadget = Gadget::from_serializable(gadget, &defs);
            let size = gadget.size;
            grid.insert(gadget, vec2(x, y), size);
        }

        Ok(grid)
    }
}

/// Walks around in a maze of gadgets
pub struct Agent {
    /// Double the position, because then it's integers
    double_xy: XY,
    /// either (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), or (0.0, -1.0)
    direction: Vec2i,
}

impl Agent {
    pub fn new(position: Vec2, direction: Vec2i) -> Self {
        let double_xy = vec2(
            (position.x * 2.0).round() as isize,
            (position.y * 2.0).round() as isize,
        );

        Self {
            double_xy,
            direction,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.double_xy.cast::<f64>().unwrap() * 0.5
    }

    pub fn direction(&self) -> Vec2i {
        self.direction
    }

    pub fn set_position(&mut self, position: Vec2) {
        // Also make sure the direction is valid
        let old_x_misaligned = self.double_xy.x.rem_euclid(2) != 0;

        self.double_xy = vec2(
            (position.x * 2.0).round() as isize,
            (position.y * 2.0).round() as isize,
        );

        let new_x_misaligned = self.double_xy.x.rem_euclid(2) != 0;

        if old_x_misaligned && !new_x_misaligned {
            self.direction = self.direction.right_ccw();
        } else if !old_x_misaligned && new_x_misaligned {
            self.direction = self.direction.right_cw();
        }
    }

    //pub fn rotate(&mut self, num_right_turns: i32) {
    //    for _ in 0..(num_right_turns.rem_euclid(4)) {
    //        self.direction = self.direction.right_ccw();
    //    }
    //}

    /// Flips the agent so it faces the opposite direction
    pub fn flip(&mut self) {
        self.direction = -self.direction
    }

    /// Advances the agent according to internal rules.
    /// Returns a reference to the gadget, its position, and its previous state
    /// if a gadget changed state as a result.
    pub fn advance<'a>(
        &mut self,
        grid: &'a mut Grid<Gadget>,
        input: Vec2i,
    ) -> Option<(&'a Gadget, XY, State)> {
        if input.dot_ex(self.direction) == -1 {
            // Turn around, that's it
            self.direction *= -1;
            return None;
        }

        if let Some((gadget, xy, (_w, _h), idx)) =
            grid.get_item_touching_edge_mut(self.double_xy, self.direction)
        {
            if let Some(port) = gadget.port(idx) {
                let [back, right, front, left] =
                    gadget.targets_from_state_port_brfl(port, self.direction);

                // TODO: Make this more sophisticated; don't just take the first traversal

                let sp;

                if input.dot_ex(self.direction) == 1 {
                    // Forward
                    sp = front
                        .first()
                        .or_else(|| left.first().xor(right.first()))
                        .or_else(|| back.first());
                } else if self.direction.right_ccw() == input {
                    // Left
                    sp = left.first();
                } else if input.dot_ex(self.direction) == -1 {
                    // Back
                    // TODO: Unreachable right now
                    sp = None;
                } else {
                    // Right
                    sp = right.first();
                }

                if let Some((s1, p1)) = sp {
                    // No floor necessary because this becomes an integer
                    // when multiplied by 2
                    let pos2 = (gadget.port_positions()[p1.0] * 2.0)
                        .cast::<isize>()
                        .unwrap();
                    self.direction = if pos2.x.rem_euclid(2) != 0 {
                        if pos2.y == 0 {
                            // Bottom
                            vec2(0, -1)
                        } else {
                            // Top
                            vec2(0, 1)
                        }
                    } else {
                        if pos2.x == 0 {
                            // Left
                            vec2(-1, 0)
                        } else {
                            // Right
                            vec2(1, 0)
                        }
                    };

                    self.double_xy = xy * 2 + pos2;
                    let state = gadget.state();
                    gadget.set_state(*s1);

                    return Some((gadget, xy, state));
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nope() {
        let def = GadgetDef::new(4, 3);
        assert_eq!(3, def.num_ports());
        assert_eq!(4, def.num_states());
        assert_eq!(0, def.traversals().count());
    }

    #[test]
    fn test_from_traversals() {
        let def = GadgetDef::from_traversals(2, 2, spsp_multi![((0, 0), (1, 1)), ((1, 1), (0, 0))]);
        assert_eq!(2, def.num_ports());
        assert_eq!(2, def.num_states());

        let expected = spsp_multi![((0, 0), (1, 1)), ((1, 1), (0, 0))].collect::<FnvHashSet<_>>();
        let result = def.traversals().copied().collect::<FnvHashSet<_>>();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_targets_from_state_port_empty() {
        let def = GadgetDef::from_traversals(2, 2, spsp_multi![((0, 0), (1, 1)), ((1, 1), (0, 0))]);

        let expected = sp_multi![].collect::<FnvHashSet<_>>();
        let result = def
            .targets_from_state_port((State(0), Port(1)))
            .collect::<FnvHashSet<_>>();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_targets_from_state_port_one() {
        let def = GadgetDef::from_traversals(2, 2, spsp_multi![((0, 0), (1, 1)), ((1, 1), (0, 0))]);

        let expected = sp_multi![(0, 0)].collect::<FnvHashSet<_>>();
        let result = def
            .targets_from_state_port((State(1), Port(1)))
            .collect::<FnvHashSet<_>>();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_targets_from_state_port_multiple() {
        let def = GadgetDef::from_traversals(2, 2, spsp_multi![((0, 0), (1, 1)), ((0, 0), (0, 1))]);

        let expected = sp_multi![(1, 1), (0, 1)].collect::<FnvHashSet<_>>();
        let result = def
            .targets_from_state_port((State(0), Port(0)))
            .collect::<FnvHashSet<_>>();
        assert_eq!(result, expected);
    }

    #[test]
    fn test_port_traversals_in_state_empty() {
        let def = GadgetDef::from_traversals(
            4,
            4,
            spsp_multi![
                ((0, 0), (1, 1)),
                ((2, 0), (3, 1)),
                ((0, 2), (2, 3)),
                ((1, 2), (3, 3)),
            ],
        );

        let expected = pp_multi![].collect::<FnvHashSet<_>>();
        let result = def.port_traversals_in_state(State(3));
        assert_eq!(result, expected)
    }

    #[test]
    fn test_port_traversals_in_state_one() {
        let def = GadgetDef::from_traversals(
            4,
            4,
            spsp_multi![
                ((0, 0), (1, 1)),
                ((2, 0), (3, 1)),
                ((0, 2), (2, 3)),
                ((1, 2), (3, 3)),
            ],
        );

        let expected = pp_multi![(2, 3)].collect::<FnvHashSet<_>>();
        let result = def.port_traversals_in_state(State(1));
        assert_eq!(result, expected)
    }

    #[test]
    fn test_port_traversals_in_state_multiple() {
        let def = GadgetDef::from_traversals(
            4,
            4,
            spsp_multi![
                ((0, 0), (1, 1)),
                ((2, 0), (3, 1)),
                ((0, 2), (2, 3)),
                ((1, 2), (3, 3)),
            ],
        );

        let expected = pp_multi![(0, 1), (2, 3)].collect::<FnvHashSet<_>>();
        let result = def.port_traversals_in_state(State(0));
        assert_eq!(result, expected)
    }

    fn assert_gadget_def_valid(def: &GadgetDef) {
        assert!(def.is_valid(), "Gadget def {:?} is not valid", def);
    }

    #[test]
    fn test_gadget_def_valid() {
        let def = GadgetDef::new(1, 0);
        assert_gadget_def_valid(&def);

        let def = GadgetDef::from_traversals(2, 3, spsp_multi![((0, 0), (1, 1)), ((1, 2), (0, 0))]);
        assert_gadget_def_valid(&def);

        let def = GadgetDef::from_traversals(3, 2, spsp_multi![((0, 0), (1, 1)), ((2, 1), (0, 0))]);
        assert_gadget_def_valid(&def);
    }

    #[test]
    #[should_panic(expected = "is not valid")]
    fn test_gadget_def_invalid_no_states() {
        let def = GadgetDef::new(0, 3);
        assert_gadget_def_valid(&def);
    }

    #[test]
    #[should_panic(expected = "is not valid")]
    fn test_gadget_def_out_of_bounds_state() {
        let def = GadgetDef::from_traversals(2, 3, spsp_multi![((0, 0), (1, 1)), ((2, 1), (0, 0))]);
        assert_gadget_def_valid(&def);
    }

    #[test]
    #[should_panic(expected = "is not valid")]
    fn test_gadget_def_out_of_bounds_port() {
        let def = GadgetDef::from_traversals(3, 2, spsp_multi![((0, 0), (1, 1)), ((1, 2), (0, 0))]);
        assert_gadget_def_valid(&def);
    }

    fn assert_gadget_grid_serde_valid<'de>(grid: GadgetGridSerde) {
        grid.validate::<&mut crate::bit_serde::Deserializer<'de>>()
            .unwrap();
    }

    #[test]
    fn test_gadget_grid_serde_valid() {
        let defs = vec![GadgetDef::new(2, 3)];
        let grid = GadgetGridSerde {
            defs,
            gadgets: vec![(
                GadgetSerde {
                    def: 0,
                    size: (1, 1),
                    port_map: vec![0, 3, 1],
                    state: State(1),
                },
                (1, -1),
            )],
        };
        assert_gadget_grid_serde_valid(grid);
    }

    #[test]
    #[should_panic(expected = "is not valid because its def is out of bounds")]
    fn test_gadget_grid_serde_invalid_out_of_bounds_def() {
        let defs = vec![GadgetDef::new(2, 3)];
        let grid = GadgetGridSerde {
            defs,
            gadgets: vec![(
                GadgetSerde {
                    def: 1,
                    size: (1, 1),
                    port_map: vec![0, 3, 1],
                    state: State(1),
                },
                (1, -1),
            )],
        };
        assert_gadget_grid_serde_valid(grid);
    }

    #[test]
    #[should_panic(expected = "is not valid because its size is not positive")]
    fn test_gadget_grid_serde_invalid_zero_width() {
        let defs = vec![GadgetDef::new(2, 3)];
        let grid = GadgetGridSerde {
            defs,
            gadgets: vec![(
                GadgetSerde {
                    def: 0,
                    size: (0, 1),
                    port_map: vec![0, 3, 1],
                    state: State(1),
                },
                (1, -1),
            )],
        };
        assert_gadget_grid_serde_valid(grid);
    }

    #[test]
    #[should_panic(expected = "is not valid because its size is not positive")]
    fn test_gadget_grid_serde_invalid_zero_height() {
        let defs = vec![GadgetDef::new(2, 3)];
        let grid = GadgetGridSerde {
            defs,
            gadgets: vec![(
                GadgetSerde {
                    def: 0,
                    size: (1, 0),
                    port_map: vec![0, 3, 1],
                    state: State(1),
                },
                (1, -1),
            )],
        };
        assert_gadget_grid_serde_valid(grid);
    }

    #[test]
    #[should_panic(expected = "is not valid because its state is out of bounds")]
    fn test_gadget_grid_serde_invalid_out_of_bounds_state() {
        let defs = vec![GadgetDef::new(2, 3)];
        let grid = GadgetGridSerde {
            defs,
            gadgets: vec![(
                GadgetSerde {
                    def: 0,
                    size: (1, 1),
                    port_map: vec![0, 3, 1],
                    state: State(2),
                },
                (1, -1),
            )],
        };
        assert_gadget_grid_serde_valid(grid);
    }

    #[test]
    #[should_panic(expected = "is not valid because its port map is not the right size")]
    fn test_gadget_grid_serde_invalid_port_map_small() {
        let defs = vec![GadgetDef::new(2, 3)];
        let grid = GadgetGridSerde {
            defs,
            gadgets: vec![(
                GadgetSerde {
                    def: 0,
                    size: (1, 1),
                    port_map: vec![0, 3],
                    state: State(1),
                },
                (1, -1),
            )],
        };
        assert_gadget_grid_serde_valid(grid);
    }

    #[test]
    #[should_panic(expected = "is not valid because its port map is not the right size")]
    fn test_gadget_grid_serde_invalid_port_map_big() {
        let defs = vec![GadgetDef::new(2, 3)];
        let grid = GadgetGridSerde {
            defs,
            gadgets: vec![(
                GadgetSerde {
                    def: 0,
                    size: (1, 1),
                    port_map: vec![0, 3, 1, 2],
                    state: State(1),
                },
                (1, -1),
            )],
        };
        assert_gadget_grid_serde_valid(grid);
    }

    #[test]
    #[should_panic(expected = "is not valid because a port position index is out of bounds")]
    fn test_gadget_grid_serde_invalid_port_map_out_of_bounds() {
        let defs = vec![GadgetDef::new(2, 3)];
        let grid = GadgetGridSerde {
            defs,
            gadgets: vec![(
                GadgetSerde {
                    def: 0,
                    size: (1, 1),
                    port_map: vec![0, 4, 1],
                    state: State(1),
                },
                (1, -1),
            )],
        };
        assert_gadget_grid_serde_valid(grid);
    }

    #[test]
    #[should_panic(expected = "is not valid because 2 ports map to the same position")]
    fn test_gadget_grid_serde_invalid_port_map_not_one_to_one() {
        let defs = vec![GadgetDef::new(2, 3)];
        let grid = GadgetGridSerde {
            defs,
            gadgets: vec![(
                GadgetSerde {
                    def: 0,
                    size: (1, 1),
                    port_map: vec![0, 3, 3],
                    state: State(1),
                },
                (1, -1),
            )],
        };
        assert_gadget_grid_serde_valid(grid);
    }
}
