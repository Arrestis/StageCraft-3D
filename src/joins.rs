// Copyright (C) 2026 Doug Rumble
// SPDX-License-Identifier: GPL-3.0-only

//! Flats hinged together. A flat hinged to another flat's edge stands against that edge, moves
//! with it and folds about the hinge line: book flats, folding screens, a door flat hinged to
//! a wall. The flat at the top of a chain is placed as usual; every flat hinged to it is placed
//! from its own settings (which edge, how far folded) each time anything changes.

use crate::library;
use crate::math::*;
use crate::model::*;

/// Library templates that are flats (and can be hinged together).
const FLAT_KEYS: [&str; 4] = ["flat", "doorflat", "windowflat", "arch"];

/// Can this piece carry hinges: a flat, wall, door or window flat, archway, or a hand-built
/// piece that is thin and wider than it is deep?
pub fn is_flat_like(p: &Piece) -> bool {
    match &p.template {
        Some(t) => FLAT_KEYS.contains(&t.key.as_str()),
        None => p.size[2] <= 0.5 && p.size[0] >= 2.0 * p.size[2],
    }
}

/// The panel of a flat in the piece's own space (after any stretch): its left and right edges,
/// the middle of its thickness and the thickness. Braces, trims and door leaves don't count.
#[derive(Clone, Copy, Debug)]
pub struct Panel {
    pub x0: f32,
    pub x1: f32,
    pub zc: f32,
    pub th: f32,
}

pub fn panel(p: &Piece) -> Panel {
    let whole = Panel { x0: -p.size[0] * 0.5, x1: p.size[0] * 0.5, zc: 0.0, th: p.size[2] };
    let Some(t) = &p.template else { return whole };
    if !t.params.iter().any(|q| q.key == "thick") {
        return whole;
    }
    let th = t.get("thick");
    let k = [0, 1, 2].map(|i| p.size[i] / p.base_size[i].max(1e-3));
    let (mut x0, mut x1, mut zc, mut found) = (f32::MAX, f32::MIN, 0.0, false);
    for q in &p.parts {
        // The panels are plain blocks in the piece's own colour, exactly as thick as the flat.
        let plain = q.hinge.is_none() && q.color.is_none() && q.rot == 0.0 && matches!(q.shape, Shape::Box | Shape::ArchTop);
        if !plain || (q.size[2] - th).abs() > 1e-3 {
            continue;
        }
        x0 = x0.min(q.offset[0] - q.size[0] * 0.5);
        x1 = x1.max(q.offset[0] + q.size[0] * 0.5);
        zc = q.offset[2];
        found = true;
    }
    if !found {
        return whole;
    }
    Panel { x0: x0 * k[0], x1: x1 * k[0], zc: zc * k[2], th: th * k[2] }
}

pub fn wrap_deg(a: f32) -> f32 {
    let mut a = a % 360.0;
    if a > 180.0 {
        a -= 360.0;
    }
    if a < -180.0 {
        a += 360.0;
    }
    a
}

/// The hinge line in each flat's own space: the point on the parent's edge, the matching point
/// on the hinged flat's edge, and which face the hinges sit on (+1 front, -1 back). Hinges go
/// on the inside of the fold, so the faces meet as it folds.
pub fn hinge_points(parent: &Piece, child: &Piece, j: &HingeJoin) -> (V3, V3, f32) {
    let s = if j.side >= 0 { 1.0 } else { -1.0 };
    let f = if j.angle >= 0.0 { 1.0 } else { -1.0 };
    let pp = panel(parent);
    let cp = panel(child);
    let pivot = v3(if s > 0.0 { pp.x1 } else { pp.x0 }, 0.0, pp.zc + f * pp.th * 0.5);
    let attach = v3(if s > 0.0 { cp.x0 } else { cp.x1 }, 0.0, cp.zc + f * cp.th * 0.5);
    (pivot, attach, f)
}

/// Where a hinged flat stands (x, height above the floor, z, in its parent's frame) and which
/// way it faces, given the flat it is hinged to.
pub fn place(parent: &Piece, child: &Piece, j: &HingeJoin) -> ([f32; 3], f32) {
    let s = if j.side >= 0 { 1.0 } else { -1.0 };
    let (pivot, attach, _) = hinge_points(parent, child, j);
    let rot = wrap_deg(parent.rot - s * j.angle.clamp(-180.0, 180.0));
    let at = v3(parent.pos[0], 0.0, parent.pos[2]) + pivot.rot_y(parent.rot) - attach.rot_y(rot);
    ([at.x, parent.pos[1], at.z], rot)
}

