use std::rc::Rc;
use ref_thread_local::RefThreadLocal;

use crate::gadget::{Gadget, GadgetDef, State};
use crate::render::lang::{GRLS, Grl};
use crate::{spsp_multi, grl};

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

    // Now for more interesting stuff
    let mut renderers = vec![];

    def = Rc::new(GadgetDef::from_traversals(
        2,
        2,
        spsp_multi![((0, 0), (1, 1)), ((1, 1), (0, 0))],
    ));

    let toggle = Gadget::new(&def, (1, 1), vec![0, 2], State(0)).name_this("Toggle");

    let sqrt_half: f64 = 0.5f64.sqrt();

    renderers.push((Rc::clone(&def), grl!(
        { (rect ((0 => 1, 0.5) + (z Grl::Z)), ((0 => 1, 0.5; dir sqrt_half, sqrt_half) + (z Grl::Z)), 0.15, 0.15) }
        { (rect ((0 => 1, 0.5) + (z Grl::Z)), ((0 => 1, 0.5; dir sqrt_half, sqrt_half) + (z Grl::Z)), 0.15, 0.15) }
    ), false));

    def = Rc::new(GadgetDef::from_traversals(
        2,
        2,
        spsp_multi![((0, 0), (1, 1))],
    ));

    let dicrumbler = Gadget::new(&def, (1, 1), vec![0, 2], State(0)).name_this("Directed crumbler");

    renderers.push((Rc::clone(&def), {
        let size = 0.15 / 2.0;
        grl!(
        {
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid),
        }
        {
            (path (port_path 0 => 1, 0.0 => 1.0, Grl::Z), dotted, |>, fade),
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid, fade),
        }
    )}, false));

    def = Rc::new(GadgetDef::from_traversals(
        2,
        2,
        spsp_multi![((0, 0), (1, 1)), ((0, 1), (1, 0))],
    ));

    let crumbler = Gadget::new(&def, (1, 1), vec![0, 2], State(0)).name_this("Crumbler");

    renderers.push((Rc::clone(&def), {
        let size = 0.15 / 2.0;
        grl!(
        {
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid),
        }
        {
            (path (port_path 0 => 1, 0.0 => 1.0, Grl::Z), dotted, fade),
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid, fade),
        }
    )}, false));

    def = Rc::new(GadgetDef::from_traversals(
        2,
        3,
        spsp_multi![((0, 0), (1, 0)), ((1, 1), (0, 2))],
    ));

    let scd = Gadget::new(&def, (1, 1), vec![0, 3, 1], State(0)).name_this("Self-closing door");

    renderers.push((Rc::clone(&def), {
        let size = 0.15 / 2.0;
        grl!(
        {
            (path (port_path 1 => 2, 0.0 => 1.0, Grl::Z), dotted, |>, fade),
            (path (line ((1 => 2, 0.5; 1.0, size, size) + (z Grl::Z)) => ((1 => 2, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((1 => 2, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((1 => 2, 0.5; 1.0, size, -size) + (z Grl::Z))), solid, fade),
        }
        {
            (path (line ((1 => 2, 0.5; 1.0, size, size) + (z Grl::Z)) => ((1 => 2, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid),
            (path (line ((1 => 2, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((1 => 2, 0.5; 1.0, size, -size) + (z Grl::Z))), solid),
        }
    )}, false));

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

    renderers.push((Rc::clone(&def), grl!(
        { 
            (rect ((0 => 1, 0.5) + (z Grl::Z)), ((0 => 1, 0.5; dir sqrt_half, sqrt_half) + (z Grl::Z)), 0.15, 0.15),
            (rect ((2 => 3, 0.5) + (z Grl::Z)), ((2 => 3, 0.5; dir sqrt_half, sqrt_half) + (z Grl::Z)), 0.15, 0.15),
        }
        { 
            (rect ((0 => 1, 0.5) + (z Grl::Z)), ((0 => 1, 0.5; dir sqrt_half, sqrt_half) + (z Grl::Z)), 0.15, 0.15),
            (rect ((2 => 3, 0.5) + (z Grl::Z)), ((2 => 3, 0.5; dir sqrt_half, sqrt_half) + (z Grl::Z)), 0.15, 0.15),
        }
    ), false));

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

    renderers.push((Rc::clone(&def), {
        let size = 0.15 / 2.0;
        grl!(
        {
            (path (port_path 2 => 3, 0. => 1., Grl::Z), dotted, |>, fade),
            (path (line ((2 => 3, 0.5; 1.0, size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((2 => 3, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid),
        }
        {
            (path (port_path 0 => 1, 0. => 1., Grl::Z), dotted, |>, fade),
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((2 => 3, 0.5; 1.0, size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid),
            (path (line ((2 => 3, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, size, -size) + (z Grl::Z))), solid),
        }
    )}, false));

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

    renderers.push((Rc::clone(&def), {
        let size = 0.15 / 2.0;
        grl!(
        {
            (path (port_path 2 => 3, 0. => 1., Grl::Z), dotted, fade),
            (path (line ((2 => 3, 0.5; 1.0, size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((2 => 3, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid),
        }
        {
            (path (port_path 0 => 1, 0. => 1., Grl::Z), dotted, fade),
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((2 => 3, 0.5; 1.0, size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid),
            (path (line ((2 => 3, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, size, -size) + (z Grl::Z))), solid),
        }
    )}, false));

    def = Rc::new(GadgetDef::from_traversals(
        2,
        4,
        spsp_multi![((0, 0), (1, 1)), ((0, 2), (1, 3))],
    ));

    let matched_dicrumbler =
        Gadget::new(&def, (1, 1), vec![0, 1, 2, 3], State(0)).name_this("Matched dicrumblers");

    renderers.push((Rc::clone(&def), {
        let size = 0.15 / 2.0;
        grl!(
        {
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid),
            (path (line ((2 => 3, 0.5; 1.0, size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid),
            (path (line ((2 => 3, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, size, -size) + (z Grl::Z))), solid),
        }
        {
            (path (port_path 0 => 1, 0. => 1., Grl::Z), dotted, |>, fade),
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid, fade),
            (path (port_path 2 => 3, 0. => 1., Grl::Z), dotted, |>, fade),
            (path (line ((2 => 3, 0.5; 1.0, size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((2 => 3, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, size, -size) + (z Grl::Z))), solid, fade),
        }
    )}, false));

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

    renderers.push((Rc::clone(&def), {
        let size = 0.15 / 2.0;
        grl!(
        {
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid),
            (path (line ((2 => 3, 0.5; 1.0, size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid),
            (path (line ((2 => 3, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, size, -size) + (z Grl::Z))), solid),
        }
        {
            (path (port_path 0 => 1, 0. => 1., Grl::Z), dotted, fade),
            (path (line ((0 => 1, 0.5; 1.0, size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((0 => 1, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, size, -size) + (z Grl::Z))), solid, fade),
            (path (port_path 2 => 3, 0. => 1., Grl::Z), dotted, fade),
            (path (line ((2 => 3, 0.5; 1.0, size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, -size, -size) + (z Grl::Z))), solid, fade),
            (path (line ((2 => 3, 0.5; 1.0, -size, size) + (z Grl::Z)) => ((2 => 3, 0.5; 1.0, size, -size) + (z Grl::Z))), solid, fade),
        }
    )}, false));

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

    renderers.push((Rc::clone(&def), {
        let size = 0.15 / 2.0;
        grl!(
        { 
            (circle ((2 => 3, 0.5) + (z Grl::Z - 0.0001)), size, (0.75, 0.85, 1.0, 1.0)),
            (path (circle ((2 => 3, 0.5) + (z Grl::Z - 0.0002)), size), solid),
            (rect ((0 => 1, 0.5) + (z Grl::Z)), ((0 => 1, 0.5; dir sqrt_half, sqrt_half) + (z Grl::Z)), 2.0 * size, 2.0 * size),
        }
        { 
            (circle ((2 => 3, 0.5) + (z Grl::Z - 0.0001)), size, (0.75, 0.85, 1.0, 1.0)),
            (path (circle ((2 => 3, 0.5) + (z Grl::Z - 0.0002)), size), solid, fade),
            (path (port_path 2 => 3, 0. => 1., Grl::Z), dotted, fade),
            (rect ((0 => 1, 0.5) + (z Grl::Z)), ((0 => 1, 0.5; dir sqrt_half, sqrt_half) + (z Grl::Z)), 2.0 * size, 2.0 * size),
        }
    )}, false));

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

    renderers.push((Rc::clone(&def), {
        let size = 0.15 / 2.0;
        grl!(
        { 
            (circle ((2 => 3, 0.5) + (z Grl::Z - 0.0001)), size, (0.75, 0.85, 1.0, 1.0)),
            (path (circle ((2 => 3, 0.5) + (z Grl::Z - 0.0002)), size), solid),
            (path (line ((0 => 1, 0.5; 1.0, size, 0.0) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, 0.0) + (z Grl::Z))), solid),
        }
        { 
            (circle ((2 => 3, 0.5) + (z Grl::Z - 0.0001)), size, (0.75, 0.85, 1.0, 1.0)),
            (path (circle ((2 => 3, 0.5) + (z Grl::Z - 0.0002)), size), solid, fade),
            (path (port_path 2 => 3, 0. => 1., Grl::Z), dotted, fade),
            (path (line ((0 => 1, 0.5; 1.0, size, 0.0) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, 0.0) + (z Grl::Z))), solid),
        }
    )}, false));

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

    renderers.push((Rc::clone(&def), {
        let size = 0.15 / 2.0;
        grl!(
        { 
            (path (line ((0 => 1, 0.5; 1.0, size, 0.0) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, 0.0) + (z Grl::Z))), solid),
            (rect ((2 => 3, 0.5) + (z Grl::Z)), ((2 => 3, 0.5; dir sqrt_half, sqrt_half) + (z Grl::Z)), 2.0 * size, 2.0 * size),
        }
        { 
            (path (line ((0 => 1, 0.5; 1.0, size, 0.0) + (z Grl::Z)) => ((0 => 1, 0.5; 1.0, -size, 0.0) + (z Grl::Z))), solid),
            (rect ((2 => 3, 0.5) + (z Grl::Z)), ((2 => 3, 0.5; dir sqrt_half, sqrt_half) + (z Grl::Z)), 2.0 * size, 2.0 * size),
        }
    )}, false));

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

    renderers.push((Rc::clone(&def), {
        let size = 0.15 / 2.0;
        grl!(
        { 
            (path (port_path 4 => 5, 0. => 1., Grl::Z), dotted, |>, fade),
            (path (port_path 0 => 1, 0. => 1., Grl::Z), solid, |>, (0.0, 0.5, 0.0, 1.0)),
            (path (port_path 2 => 3, 0. => 1., Grl::Z), solid, |>, (1.0, 0.0, 0.0, 1.0)),
        }
        { 
            (path (port_path 4 => 5, 0. => 1., Grl::Z), solid, |>),
            (path (port_path 0 => 1, 0. => 1., Grl::Z), solid, |>, (0.0, 0.7, 0.0, 1.0)),
            (path (port_path 2 => 3, 0. => 1., Grl::Z), solid, |>, (1.0, 0.0, 0.0, 1.0)),
        }
    )}, true));

    GRLS.borrow_mut().init(renderers);

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
