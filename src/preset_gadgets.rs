use std::rc::Rc;

use crate::gadget::{Gadget, GadgetDef};

pub fn preset_gadgets() -> Vec<Gadget> {
    let mut def = Rc::new(GadgetDef::new(1, 0));

    let nope = Gadget::new(&def, (1, 1), vec![], 0);

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

    def = Rc::new(GadgetDef::from_traversals(1, 2, vec![((0, 0), (0, 1))]));

    let diode = Gadget::new(&def, (1, 1), vec![Some(0), None, Some(1), None], 0);

    def = Rc::new(GadgetDef::from_traversals(
        2,
        2,
        vec![((0, 0), (1, 1)), ((1, 1), (0, 0))],
    ));

    let toggle = Gadget::new(&def, (1, 1), vec![Some(0), None, Some(1), None], 0);

    def = Rc::new(GadgetDef::from_traversals(2, 2, vec![((0, 0), (1, 1))]));

    let dicrumbler = Gadget::new(&def, (1, 1), vec![Some(0), None, Some(1), None], 0);

    def = Rc::new(GadgetDef::from_traversals(
        2,
        2,
        vec![((0, 0), (1, 1)), ((0, 1), (1, 0))],
    ));

    let crumbler = Gadget::new(&def, (1, 1), vec![Some(0), None, Some(1), None], 0);

    def = Rc::new(GadgetDef::from_traversals(
        2,
        3,
        vec![((0, 0), (1, 0)), ((1, 1), (0, 2))],
    ));

    let scd = Gadget::new(&def, (1, 1), vec![Some(0), Some(2), None, Some(1)], 0);

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        vec![
            ((0, 0), (1, 1)),
            ((1, 1), (0, 0)),
            ((0, 2), (1, 3)),
            ((1, 3), (0, 2)),
        ],
    ));

    let toggle2 = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), Some(2), Some(3)], 0);

    def = Rc::new(GadgetDef::from_traversals(
        3,
        4,
        vec![
            ((0, 0), (1, 1)),
            ((1, 1), (0, 0)),
            ((0, 2), (2, 3)),
            ((2, 3), (0, 2)),
        ],
    ));

    let lockToggle2 = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), Some(2), Some(3)], 0);

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        vec![((0, 0), (1, 1)), ((1, 2), (0, 3))],
    ));

    let mismatchedDicrumbler =
        Gadget::new(&def, (1, 1), vec![Some(0), Some(1), Some(2), Some(3)], 0);

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        vec![
            ((0, 0), (1, 1)),
            ((0, 1), (1, 0)),
            ((1, 2), (0, 3)),
            ((1, 3), (0, 2)),
        ],
    ));

    let mismatchedCrumbler = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), Some(2), Some(3)], 0);

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        vec![((0, 0), (1, 1)), ((0, 2), (1, 3))],
    ));

    let matchedDicrumbler = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), Some(2), Some(3)], 0);

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        vec![
            ((0, 0), (1, 1)),
            ((0, 1), (1, 0)),
            ((0, 2), (1, 3)),
            ((0, 3), (1, 2)),
        ],
    ));

    let matchedCrumbler = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), Some(2), Some(3)], 0);

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        vec![
            ((0, 0), (1, 1)),
            ((1, 1), (0, 0)),
            ((0, 2), (0, 3)),
            ((0, 3), (0, 2)),
        ],
    ));

    let toggleLock = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), Some(2), Some(3)], 0);

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        vec![
            ((0, 0), (1, 1)),
            ((1, 1), (0, 0)),
            ((0, 1), (1, 0)),
            ((1, 0), (0, 1)),
            ((0, 2), (0, 3)),
            ((0, 3), (0, 2)),
        ],
    ));

    let tripwireLock = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), Some(2), Some(3)], 0);

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        vec![
            ((0, 0), (1, 1)),
            ((1, 1), (0, 0)),
            ((0, 1), (1, 0)),
            ((1, 0), (0, 1)),
            ((0, 2), (1, 3)),
            ((1, 3), (0, 2)),
        ],
    ));

    let tripwireToggle = Gadget::new(&def, (1, 1), vec![Some(0), Some(1), Some(2), Some(3)], 0);

    vec![
        nope,
        straight,
        turn,
        cross,
        turn2,
        way3,
        way4,
        diode,
        toggle,
        dicrumbler,
        crumbler,
        scd,
        toggle2,
        lockToggle2,
        mismatchedDicrumbler,
        mismatchedCrumbler,
        matchedDicrumbler,
        matchedCrumbler,
        toggleLock,
        tripwireLock,
        tripwireToggle,
    ]
}