/// Stand every hinged flat against the edge it hangs from. Joins to flats that no longer exist
/// are dropped (the flat stays where it is). A few passes settle chains of hinged flats.
pub fn apply(s: &mut StageState) {
    if !s.pieces.iter().any(|p| p.hinged_to.is_some()) {
        return;
    }
    let n = s.pieces.len();
    for _ in 0..n.clamp(1, 24) {
        let mut moved = false;
        for i in 0..n {
            let Some(j) = s.pieces[i].hinged_to.clone() else { continue };
            let Some(pi) = s.pieces.iter().position(|q| q.id == j.to).filter(|&pi| pi != i) else {
                s.pieces[i].hinged_to = None;
                continue;
            };
            let (pos, rot) = place(&s.pieces[pi], &s.pieces[i], &j);
            let (rev, flown) = (s.pieces[pi].on_revolve, s.pieces[pi].flown_on);
            let c = &mut s.pieces[i];
            let d = (0..3).map(|k| (c.pos[k] - pos[k]).abs()).sum::<f32>() + wrap_deg(c.rot - rot).abs() * 0.01;
            if d > 1e-5 || c.on_revolve != rev || c.flown_on != flown {
                moved = true;
            }
            c.pos = pos;
            c.rot = rot;
            c.on_revolve = rev;
            c.flown_on = flown;
        }
        if !moved {
            break;
        }
    }
}

/// The index of the flat a piece is hinged to, if any.
pub fn parent_of(s: &StageState, idx: usize) -> Option<usize> {
    let j = s.pieces.get(idx)?.hinged_to.as_ref()?;
    s.pieces.iter().position(|q| q.id == j.to).filter(|&pi| pi != idx)
}

/// The flat at the top of a hinged chain (the one that isn't hinged to anything). Dragging any
/// flat in the group moves this one, and the rest follow.
pub fn root_of(s: &StageState, idx: usize) -> usize {
    let mut cur = idx;
    for _ in 0..64 {
        match parent_of(s, cur) {
            Some(pi) => cur = pi,
            None => break,
        }
    }
    cur
}

/// Does `idx` hang from `anc`, directly or further up its chain?
fn hangs_from(s: &StageState, idx: usize, anc: usize) -> bool {
    let mut cur = idx;
    for _ in 0..64 {
        match parent_of(s, cur) {
            Some(pi) if pi == anc => return true,
            Some(pi) => cur = pi,
            None => return false,
        }
    }
    false
}

/// Is this edge of the flat free for a hinge? (side -1 = left, +1 = right, seen from the front)
pub fn edge_free(s: &StageState, idx: usize, side: i8) -> bool {
    let Some(p) = s.pieces.get(idx) else { return false };
    if p.hinged_to.as_ref().map(|j| -j.side == side).unwrap_or(false) {
        return false;
    }
    !s.pieces.iter().any(|q| q.hinged_to.as_ref().map(|j| j.to == p.id && j.side == side).unwrap_or(false))
}

/// Can flat `child` be hinged to the `side` edge of flat `parent`?
pub fn can_join(s: &StageState, child: usize, parent: usize, side: i8) -> Result<(), &'static str> {
    let (Some(c), Some(p)) = (s.pieces.get(child), s.pieces.get(parent)) else {
        return Err("That piece isn't there any more");
    };
    if child == parent {
        return Err("A flat can't be hinged to itself");
    }
    if !is_flat_like(c) || !is_flat_like(p) {
        return Err("Only flats, walls and archways can be hinged together");
    }
    if hangs_from(s, parent, child) {
        return Err("That flat already carries this one");
    }
    if !edge_free(s, parent, side) {
        return Err("That edge already has a hinge");
    }
    // The joining flat uses its other edge: nothing may already hang there.
    if s.pieces.iter().any(|q| q.hinged_to.as_ref().map(|j| j.to == c.id && j.side == -side).unwrap_or(false)) {
        return Err("The other flat already has a flat hinged on that edge");
    }
    Ok(())
}

/// Hinge flat `child` to the `side` edge of flat `parent`. It keeps the angle the two stand
/// at now when that's a sensible fold, otherwise it starts in line.
pub fn join(s: &mut StageState, child: usize, parent: usize, side: i8) -> Result<(), &'static str> {
    can_join(s, child, parent, side)?;
    let (c, p) = (&s.pieces[child], &s.pieces[parent]);
    let mut angle = 0.0;
    if c.on_revolve == p.on_revolve {
        let a = wrap_deg(-(side as f32) * (c.rot - p.rot));
        if a.abs() <= 120.0 {
            angle = a.round();
        }
    }
    let to = p.id;
    s.pieces[child].hinged_to = Some(HingeJoin { to, side, angle });
    apply(s);
    Ok(())
}

