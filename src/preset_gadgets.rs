use std::rc::Rc;

use crate::gadget::{Gadget, GadgetDef, State};
use crate::spsp_multi;

pub fn preset_gadgets() -> Vec<Gadget> {
    let def = Rc::new(GadgetDef::new(1, 0));

    let nope = Gadget::new(&def, (1, 1), vec![], State(0)).name_this("Nope");

    let mut def = Rc::new(GadgetDef::from_traversals(
        1,
        2,
        spsp_multi![((0, 0), (0, 1)), ((0, 1), (0, 0))],
    ));

    let straight = Gadget::new(&def, (1, 1), vec![0, 2], State(0)).name_this("Straight");

    let turn = Gadget::new(&def, (1, 1), vec![0, 1], State(0)).name_this("Turn");

    def = Rc::new(GadgetDef::from_traversals(
        1,
        4,
        spsp_multi![
            ((0, 0), (0, 1)),
            ((0, 1), (0, 0)),
            ((0, 2), (0, 3)),
            ((0, 3), (0, 2)),
        ],
    ));

    let cross = Gadget::new(&def, (1, 1), vec![0, 2, 1, 3], State(0)).name_this("Cross");

    let turn2 = Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("2 turns");

    def = Rc::new(GadgetDef::from_traversals(
        1,
        3,
        spsp_multi![
            ((0, 0), (0, 1)),
            ((0, 1), (0, 0)),
            ((0, 1), (0, 2)),
            ((0, 2), (0, 1)),
            ((0, 2), (0, 0)),
            ((0, 0), (0, 2)),
        ],
    ));

    let way3 = Gadget::new(&def, (1, 1), vec![0, 1, 3], State(0)).name_this("3-way");

    def = Rc::new(GadgetDef::from_traversals(
        1,
        4,
        spsp_multi![
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

    let way4 = Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("4-way");

    def = Rc::new(GadgetDef::from_traversals(
        1,
        2,
        spsp_multi![((0, 0), (0, 1))],
    ));

    let diode = Gadget::new(&def, (1, 1), vec![0, 2], State(0)).name_this("Diode");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        2,
        spsp_multi![((0, 0), (1, 1)), ((1, 1), (0, 0))],
    ));

    let toggle = Gadget::new(&def, (1, 1), vec![0, 2], State(0)).name_this("Toggle");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        2,
        spsp_multi![((0, 0), (1, 1))],
    ));

    let dicrumbler = Gadget::new(&def, (1, 1), vec![0, 2], State(0)).name_this("Directed crumbler");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        2,
        spsp_multi![((0, 0), (1, 1)), ((0, 1), (1, 0))],
    ));

    let crumbler = Gadget::new(&def, (1, 1), vec![0, 2], State(0)).name_this("Crumbler");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        3,
        spsp_multi![((0, 0), (1, 0)), ((1, 1), (0, 2))],
    ));

    let scd = Gadget::new(&def, (1, 1), vec![0, 3, 1], State(0)).name_this("Self-closing door");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        spsp_multi![
            ((0, 0), (1, 1)),
            ((1, 1), (0, 0)),
            ((0, 2), (1, 3)),
            ((1, 3), (0, 2)),
        ],
    ));

    let toggle2 = Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("2-toggle");

    def = Rc::new(GadgetDef::from_traversals(
        3,
        4,
        spsp_multi![
            ((0, 0), (1, 1)),
            ((1, 1), (0, 0)),
            ((0, 2), (2, 3)),
            ((2, 3), (0, 2)),
        ],
    ));

    let lock_toggle_2 =
        Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("Locking 2-toggle");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        spsp_multi![((0, 0), (1, 1)), ((1, 2), (0, 3))],
    ));

    let mismatched_dicrumbler =
        Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("Mismatched dicrumblers");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        spsp_multi![
            ((0, 0), (1, 1)),
            ((0, 1), (1, 0)),
            ((1, 2), (0, 3)),
            ((1, 3), (0, 2)),
        ],
    ));

    let mismatched_crumbler =
        Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("Mismatched crumblers");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        spsp_multi![((0, 0), (1, 1)), ((0, 2), (1, 3))],
    ));

    let matched_dicrumbler =
        Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("Matched dicrumblers");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        spsp_multi![
            ((0, 0), (1, 1)),
            ((0, 1), (1, 0)),
            ((0, 2), (1, 3)),
            ((0, 3), (1, 2)),
        ],
    ));

    let matched_crumbler =
        Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("Matched crumblers");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        spsp_multi![
            ((0, 0), (1, 1)),
            ((1, 1), (0, 0)),
            ((0, 2), (0, 3)),
            ((0, 3), (0, 2)),
        ],
    ));

    let toggle_lock =
        Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("Toggle lock");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        spsp_multi![
            ((0, 0), (1, 1)),
            ((1, 1), (0, 0)),
            ((0, 1), (1, 0)),
            ((1, 0), (0, 1)),
            ((0, 2), (0, 3)),
            ((0, 3), (0, 2)),
        ],
    ));

    let tripwire_lock =
        Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("Tripwire lock");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        spsp_multi![
            ((0, 0), (1, 1)),
            ((1, 1), (0, 0)),
            ((0, 1), (1, 0)),
            ((1, 0), (0, 1)),
            ((0, 2), (1, 3)),
            ((1, 3), (0, 2)),
        ],
    ));

    let tripwire_toggle =
        Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("Tripwire toggle");

    def = Rc::new(GadgetDef::from_traversals(
        2,
        6,
        spsp_multi![
            ((0, 0), (1, 1)),
            ((0, 2), (0, 3)),
            ((1, 0), (1, 1)),
            ((1, 2), (0, 3)),
            ((1, 4), (1, 5))
        ],
    ));

    let door = Gadget::new(&def, (2, 1), vec![4, 5, 1, 2, 0, 3], State(0)).name_this("Door");

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
        lock_toggle_2,
        mismatched_dicrumbler,
        mismatched_crumbler,
        matched_dicrumbler,
        matched_crumbler,
        toggle_lock,
        tripwire_lock,
        tripwire_toggle,
        door,
    ]
}
