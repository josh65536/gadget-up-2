use std::rc::Rc;

use crate::gadget::{Gadget, GadgetDef};

pub fn preset_gadgets() -> Vec<Gadget> {
    let mut def = Rc::new(GadgetDef::from_traversals(
        1,
        2,
        vec![((0, 0), (0, 1)), ((0, 1), (0, 0))],
    ));

    let straight = Gadget::new(&def, (1, 1), vec![Some(0), None, Some(1), None], 0);

    let turn = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), None, None], 0);

    def = Rc::new(GadgetDef::from_traversals(
        1,
        4,
        vec![
            ((0, 0), (0, 1)),
            ((0, 1), (0, 0)),
            ((0, 2), (0, 3)),
            ((0, 3), (0, 2)),
        ],
    ));

    let cross = Gadget::new(&def, (1, 1), vec![Some(0), Some(2), Some(1), Some(3)], 0);

    let turn2 = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), Some(2), Some(3)], 0);

    def = Rc::new(GadgetDef::from_traversals(
        1,
        3,
        vec![
            ((0, 0), (0, 1)),
            ((0, 1), (0, 0)),
            ((0, 1), (0, 2)),
            ((0, 2), (0, 1)),
            ((0, 2), (0, 0)),
            ((0, 0), (0, 2)),
        ],
    ));

    let way3 = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), None, Some(2)], 0);

    def = Rc::new(GadgetDef::from_traversals(
        1,
        4,
        vec![
            ((0, 0), (0, 1)),
            ((0, 1), (0, 0)),
            ((0, 1), (0, 2)),
            ((0, 2), (0, 1)),
            ((0, 2), (0, 0)),
            ((0, 0), (0, 2)),
            ((0, 0), (0, 3)),
            ((0, 3), (0, 0)),
            ((0, 1), (0, 3)),
            ((0, 3), (0, 1)),
            ((0, 2), (0, 3)),
            ((0, 3), (0, 2)),
        ],
    ));

    let way4 = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), Some(2), Some(3)], 0);

    vec![straight, turn, cross, turn2, way3, way4]
}