/// Add a plain flat, as tall and thick as `parent`, hinged in line to its `side` edge.
/// Returns the new piece's index.
pub fn add_flat(s: &mut StageState, parent: usize, side: i8, id: u32, lib: &[LibItem]) -> Option<usize> {
    if !edge_free(s, parent, side) {
        return None;
    }
    let item = lib.iter().find(|i| i.template.as_ref().map(|t| t.key == "flat").unwrap_or(false))?;
    let par = s.pieces.get(parent)?.clone();
    let mut p = library::make_piece(item, id, par.pos);
    library::set_param(&mut p, "width", 1.2);
    library::set_param(&mut p, "height", par.size[1]);
    library::set_param(&mut p, "thick", panel(&par).th);
    p.name = "Hinged flat".into();
    p.material = par.material;
    p.color = par.color;
    p.hinged_to = Some(HingeJoin { to: par.id, side, angle: 0.0 });
    s.pieces.push(p);
    apply(s);
    Some(s.pieces.len() - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat(id: u32, w: f32) -> Piece {
        let lib = library::presets();
        let item = lib.iter().find(|i| i.name == "Flat 1.2 x 2.4 m").unwrap();
        let mut p = library::make_piece(item, id, [0.0, 0.0, 0.0]);
        library::set_param(&mut p, "width", w);
        p
    }

    fn state(pieces: Vec<Piece>) -> StageState {
        StageState { pieces, battens: vec![], lights: vec![], revolve_angle: 0.0, pit_lift: 0.0, stage_lift: 0.0, house_lights: 0.0, notes: String::new() }
    }

    #[test]
    fn in_line_flat_continues_the_wall() {
        let mut s = state(vec![flat(1, 2.0), flat(2, 1.0)]);
        join(&mut s, 1, 0, 1).unwrap();
        let c = &s.pieces[1];
        assert!((c.pos[0] - 1.5).abs() < 1e-4, "x = {}", c.pos[0]);
        assert!(c.pos[2].abs() < 1e-4);
        assert!(c.rot.abs() < 1e-4);
    }

    #[test]
    fn folding_forward_swings_towards_the_audience() {
        let mut s = state(vec![flat(1, 2.0), flat(2, 1.0)]);
        join(&mut s, 1, 0, 1).unwrap();
        s.pieces[1].hinged_to.as_mut().unwrap().angle = 90.0;
        apply(&mut s);
        let c = &s.pieces[1];
        // Its far edge comes downstage (+z) from the parent's right front corner.
        let far = v3(c.size[0] * 0.5, 0.0, 0.0).rot_y(c.rot) + v3(c.pos[0], 0.0, c.pos[2]);
        assert!(far.z > 0.9, "far edge z = {}", far.z);
        assert!((far.x - 1.0).abs() < 0.1, "far edge x = {}", far.x);
    }

    #[test]
    fn a_new_hinged_flat_matches_its_neighbour() {
        let lib = library::presets();
        let mut s = state(vec![flat(1, 2.0)]);
        library::set_param(&mut s.pieces[0], "height", 3.0);
        let i = add_flat(&mut s, 0, -1, 7, &lib).unwrap();
        let p = &s.pieces[i];
        assert!((p.size[1] - 3.0).abs() < 1e-4, "height {}", p.size[1]);
        assert!((p.pos[0] + 1.6).abs() < 1e-4, "x {}", p.pos[0]);
        assert_eq!(p.hinged_to.as_ref().map(|j| j.side), Some(-1));
        assert!(!edge_free(&s, 0, -1));
    }

    #[test]
    fn a_door_flat_hinges_on_its_panel_not_its_trim() {
        let lib = library::presets();
        let item = lib.iter().find(|i| i.name == "Door flat").unwrap();
        let p = library::make_piece(item, 1, [0.0; 3]);
        let pan = panel(&p);
        assert!((pan.x1 - pan.x0 - 2.2).abs() < 1e-3, "width {}", pan.x1 - pan.x0);
        assert!((pan.th - 0.08).abs() < 1e-4, "thickness {}", pan.th);
    }

    #[test]
    fn hinges_are_saved_and_older_projects_still_open() {
        let mut s = state(vec![flat(1, 2.0), flat(2, 1.0)]);
        join(&mut s, 1, 0, 1).unwrap();
        let txt = serde_json::to_string(&s.pieces[1]).unwrap();
        let back: Piece = serde_json::from_str(&txt).unwrap();
        assert_eq!(back.hinged_to, s.pieces[1].hinged_to);
        // A piece saved before hinges existed has no "hinged_to" at all.
        let mut v: serde_json::Value = serde_json::from_str(&txt).unwrap();
        v.as_object_mut().unwrap().remove("hinged_to");
        let old: Piece = serde_json::from_value(v).unwrap();
        assert!(old.hinged_to.is_none());
    }

    #[test]
    fn no_loops_and_one_hinge_per_edge() {
        let mut s = state(vec![flat(1, 2.0), flat(2, 1.0), flat(3, 1.0)]);
        join(&mut s, 1, 0, 1).unwrap();
        assert!(can_join(&s, 0, 1, 1).is_err(), "would make a loop");
        assert!(can_join(&s, 2, 0, 1).is_err(), "right edge is taken");
        assert!(can_join(&s, 2, 0, -1).is_ok());
        assert!(!edge_free(&s, 1, -1), "the hinged flat's left edge carries its hinge");
    }
}
