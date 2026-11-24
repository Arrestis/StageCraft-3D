// Copyright (C) 2026 Doug Rumble
// SPDX-License-Identifier: GPL-3.0-only

//! Collision and stage-bounds warnings for set pieces.

use crate::math::*;
use crate::model::*;
use crate::scene::*;

pub struct Warning {
    pub piece: usize,
    pub text: String,
}

fn sat_overlap(a: &[V3; 4], b: &[V3; 4]) -> bool {
    for poly in [a, b] {
        for i in 0..4 {
            let e = poly[(i + 1) % 4] - poly[i];
            let ax = v3(-e.z, 0.0, e.x);
            let proj = |p: &[V3; 4]| {
                let mut lo = f32::MAX;
                let mut hi = f32::MIN;
                for q in p {
                    let d = q.x * ax.x + q.z * ax.z;
                    lo = lo.min(d);
                    hi = hi.max(d);
                }
                (lo, hi)
            };
            let (a0, a1) = proj(a);
            let (b0, b1) = proj(b);
            let shrink = 0.01 * (ax.x.abs() + ax.z.abs());
            if a1 - shrink <= b0 || b1 - shrink <= a0 {
                return false;
            }
        }
    }
    true
}

pub fn check(pr: &Project, d: &Display) -> Vec<Warning> {
    let v = &pr.venue;
    let s = &pr.scenes[pr.current.min(pr.scenes.len() - 1)].state;
    let mut out = Vec::new();
    let bb = v.kind == VenueKind::BlackBox;
    let hw = v.stage_width() * 0.5;
    let pw = v.pros_width * 0.5;
    let house_hw = v.house_width * 0.5;
    let seat_z = v.house_start_z() + 0.5;
    let eye_y = -v.stage_height + 1.15;
    // Extreme seats: the ends of the front row (front rows are roughly as wide as the opening).
    let seat_x = (house_hw - 1.4).min(pw + 2.5);
    let seats = [v3(-seat_x, eye_y, seat_z), v3(seat_x, eye_y, seat_z)];
    let visible_from = |seat: V3, p: V3| -> bool {
        if p.z >= 0.0 {
            return true;
        }
        let t = seat.z / (seat.z - p.z);
        let x0 = seat.x + (p.x - seat.x) * t;
        x0.abs() <= pw
    };
    let ceiling = if v.has_fly_tower() {
        v.grid_height
    } else if v.open_end_stage {
        v.pros_height + 0.01
    } else {
        v.tower_height() - 1.0
    };

    let foot: Vec<_> = s.pieces.iter().map(|p| piece_footprint(p, v, d, &s.battens)).collect();

    for (i, p) in s.pieces.iter().enumerate() {
        if !p.visible {
            continue;
        }
        let (pts, bot, top) = foot[i];
        let mut warn = |t: String| out.push(Warning { piece: i, text: t });
        let flown = p.flown_on.is_some();
        let xs = pts.iter().map(|q| q.x);
        let zs = pts.iter().map(|q| q.z);
        let (x_lo, x_hi) = (xs.clone().fold(f32::MAX, f32::min), xs.fold(f32::MIN, f32::max));
        let (z_lo, z_hi) = (zs.clone().fold(f32::MAX, f32::min), zs.fold(f32::MIN, f32::max));

        if flown {
            if bot < v.deck_y(pts[0].z) - 0.01 {
                warn("Flown piece is lower than the deck – raise its batten".into());
            }
        } else {
            // Off the deck / over the pit.
            let (bx0, bx1, bz0, bz1) = if bb {
                (-house_hw, house_hw, -v.stage_depth, v.house_start_z() - 0.6)
            } else {
                (-hw, hw, -v.stage_depth, v.apron_depth)
            };
            let off = pts.iter().any(|q| q.x < bx0 - 0.01 || q.x > bx1 + 0.01 || q.z < bz0 - 0.01 || q.z > bz1 + 0.01);
            let on_pit = v.has_pit && pts.iter().any(|q| q.z > v.apron_depth && q.x.abs() < v.pit_width * 0.5);
            if on_pit && bot > d.pit + 0.05 {
                warn("Hangs over the orchestra pit".into());
            } else if off && !(on_pit && bot <= d.pit + 0.05) {
                warn("Extends off the stage deck".into());
            }
        }

        if !bb {
            // Wings and sightlines.
            let in_wings = pts.iter().any(|q| q.x.abs() > pw + 0.02 && q.z < 0.0);
            if in_wings {
                let seen = pts.iter().filter(|q| q.x.abs() > pw).any(|q| seats.iter().any(|s| visible_from(*s, *q)));
                if seen && !flown {
                    warn("In the wings but visible from side seats – needs masking".into());
                }
            } else {
                // Judge by the middle of the piece: corners of upstage pieces are often cut off for side seats.
                let c = (pts[0] + pts[1] + pts[2] + pts[3]) * 0.25;
                let hidden = c.z < 0.0 && seats.iter().any(|s| !visible_from(*s, c));
                if hidden {
                    warn("Outside sightlines – side seats can't see it".into());
                }
            }
            // Proscenium header from the front row (centre seat).
            if !flown {
                let near_z = z_hi.min(-0.05);
                let t = seat_z / (seat_z - near_z);
                let y0 = eye_y + (top - eye_y) * t;
                if y0 > v.pros_height && z_hi < 0.0 {
                    warn("Top is hidden behind the proscenium header from the front row".into());
                }
            }
            // Low room ceiling (open end stage, sloping soffit on stage left).
            if v.ceiling_at(0.0).is_some() && z_hi < 0.45 {
                let lowest = pts
                    .iter()
                    .filter_map(|q| v.ceiling_at(q.x))
                    .fold(f32::MAX, f32::min);
                if top > lowest - 0.02 {
                    warn(format!("Taller than the ceiling here ({:.2} m) – it won't fit", lowest));
                }
            }
        }

        // Curtain line and flown goods.
        let x_overlap = |half: f32| x_hi > -half && x_lo < half;
        for (bi, b) in s.battens.iter().enumerate() {
            if !b.visible || p.flown_on == Some(bi) {
                continue;
            }
            let bz = -b.dist;
            if !(z_lo < bz + 0.05 && z_hi > bz - 0.05) {
                continue;
            }
            let h = d.batten_h.get(bi).copied().unwrap_or(b.height);
            match b.kind {
                BattenKind::HouseCurtain => {
                    if x_overlap(pw) {
                        warn("Crosses the house curtain line – the curtain can't close".into());
                    }
                }
                BattenKind::Lighting | BattenKind::Scenery => {
                    if top > h - 0.4 && x_overlap(pw + 1.0) {
                        warn(format!("Too tall for {} at its current trim ({:.1} m)", b.name, h));
                    }
                }
                BattenKind::Legs | BattenKind::LegsAndBorder => {
                    let lb = (h - b.goods_h).max(0.0);
                    let leg_hit = (x_hi > pw - 0.25 && x_lo < pw + 1.15) || (x_lo < -pw + 0.25 && x_hi > -pw - 1.15);
                    if leg_hit && top > lb {
                        warn(format!("Collides with {} (legs)", b.name));
                    } else if b.kind == BattenKind::LegsAndBorder && top > h - 1.6 && x_overlap(pw + 1.0) {
                        warn(format!("Hits the border on {}", b.name));
                    }
                }
                _ => {
                    let lb = h - b.goods_h;
                    let closed_tabs = b.kind != BattenKind::Tabs || b.open < 0.5;
                    if top > lb && closed_tabs && x_overlap(pw + 1.0) {
                        warn(format!("Collides with {} at its current trim", b.name));
                    }
                }
            }
        }

        // Revolve.
        if v.has_revolve && !flown {
            let r = v.revolve_diameter * 0.5;
            let inside = |q: &V3| (q.x * q.x + (q.z - v.revolve_z).powi(2)).sqrt() <= r + 0.01;
            if p.on_revolve {
                if !pts.iter().all(inside) {
                    warn("Sticks out past the revolve edge – it will sweep into other things".into());
                }
            } else if pts.iter().all(inside) {
                warn("Sits on the revolve but isn't set to ride it".into());
            }
        }

        if top > ceiling {
            warn("Taller than the grid / ceiling".into());
        }

        // Overlaps with other pieces.
        for (j, q) in s.pieces.iter().enumerate() {
            if j <= i || !q.visible {
                continue;
            }
            let (qp, qb, qt) = foot[j];
            if bot >= qt - 0.02 || qb >= top - 0.02 {
                continue;
            }
            // Flats hinged together touch along the hinge by design.
            let hinged = |a: &Piece, b: &Piece| a.hinged_to.as_ref().map(|h| h.to == b.id).unwrap_or(false);
            if hinged(p, q) || hinged(q, p) {
                continue;
            }
            if sat_overlap(&pts, &qp) {
                let t = format!("Overlaps {}", q.name);
                out.push(Warning { piece: i, text: t });
                out.push(Warning { piece: j, text: format!("Overlaps {}", p.name) });
            }
        }
    }
    out
}
