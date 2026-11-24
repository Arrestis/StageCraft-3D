// Copyright (C) 2026 Doug Rumble
// SPDX-License-Identifier: GPL-3.0-only

//! Turns the project (venue + current scene state) into triangles, lines, lights and labels.

use crate::math::*;
use crate::mesh::*;
use crate::model::*;
use crate::raster::*;
use egui::Color32;

pub const OBJ_PIECE: u32 = 1 << 24;
pub const OBJ_BATTEN: u32 = 2 << 24;
pub const OBJ_LIGHT: u32 = 3 << 24;
pub const OBJ_REVOLVE: u32 = 4 << 24;
pub const OBJ_PIT: u32 = 5 << 24;
pub const OBJ_LIFT: u32 = 6 << 24;
pub const OBJ_DRAFT: u32 = 7 << 24;
pub const OBJ_MASK: u32 = 0xFF << 24;

/// Animated values actually shown (they glide towards the scene's targets).
#[derive(Clone, Default)]
pub struct Display {
    pub batten_h: Vec<f32>,
    pub batten_open: Vec<f32>,
    pub revolve: f32,
    pub pit: f32,
    pub lift: f32,
}

pub struct BuildOpts {
    pub show_beams: bool,
    pub plan_view: bool,
    pub work_mode: bool,
    pub show_sightlines: bool,
    pub show_labels: bool,
    pub show_rigging: bool,
    pub selected: u32,
    pub draft: Option<Piece>,
    pub feet: bool,
}

pub struct Label {
    pub pos: V3,
    pub text: String,
    pub color: Color32,
}

pub fn fmt_len(m: f32, feet: bool) -> String {
    if feet {
        let total_in = (m / 0.0254).round() as i64;
        let neg = total_in < 0;
        let t = total_in.abs();
        format!("{}{}' {}\"", if neg { "-" } else { "" }, t / 12, t % 12)
    } else {
        format!("{:.2} m", m)
    }
}

fn in_rect(x: f32, z: f32, cx: f32, cz: f32, w: f32, d: f32) -> bool {
    (x - cx).abs() <= w * 0.5 && (z - cz).abs() <= d * 0.5
}

/// Height of whatever a piece at (x, z) stands on.
pub fn support_y(v: &Venue, d: &Display, x: f32, z: f32) -> f32 {
    let hw = v.stage_width() * 0.5;
    if v.has_stage_lift && in_rect(x, z, v.lift_x, v.lift_z, v.lift_width, v.lift_depth) {
        return v.deck_y(v.lift_z) + d.lift;
    }
    if v.has_revolve && ((x * x + (z - v.revolve_z).powi(2)).sqrt() <= v.revolve_diameter * 0.5) {
        return v.deck_y(z) + 0.04;
    }
    if z <= v.apron_depth && z >= -v.stage_depth && x.abs() <= hw {
        return v.deck_y(z);
    }
    if v.has_pit && z > v.apron_depth && z <= v.apron_depth + v.pit_depth && x.abs() <= v.pit_width * 0.5 {
        return d.pit;
    }
    -v.stage_height
}

/// World base position and rotation of a piece.
pub fn piece_world(p: &Piece, v: &Venue, d: &Display, battens: &[Batten]) -> (V3, f32) {
    let mut x = p.pos[0];
    let mut z = p.pos[2];
    let mut rot = p.rot;
    if p.on_revolve && v.has_revolve {
        let l = v3(x, 0.0, z).rot_y(d.revolve);
        x = l.x;
        z = l.z + v.revolve_z;
        rot += d.revolve;
    }
    if let Some(bi) = p.flown_on {
        if let Some(b) = battens.get(bi) {
            let h = d.batten_h.get(bi).copied().unwrap_or(b.height);
            return (v3(x, h - p.size[1], -b.dist), rot);
        }
    }
    let y = support_y(v, d, x, z) + p.pos[1];
    (v3(x, y, z), rot)
}

/// World-space footprint corners (x, z) and the vertical range.
pub fn piece_footprint(p: &Piece, v: &Venue, d: &Display, battens: &[Batten]) -> ([V3; 4], f32, f32) {
    let (base, rot) = piece_world(p, v, d, battens);
    let hw = p.size[0] * 0.5;
    let hd = p.size[2] * 0.5;
    let c = [v3(-hw, 0.0, -hd), v3(hw, 0.0, -hd), v3(hw, 0.0, hd), v3(-hw, 0.0, hd)];
    let pts = c.map(|q| base + q.rot_y(rot));
    (pts, base.y, base.y + p.size[1])
}

/// Move a point of a hinged part (part space, metres) to where it is when swung open.
pub fn swing_point(l: V3, size: V3, h: &Hinge) -> V3 {
    let a = h.open_deg.clamp(-180.0, 180.0);
    if a.abs() < 1e-4 {
        return l;
    }
    if h.side == 0 {
        // Lid / trapdoor: lifts about its bottom back edge.
        let [_, hy, hz] = h.pivot.unwrap_or([0.0, 0.0, -size.z * 0.5]);
        let (s, c) = a.to_radians().sin_cos();
        let r = l.z - hz;
        let y = l.y - hy;
        return v3(l.x, hy + y * c + r * s, hz + r * c - y * s);
    }
    let side = h.side.signum() as f32;
    let [hx, _, hz] = h.pivot.unwrap_or([side * size.x * 0.5, 0.0, 0.0]);
    let theta = if h.outwards { side * a } else { -side * a };
    let rel = v3(l.x - hx, l.y, l.z - hz).rot_y(theta);
    v3(rel.x + hx, rel.y, rel.z + hz)
}

fn piece_geo(g: &mut Geo,p: &Piece, base: V3, rot: f32, obj: u32, highlight: bool) {
    let sx = p.size[0] / p.base_size[0].max(1e-3);
    let sy = p.size[1] / p.base_size[1].max(1e-3);
    let sz = p.size[2] / p.base_size[2].max(1e-3);
    for part in &p.parts {
        let m = part.material.unwrap_or(p.material);
        let mut rgb = part.color.unwrap_or(p.color);
        if highlight {
            rgb = [rgb[0].saturating_add(30), rgb[1].saturating_add(40), rgb[2].saturating_add(60)];
        }
        let mat = g.mat(rgb, m.specular(), 0);
        if m == Material::Glass {
            g.mats[mat as usize].emissive = 0.35;
        }
        let ps = V3::from_arr(part.size);
        let po = V3::from_arr(part.offset);
        let prot = part.rot;
        let hinge = part.hinge.clone();
        let xf = move |u: V3| -> V3 {
            let mut l = v3(u.x * ps.x, u.y * ps.y, u.z * ps.z);
            if let Some(h) = &hinge {
                l = swing_point(l, ps, h);
            }
            let local = l.rot_y(prot) + po;
            let scaled = v3(local.x * sx, local.y * sy, local.z * sz);
            scaled.rot_y(rot) + base
        };
        add_shape(g, part.shape, &xf, mat, obj);
    }
}

/// The hinges where a flat is joined to another: a leaf screwed to each flat and a knuckle on
/// the hinge line, spaced up the edge (2 on short flats, up to 4 on tall ones).
#[allow(clippy::too_many_arguments)]
fn hinge_hardware(g: &mut Geo, par: &Piece, pb: V3, prot: f32, ch: &Piece, cb: V3, crot: f32, j: &HingeJoin, obj: u32) {
    let (pivot, attach, f) = crate::joins::hinge_points(par, ch, j);
    let s = if j.side >= 0 { 1.0 } else { -1.0 };
    let h = par.size[1].min(ch.size[1]);
    if h < 0.3 {
        return;
    }
    let n = if h < 1.6 { 2 } else if h < 3.2 { 3 } else { 4 };
    let (lw, lh, lt) = (0.075, 0.13, 0.005);
    let leaf_m = g.mat([184, 184, 192], 0.55, 0);
    let pin_m = g.mat([126, 126, 134], 0.55, 0);
    for k in 0..n {
        let y = 0.18 + (h - 0.36 - lh) * k as f32 / (n - 1) as f32;
        let on_par = pivot + v3(-s * lw * 0.5, y, f * lt * 0.5);
        add_shape(g, Shape::Box, &|u: V3| pb + (on_par + v3(u.x * lw, u.y * lh, u.z * lt)).rot_y(prot), leaf_m, obj);
        let on_ch = attach + v3(s * lw * 0.5, y, f * lt * 0.5);
        add_shape(g, Shape::Box, &|u: V3| cb + (on_ch + v3(u.x * lw, u.y * lh, u.z * lt)).rot_y(crot), leaf_m, obj);
        let pin = pivot + v3(0.0, y - 0.005, f * 0.01);
        add_shape(g, Shape::Cylinder, &|u: V3| pb + (pin + v3(u.x * 0.02, u.y * (lh + 0.01), u.z * 0.02)).rot_y(prot), pin_m, obj);
    }
}

pub fn light_pos(l: &Light, battens: &[Batten], d: &Display) -> V3 {
    match &l.mount {
        LightMount::Batten { index, x } => {
            let (h, z) = battens
                .get(*index)
                .map(|b| (d.batten_h.get(*index).copied().unwrap_or(b.height), -b.dist))
                .unwrap_or((5.0, -1.0));
            v3(*x, h - 0.3, z)
        }
        LightMount::Fixed { pos, .. } => V3::from_arr(*pos),
    }
}

pub fn build(pr: &Project, d: &Display, o: &BuildOpts) -> (Geo, Vec<Label>) {
    let v = &pr.venue;
    let s = &pr.scenes[pr.current.min(pr.scenes.len() - 1)].state;
    let mut g = Geo::default();
    let mut labels: Vec<Label> = Vec::new();
    let feet = o.feet;
    let bb = v.kind == VenueKind::BlackBox;

    let hw = v.stage_width() * 0.5;
    let pw = v.pros_width * 0.5;
    let depth = v.stage_depth;
    let apron = v.apron_depth;
    let sh = v.stage_height;
    let ph = v.pros_height;
    let th = v.tower_height();
    let house_hw = v.house_width * 0.5;
    let house_start = v.house_start_z();
    let house_end = house_start + v.house_depth();
    let outer = hw.max(house_hw);

    let pal = v.palette();
    let open_end = v.open_end_stage && !bb;
    let deck_col = if bb { [34, 34, 38] } else { pal.deck };
    let deck_m = g.mat(deck_col, 0.12, F_DECK_GRID);
    let wall_m = g.mat(pal.stage_wall, 0.0, 0);
    let black_m = g.mat([22, 22, 26], 0.0, 0);
    let pros_m = g.mat(pal.pros, 0.05, F_HOUSE);
    let trim_m = g.mat(pal.trim, 0.4, F_HOUSE);
    let house_wall_m = g.mat(pal.house_wall, 0.0, F_HOUSE);
    let carpet_m = g.mat(pal.carpet, 0.0, F_HOUSE);
    let seat_m = g.mat(pal.seat, 0.05, F_HOUSE);
    let riser_m = g.mat(pal.riser, 0.0, F_HOUSE);
    let tread_m = g.mat(pal.tread, 0.15, F_HOUSE);
    let steel_m = g.mat([90, 92, 98], 0.4, 0);

    // ---------------- Stage deck ----------------
    let lift_rect = if v.has_stage_lift {
        Some((v.lift_x - v.lift_width * 0.5, v.lift_x + v.lift_width * 0.5, v.lift_z - v.lift_depth * 0.5, v.lift_z + v.lift_depth * 0.5))
    } else {
        None
    };
    let deck_left = if bb { -house_hw } else { -hw };
    let deck_right = -deck_left;
    let deck_front = if bb { house_start - 0.6 } else { apron };
    let mut rects = vec![(deck_left, deck_right, -depth, deck_front)];
    if let Some((x0, x1, z0, z1)) = lift_rect {
        let (ax0, ax1, az0, az1) = rects.pop().unwrap();
        rects.push((ax0, ax1, az0, z0));
        rects.push((ax0, ax1, z1, az1));
        rects.push((ax0, x0, z0, z1));
        rects.push((x1, ax1, z0, z1));
    }
    for (x0, x1, z0, z1) in rects {
        if x1 - x0 < 1e-3 || z1 - z0 < 1e-3 {
            continue;
        }
        // Split at the setting line so the rake bends there.
        let splits: Vec<f32> = if z0 < 0.0 && z1 > 0.0 { vec![z0, 0.0, z1] } else { vec![z0, z1] };
        for w in splits.windows(2) {
            let (za, zb) = (w[0], w[1]);
            g.quad(
                v3(x0, v.deck_y(za), za),
                v3(x1, v.deck_y(za), za),
                v3(x1, v.deck_y(zb), zb),
                v3(x0, v.deck_y(zb), zb),
                deck_m,
                0,
            );
        }
    }
    if !bb {
        // Apron front face and (if pit) the pit back wall.
        let bottom = if v.has_pit { -sh - 2.8 } else { -sh };
        g.quad(v3(-hw, 0.0, apron), v3(hw, 0.0, apron), v3(hw, bottom, apron), v3(-hw, bottom, apron), black_m, 0);
        // Edge of stage: a white safety line.
        g.line(v3(-hw, 0.005, apron - 0.03), v3(hw, 0.005, apron - 0.03), [235, 235, 220, 200]);
    }

    // ---------------- Stage house walls (cut-away: only seen from inside) ----------------
    if bb {
        let top = v.pipe_height + 1.2;
        g.quad_one_sided(v3(-house_hw, 0.0, -depth), v3(house_hw, 0.0, -depth), v3(house_hw, top, -depth), v3(-house_hw, top, -depth), v3(0.0, 0.0, 1.0), black_m, 0);
        g.quad_one_sided(v3(-house_hw, 0.0, -depth), v3(-house_hw, 0.0, house_end), v3(-house_hw, top, house_end), v3(-house_hw, top, -depth), v3(1.0, 0.0, 0.0), black_m, 0);
        g.quad_one_sided(v3(house_hw, 0.0, -depth), v3(house_hw, 0.0, house_end), v3(house_hw, top, house_end), v3(house_hw, top, -depth), v3(-1.0, 0.0, 0.0), black_m, 0);
        g.quad_one_sided(v3(-house_hw, 0.0, house_end), v3(house_hw, 0.0, house_end), v3(house_hw, top, house_end), v3(-house_hw, top, house_end), v3(0.0, 0.0, -1.0), black_m, 0);
        // Pipe grid.
        if !o.plan_view {
            let mut x = -house_hw + 1.0;
            while x < house_hw {
                g.line(v3(x, v.pipe_height, -depth), v3(x, v.pipe_height, house_end), [110, 110, 120, 200]);
                x += 1.5;
            }
        }
    } else if open_end {
        // Open end stage in a low room: back wall, side walls, a lighting bulkhead across the
        // front instead of a proscenium arch, and a sloping low ceiling on the stage-left side.
        let yb = 0.0;
        let top = ph;
        g.quad_one_sided(v3(-hw, yb, -depth), v3(hw, yb, -depth), v3(hw, top, -depth), v3(-hw, top, -depth), v3(0.0, 0.0, 1.0), wall_m, 0);
        let side_top_l = v.ceiling_at(hw).unwrap_or(top);
        g.quad_one_sided(v3(-hw, yb, -depth), v3(-hw, yb, 0.45), v3(-hw, top, 0.45), v3(-hw, top, -depth), v3(1.0, 0.0, 0.0), black_m, 0);
        g.quad_one_sided(v3(hw, yb, -depth), v3(hw, yb, 0.45), v3(hw, side_top_l, 0.45), v3(hw, side_top_l, -depth), v3(-1.0, 0.0, 0.0), black_m, 0);
        // Front bulkhead (lighting bridge fascia) over the stage edge.
        let bulk_m = g.mat([26, 26, 30], 0.05, F_HOUSE);
        world_box(&mut g, v3(-hw, top - 0.05, 0.0), v3(hw, top + 0.5, 0.45), bulk_m, 0);
        // Stage-left low ceiling: a sloping soffit from the main ceiling down to the SL wall.
        if v.sl_soffit_width > 0.05 && v.sl_soffit_height > 0.5 && !o.plan_view {
            let x_in = hw - v.sl_soffit_width;
            let lo = v.sl_soffit_height;
            let soff_m = g.mat([30, 30, 34], 0.0, 0);
            let n = v3(-(top - lo), -v.sl_soffit_width, 0.0).norm();
            g.quad_one_sided(v3(x_in, top, -depth), v3(hw, lo, -depth), v3(hw, lo, 0.45), v3(x_in, top, 0.45), n, soff_m, 0);
            // Its end face seen from the house.
            g.tri(v3(x_in, top, 0.45), v3(hw, lo, 0.45), v3(hw, top, 0.45), soff_m, 0);
            if o.show_labels {
                labels.push(Label {
                    pos: v3(hw - v.sl_soffit_width * 0.5, (lo + top) * 0.5, -depth * 0.5),
                    text: format!("Low ceiling {} → {}", fmt_len(lo, feet), fmt_len(top, feet)),
                    color: Color32::from_rgb(200, 170, 140),
                });
            }
        }
        // Sound lock doors in the back wall, either side.
        let door_m = g.mat([40, 40, 44], 0.1, 0);
        for side in [-1.0f32, 1.0] {
            let cx = side * (pw - 0.2);
            world_box(&mut g, v3(cx - 0.45, 0.0, -depth), v3(cx + 0.45, 2.05, -depth + 0.03), door_m, 0);
        }
        if o.show_labels && v.has_crossover {
            labels.push(Label {
                pos: v3(0.0, 2.3, -depth),
                text: "sound lock → green room".into(),
                color: Color32::from_gray(150),
            });
        }
    } else {
        let yb = 0.0;
        g.quad_one_sided(v3(-hw, yb, -depth), v3(hw, yb, -depth), v3(hw, th, -depth), v3(-hw, th, -depth), v3(0.0, 0.0, 1.0), wall_m, 0);
        g.quad_one_sided(v3(-hw, yb, -depth), v3(-hw, yb, 0.0), v3(-hw, th, 0.0), v3(-hw, th, -depth), v3(1.0, 0.0, 0.0), wall_m, 0);
        g.quad_one_sided(v3(hw, yb, -depth), v3(hw, yb, 0.0), v3(hw, th, 0.0), v3(hw, th, -depth), v3(-1.0, 0.0, 0.0), wall_m, 0);
        // Proscenium wall and portal.
        // Cut away above the header so the stage stays visible from above.
        let pz0 = 0.0;
        let pz1 = 0.45;
        let ptop = ph + 1.3;
        world_box(&mut g, v3(-outer - 0.3, -sh, pz0), v3(-pw, ptop, pz1), pros_m, 0);
        world_box(&mut g, v3(pw, -sh, pz0), v3(outer + 0.3, ptop, pz1), pros_m, 0);
        world_box(&mut g, v3(-pw, ph, pz0), v3(pw, ptop, pz1), pros_m, 0);
        // Outline of the full fly tower behind (cut-away).
        let tc = [120, 110, 120, 140];
        g.line(v3(-hw, th, 0.0), v3(hw, th, 0.0), tc);
        g.line(v3(-hw, th, -depth), v3(hw, th, -depth), tc);
        g.line(v3(-hw, th, 0.0), v3(-hw, th, -depth), tc);
        g.line(v3(hw, th, 0.0), v3(hw, th, -depth), tc);
        g.line(v3(-hw, ptop, 0.0), v3(-hw, th, 0.0), tc);
        g.line(v3(hw, ptop, 0.0), v3(hw, th, 0.0), tc);
        let t = 0.22;
        world_box(&mut g, v3(-pw - t, 0.0, pz1), v3(-pw, ph + t, pz1 + 0.06), trim_m, 0);
        world_box(&mut g, v3(pw, 0.0, pz1), v3(pw + t, ph + t, pz1 + 0.06), trim_m, 0);
        world_box(&mut g, v3(-pw - t, ph, pz1), v3(pw + t, ph + t, pz1 + 0.06), trim_m, 0);
        if pal.wainscot.is_some() && !o.plan_view {
            // Panelled proscenium: a grid of raised panels on the house face.
            let panel_m = g.mat(pal.trim, 0.25, F_HOUSE);
            let pz = pz1 + 0.07;
            let mut x = pw + 0.45;
            while x + 0.7 < outer {
                let mut y = -sh + 0.3;
                while y + 0.9 < ptop - 0.2 {
                    for sx in [-1.0f32, 1.0] {
                        let (a, b) = if sx > 0.0 { (x, x + 0.7) } else { (-x - 0.7, -x) };
                        world_box(&mut g, v3(a, y, pz - 0.03), v3(b, y + 0.9, pz), panel_m, 0);
                    }
                    y += 1.1;
                }
                x += 0.9;
            }
        }
        if v.side_screens && !o.plan_view {
            // Projection screens on the proscenium wall either side of the opening.
            let scr_m = g.mat_emissive([150, 175, 150], 0.45, F_HOUSE);
            let sw = ((outer - pw) * 0.6).clamp(1.5, 4.0);
            for sx in [-1.0f32, 1.0] {
                let cx = sx * (pw + (outer - pw) * 0.5);
                world_box(&mut g, v3(cx - sw * 0.5, ph * 0.35, pz1 + 0.06), v3(cx + sw * 0.5, ph * 0.35 + sw * 0.56, pz1 + 0.12), scr_m, 0);
            }
        }
        // Fly tower grid and gallery.
        if v.has_fly_tower() && !o.plan_view && o.show_rigging {
            let gy = v.grid_height;
            let mut x = -hw + 0.4;
            while x < hw {
                world_box(&mut g, v3(x - 0.06, gy, -depth), v3(x + 0.06, gy + 0.25, 0.0), steel_m, 0);
                x += 1.2;
            }
            if v.fly_gallery_height > 0.5 {
                let gh = v.fly_gallery_height;
                world_box(&mut g, v3(-hw, gh - 0.15, -depth), v3(-hw + 1.2, gh, 0.0), steel_m, 0);
                g.line(v3(-hw + 1.2, gh + 1.0, -depth), v3(-hw + 1.2, gh + 1.0, 0.0), [200, 190, 80, 255]);
                g.line(v3(hw - 0.8, 0.0, -depth), v3(hw - 0.8, 0.0, 0.0), [120, 120, 130, 160]);
            }
        }
    }

    // ---------------- Revolve ----------------
    if v.has_revolve {
        let r = v.revolve_diameter * 0.5;
        let cz = v.revolve_z;
        let n = 48;
        let rm = g.mat([82, 70, 60], 0.15, 0);
        let edge_m = g.mat([40, 34, 30], 0.0, 0);
        let pt = |i: usize, rr: f32| {
            let t = i as f32 / n as f32 * std::f32::consts::TAU;
            let x = rr * t.cos();
            let z = cz + rr * t.sin();
            (x, z)
        };
        let centre = v3(0.0, v.deck_y(cz) + 0.04, cz);
        for i in 0..n {
            let (x0, z0) = pt(i, r);
            let (x1, z1) = pt(i + 1, r);
            let a = v3(x0, v.deck_y(z0) + 0.04, z0);
            let b = v3(x1, v.deck_y(z1) + 0.04, z1);
            g.tri(centre, a, b, rm, OBJ_REVOLVE);
            g.quad(a, b, v3(x1, v.deck_y(z1), z1), v3(x0, v.deck_y(z0), z0), edge_m, OBJ_REVOLVE);
        }
        for k in 0..8 {
            let ang = d.revolve + k as f32 * 45.0;
            let dir = v3(0.0, 0.0, 1.0).rot_y(ang);
            let p1 = v3(0.0, 0.0, cz) + dir * r;
            let col = if k == 0 { [255, 180, 40, 255] } else { [150, 130, 110, 200] };
            g.line(v3(0.0, v.deck_y(cz) + 0.05, cz), v3(p1.x, v.deck_y(p1.z) + 0.05, p1.z), col);
        }
        if o.show_labels {
            labels.push(Label {
                pos: v3(r * 0.7, v.deck_y(cz) + 0.1, cz + r * 0.7),
                text: format!("Revolve {:.0}°", d.revolve.rem_euclid(360.0)),
                color: Color32::from_rgb(255, 190, 70),
            });
        }
    }

    // ---------------- Stage lift ----------------
    if let Some((x0, x1, z0, z1)) = lift_rect {
        let base = v.deck_y(v.lift_z);
        let top = base + d.lift;
        let lm = g.mat([96, 86, 70], 0.1, F_DECK_GRID);
        world_box(&mut g, v3(x0 + 0.01, base - 3.2, z0 + 0.01), v3(x1 - 0.01, top, z1 - 0.01), lm, OBJ_LIFT);
        let shaft = g.mat([16, 16, 18], 0.0, 0);
        let bot = base - 3.2;
        g.quad(v3(x0, base, z0), v3(x1, base, z0), v3(x1, bot, z0), v3(x0, bot, z0), shaft, 0);
        g.quad(v3(x0, base, z1), v3(x1, base, z1), v3(x1, bot, z1), v3(x0, bot, z1), shaft, 0);
        g.quad(v3(x0, base, z0), v3(x0, base, z1), v3(x0, bot, z1), v3(x0, bot, z0), shaft, 0);
        g.quad(v3(x1, base, z0), v3(x1, base, z1), v3(x1, bot, z1), v3(x1, bot, z0), shaft, 0);
        if o.show_labels {
            labels.push(Label {
                pos: v3(x1, top + 0.1, (z0 + z1) * 0.5),
                text: format!("Stage lift {}", fmt_len(d.lift, feet)),
                color: Color32::from_rgb(255, 190, 70),
            });
        }
    }

    // ---------------- Orchestra pit ----------------
    if v.has_pit && !bb {
        let px = v.pit_width * 0.5;
        let z0 = apron;
        let z1 = apron + v.pit_depth;
        let bot = -sh - 2.8;
        let pm = g.mat([60, 54, 50], 0.1, 0);
        world_box(&mut g, v3(-px + 0.01, bot, z0 + 0.01), v3(px - 0.01, d.pit, z1 - 0.01), pm, OBJ_PIT);
        let shaft = g.mat([20, 18, 20], 0.0, F_HOUSE);
        g.quad(v3(-px, -sh, z1), v3(px, -sh, z1), v3(px, bot, z1), v3(-px, bot, z1), shaft, 0);
        g.quad(v3(-px, -sh, z0), v3(-px, -sh, z1), v3(-px, bot, z1), v3(-px, bot, z0), shaft, 0);
        g.quad(v3(px, -sh, z0), v3(px, -sh, z1), v3(px, bot, z1), v3(px, bot, z0), shaft, 0);
        // House floor beside the pit.
        g.quad(v3(-house_hw, -sh, z0), v3(-px, -sh, z0), v3(-px, -sh, z1), v3(-house_hw, -sh, z1), carpet_m, 0);
        g.quad(v3(px, -sh, z0), v3(house_hw, -sh, z0), v3(house_hw, -sh, z1), v3(px, -sh, z1), carpet_m, 0);
        if d.pit < -sh - 0.2 {
            g.line(v3(-px, -sh + 0.9, z1 + 0.05), v3(px, -sh + 0.9, z1 + 0.05), [180, 160, 120, 255]);
        }
        if o.show_labels {
            let state = if d.pit > -0.05 {
                "stage level (forestage)"
            } else if d.pit > -sh - 0.1 {
                "auditorium level"
            } else {
                "pit level"
            };
            labels.push(Label {
                pos: v3(px * 0.6, d.pit + 0.1, (z0 + z1) * 0.5),
                text: format!("Orchestra pit {} – {}", fmt_len(d.pit, feet), state),
                color: Color32::from_rgb(255, 190, 70),
            });
        }
    }

    // ---------------- Auditorium ----------------
    {
        let floor_y = if bb { 0.0 } else { -sh };
        let front = if bb { house_start - 0.6 } else if v.has_pit { apron + v.pit_depth } else { apron };
        if !bb {
            g.quad(v3(-house_hw, floor_y, front), v3(house_hw, floor_y, front), v3(house_hw, floor_y, house_start), v3(-house_hw, floor_y, house_start), carpet_m, 0);
            let ceil = if open_end { ph + 0.9 } else { ph + 3.5 };
            g.quad_one_sided(v3(-house_hw, floor_y, 0.45), v3(-house_hw, floor_y, house_end), v3(-house_hw, ceil, house_end), v3(-house_hw, ceil, 0.45), v3(1.0, 0.0, 0.0), house_wall_m, 0);
            g.quad_one_sided(v3(house_hw, floor_y, 0.45), v3(house_hw, floor_y, house_end), v3(house_hw, ceil, house_end), v3(house_hw, ceil, 0.45), v3(-1.0, 0.0, 0.0), house_wall_m, 0);
            g.quad_one_sided(v3(-house_hw, floor_y, house_end), v3(house_hw, floor_y, house_end), v3(house_hw, ceil, house_end), v3(-house_hw, ceil, house_end), v3(0.0, 0.0, -1.0), house_wall_m, 0);
            let back_y = floor_y + v.row_rise(v.rows().saturating_sub(1));
            let row_end = house_start + v.row_offset(v.rows());
            if !o.plan_view {
                // Panelling / dado rail following the rake.
                let band = |g: &mut Geo, lo: f32, hi: f32, m: u32| {
                    for sx in [-1.0f32, 1.0] {
                        let x = sx * (house_hw - 0.02);
                        let n = v3(-sx, 0.0, 0.0);
                        g.quad_one_sided(v3(x, floor_y + lo, 0.45), v3(x, floor_y + lo, house_start), v3(x, floor_y + hi, house_start), v3(x, floor_y + hi, 0.45), n, m, 0);
                        g.quad_one_sided(v3(x, floor_y + lo, house_start), v3(x, back_y + lo, row_end), v3(x, back_y + hi, row_end), v3(x, floor_y + hi, house_start), n, m, 0);
                        g.quad_one_sided(v3(x, back_y + lo, row_end), v3(x, back_y + lo, house_end), v3(x, back_y + hi, house_end), v3(x, back_y + hi, row_end), n, m, 0);
                    }
                };
                if let Some(c) = pal.wainscot {
                    let m = g.mat(c, 0.2, F_HOUSE);
                    band(&mut g, 0.0, 1.4, m);
                }
                if let Some(c) = pal.rail {
                    let m = g.mat(c, 0.45, F_HOUSE);
                    band(&mut g, 1.05, 1.17, m);
                    // Gold edge round the proscenium wall too.
                    if !open_end {
                        world_box(&mut g, v3(-outer - 0.3, ph + 1.15, 0.45), v3(outer + 0.3, ph + 1.25, 0.5), m, 0);
                    }
                }
                if let Some(c) = pal.ceiling {
                    // Ceiling, seen only from below (house views); open from above.
                    let m = g.mat(c, 0.05, F_HOUSE);
                    g.quad_one_sided(v3(-house_hw, ceil, 0.45), v3(house_hw, ceil, 0.45), v3(house_hw, ceil, house_end), v3(-house_hw, ceil, house_end), v3(0.0, -1.0, 0.0), m, 0);
                    // Roof trusses / slats as beams across the room.
                    let beam_m = g.mat([c[0].saturating_sub(30), c[1].saturating_sub(24), c[2].saturating_sub(18)], 0.1, F_HOUSE);
                    let step = if v.look == RoomLook::TimberHall { 1.0 } else { 2.6 };
                    let mut z = 1.2;
                    while z < house_end - 0.3 {
                        world_box(&mut g, v3(-house_hw, ceil - 0.35, z - 0.12), v3(house_hw, ceil - 0.02, z + 0.12), beam_m, 0);
                        z += step;
                    }
                }
            }
        }
        let rows = v.rows();
        let rise = v.house_rise_per_row;
        let inner = house_hw - 1.1;
        let separate = (v.side_aisle || v.aisles_both_sides || pal.separate_seats) && !bb;
        let blocks: Vec<(f32, f32)> = if v.side_aisle && !bb {
            // One seating block against house left, aisle stairs on house right.
            let w = (v.seats_per_row() as f32 * 0.52).min(house_hw * 2.0 - 1.0);
            let x0 = -house_hw + 0.3;
            vec![(x0, x0 + w)]
        } else if v.aisles_both_sides && !bb {
            // One central block with aisle stairs down both sides.
            let w = (v.seats_per_row() as f32 * 0.52).min(house_hw * 2.0 - 2.2);
            vec![(-w * 0.5, w * 0.5)]
        } else if inner * 2.0 > 12.0 {
            let a = inner / 3.0;
            vec![(-inner, -a - 0.55), (-a + 0.55, a - 0.55), (a + 0.55, inner)]
        } else {
            vec![(-inner, -0.55), (0.55, inner)]
        };
        for r in 0..rows {
            let z = house_start + v.row_offset(r);
            let y = floor_y + v.row_rise(r);
            // A cross aisle in front of this row: extend its platform forward.
            let zf = if v.cross_aisle_after > 0 && r == v.cross_aisle_after { z - 1.2 } else { z };
            if rise > 0.0 {
                world_box(&mut g, v3(-house_hw, floor_y, zf), v3(house_hw, y, z + 0.92), riser_m, 0);
                if zf < z {
                    g.quad(v3(-house_hw, y + 0.005, zf), v3(house_hw, y + 0.005, zf), v3(house_hw, y + 0.005, z), v3(-house_hw, y + 0.005, z), carpet_m, 0);
                }
            } else {
                g.quad(v3(-house_hw, y, zf), v3(house_hw, y, zf), v3(house_hw, y, z + 0.92), v3(-house_hw, y, z + 0.92), carpet_m, 0);
            }
            if !o.plan_view || true {
                for &(x0, x1) in &blocks {
                    if separate {
                        // Individual seats with a dark frame, like the venue's red tip-up seats.
                        let n = ((x1 - x0) / 0.52).round().max(1.0) as usize;
                        let sw = (x1 - x0) / n as f32;
                        for k in 0..n {
                            let a = x0 + k as f32 * sw + 0.03;
                            let b = a + sw - 0.06;
                            world_box(&mut g, v3(a, y + 0.2, z + 0.3), v3(b, y + 0.45, z + 0.75), seat_m, 0);
                            world_box(&mut g, v3(a, y + 0.3, z + 0.72), v3(b, y + 0.95, z + 0.86), seat_m, 0);
                            world_box(&mut g, v3(a - 0.03, y, z + 0.45), v3(a, y + 0.6, z + 0.8), black_m, 0);
                        }
                    } else {
                        world_box(&mut g, v3(x0, y, z + 0.3), v3(x1, y + 0.45, z + 0.75), seat_m, 0);
                        world_box(&mut g, v3(x0, y, z + 0.75), v3(x1, y + 0.95, z + 0.86), seat_m, 0);
                    }
                }
            }
        }
        let rows_end = house_start + v.row_offset(rows);
        let back_y = floor_y + v.row_rise(rows.saturating_sub(1));
        if rise > 0.0 {
            g.quad(v3(-house_hw, back_y, rows_end), v3(house_hw, back_y, rows_end), v3(house_hw, back_y, house_end), v3(-house_hw, back_y, house_end), carpet_m, 0);
        }
        if (v.side_aisle || v.aisles_both_sides) && !bb && rise > 0.0 && !o.plan_view {
            // Aisle stairs (two treads per row) with a handrail along the wall.
            let mut aisles: Vec<(f32, f32, f32)> = Vec::new();
            if let Some(b) = blocks.last() {
                aisles.push((b.1 + 0.1, house_hw, house_hw - 0.12));
            }
            if v.aisles_both_sides {
                if let Some(b) = blocks.first() {
                    aisles.push((-house_hw, b.0 - 0.1, -house_hw + 0.12));
                }
            }
            let nose_m = g.mat(pal.trim, 0.3, F_HOUSE);
            for &(ax0, ax1, rail_x) in &aisles {
                for r in 0..rows {
                    let z = house_start + v.row_offset(r);
                    let y = floor_y + v.row_rise(r);
                    world_box(&mut g, v3(ax0, y - rise * 0.5, z - 0.46), v3(ax1, y - rise * 0.5 + 0.01, z), tread_m, 0);
                    world_box(&mut g, v3(ax0, y, z), v3(ax1, y + 0.01, z + 0.92), tread_m, 0);
                    if v.aisles_both_sides {
                        // Carpeted risers with a gold nosing on each step.
                        world_box(&mut g, v3(ax0, y - rise * 0.5, z - 0.47), v3(ax1, y - rise * 0.5 + 0.03, z - 0.44), nose_m, 0);
                        world_box(&mut g, v3(ax0, y, z - 0.01), v3(ax1, y + 0.03, z + 0.02), nose_m, 0);
                    }
                }
                g.line(v3(rail_x, floor_y + 0.9, house_start), v3(rail_x, back_y + 0.9, rows_end), [190, 190, 196, 255]);
                if v.aisles_both_sides {
                    // Inner handrail beside the seats.
                    let ix = if rail_x > 0.0 { ax0 + 0.05 } else { ax1 - 0.05 };
                    g.line(v3(ix, floor_y + 0.9, house_start), v3(ix, back_y + 0.9, rows_end), [210, 180, 110, 255]);
                }
            }
        }
        if v.side_galleries && !bb && !o.plan_view {
            // Raised side galleries along both house walls, two seats deep, with a balustrade.
            let gw = 1.3;
            let z0 = house_start + 0.4;
            let z1 = (rows_end - 0.4).max(z0 + 2.0);
            let gy = (floor_y + v.row_rise(rows / 3) + 2.6).max(back_y + 1.3);
            let deck_m2 = g.mat(pal.riser, 0.05, F_HOUSE);
            let bal_m = g.mat(pal.house_wall, 0.05, F_HOUSE);
            for sx in [-1.0f32, 1.0] {
                let (xa, xb) = if sx > 0.0 { (house_hw - gw, house_hw) } else { (-house_hw, -house_hw + gw) };
                world_box(&mut g, v3(xa, gy - 0.25, z0), v3(xb, gy, z1), deck_m2, 0);
                let inner_x = if sx > 0.0 { xa } else { xb };
                world_box(&mut g, v3(inner_x - 0.04, gy, z0), v3(inner_x + 0.04, gy + 0.95, z1), bal_m, 0);
                let mut z = z0 + 0.2;
                while z + 0.6 < z1 {
                    for k in 0..2 {
                        let a = xa + 0.1 + k as f32 * 0.58;
                        let b = a + 0.5;
                        world_box(&mut g, v3(a, gy + 0.2, z), v3(b, gy + 0.45, z + 0.45), seat_m, 0);
                        world_box(&mut g, v3(a, gy + 0.3, z + 0.42), v3(b, gy + 0.9, z + 0.55), seat_m, 0);
                    }
                    z += 0.9;
                }
            }
            if o.show_labels {
                labels.push(Label { pos: v3(house_hw - 0.6, gy + 1.1, (z0 + z1) * 0.5), text: "Side gallery".into(), color: Color32::from_gray(170) });
            }
        }
        if v.rear_gallery && !bb && !o.plan_view {
            // A gallery / balcony over the back rows.
            let gd = 3.4;
            let z0 = house_end - gd;
            let gy = back_y + 2.7;
            let deck_m2 = g.mat(pal.riser, 0.05, F_HOUSE);
            let front_m = g.mat(if pal.wainscot.is_some() { pal.trim } else { pal.house_wall }, 0.1, F_HOUSE);
            world_box(&mut g, v3(-house_hw, gy - 0.3, z0), v3(house_hw, gy, house_end), deck_m2, 0);
            world_box(&mut g, v3(-house_hw, gy - 0.3, z0 - 0.06), v3(house_hw, gy + 0.95, z0), front_m, 0);
            if let Some(c) = pal.rail {
                let m = g.mat(c, 0.45, F_HOUSE);
                world_box(&mut g, v3(-house_hw, gy + 0.95, z0 - 0.08), v3(house_hw, gy + 1.02, z0 + 0.02), m, 0);
            }
            for row in 0..3 {
                let z = z0 + 0.3 + row as f32 * 0.95;
                let y = gy + row as f32 * 0.3;
                if row > 0 {
                    world_box(&mut g, v3(-house_hw, gy, z - 0.1), v3(house_hw, y, z + 0.85), deck_m2, 0);
                }
                world_box(&mut g, v3(-house_hw + 0.6, y + 0.2, z + 0.1), v3(house_hw - 0.6, y + 0.45, z + 0.55), seat_m, 0);
                world_box(&mut g, v3(-house_hw + 0.6, y + 0.3, z + 0.52), v3(house_hw - 0.6, y + 0.9, z + 0.64), seat_m, 0);
            }
            if o.show_labels {
                labels.push(Label { pos: v3(0.0, gy + 1.3, z0), text: "Rear gallery".into(), color: Color32::from_gray(170) });
            }
        }
        if v.cross_aisle_after > 0 && v.cross_aisle_after < rows && o.show_labels {
            let r = v.cross_aisle_after;
            labels.push(Label {
                pos: v3(house_hw * 0.55, floor_y + v.row_rise(r) + 0.1, house_start + v.row_offset(r) - 0.6),
                text: "cross aisle".into(),
                color: Color32::from_gray(160),
            });
        }
        if v.control_box && !bb {
            // Control box (bio box) across the back of the house, behind a window.
            let y = back_y;
            let cb_m = g.mat([60, 60, 64], 0.0, F_HOUSE);
            let win_m = g.mat_emissive([90, 120, 140], 0.25, F_HOUSE);
            let z0 = house_end - 0.6;
            world_box(&mut g, v3(-house_hw, y, z0), v3(house_hw * 0.1, y + 1.05, z0 + 0.1), cb_m, 0);
            world_box(&mut g, v3(-house_hw + 0.8, y + 1.05, z0 + 0.02), v3(house_hw * 0.1 - 0.4, y + 1.9, z0 + 0.06), win_m, 0);
            if o.show_labels {
                labels.push(Label {
                    pos: v3(-house_hw * 0.45, y + 2.1, z0),
                    text: "Control box (lighting & sound desks)".into(),
                    color: Color32::from_gray(170),
                });
            }
        }
        if o.show_labels {
            labels.push(Label {
                pos: v3(0.0, floor_y + 0.2, house_start + v.row_offset(rows) * 0.5),
                text: format!("Auditorium · {} seats", v.capacity),
                color: Color32::from_rgb(230, 200, 200),
            });
        }
    }

    // ---------------- House steps ----------------
    if v.has_house_steps && !bb && sh > 0.2 {
        let n = ((sh / 0.18).ceil() as u8).max(2);
        let run = n as f32 * 0.28;
        let mut sx = pw * 0.8;
        if v.has_pit {
            sx = sx.max(v.pit_width * 0.5 + 0.7);
        }
        let step_m = g.mat(pal.steps, 0.05, 0);
        let sides: &[f32] = if v.side_aisle { &[1.0] } else { &[-1.0, 1.0] };
        if v.side_aisle {
            sx = hw - 0.7;
        }
        for &side in sides {
            let cx = side * sx.min(hw - 0.6);
            if v.side_aisle && !o.plan_view {
                // Handrails either side of the steps.
                for dx in [-0.55f32, 0.55] {
                    g.line(v3(cx + dx, 0.9, apron), v3(cx + dx, -sh + 0.9, apron + run), [190, 190, 196, 255]);
                }
            }
            let xf = move |u: V3| v3(cx + u.x * 1.1, -sh + u.y * sh, apron + run * 0.5 + u.z * run);
            add_shape(&mut g, Shape::Stairs(n), &xf, step_m, 0);
        }
    }

    // ---------------- Battens & soft goods ----------------
    for (bi, b) in s.battens.iter().enumerate() {
        if !b.visible || !o.show_rigging && b.kind != BattenKind::HouseCurtain && b.kind != BattenKind::Cyclorama && b.kind != BattenKind::Drop {
            continue;
        }
        let h = d.batten_h.get(bi).copied().unwrap_or(b.height);
        let open = d.batten_open.get(bi).copied().unwrap_or(b.open);
        let z = -b.dist;
        let obj = OBJ_BATTEN + bi as u32;
        let span = pw + 1.0;
        let sel = o.selected == obj;
        let mut goods_m = g.mat(b.color, 0.02, 0);
        if b.kind == BattenKind::Cyclorama {
            goods_m = g.mat(b.color, 0.0, F_NO_EDGE);
        }
        let pipe_m = g.mat([70, 70, 76], 0.5, 0);
        let pipe_col = if sel { [255, 150, 30, 255] } else { [150, 150, 160, 200] };
        let span_pipe = if bb { house_hw - 0.5 } else { span + 0.6 };
        g.line(v3(-span_pipe, h, z), v3(span_pipe, h, z), pipe_col);
        if !o.plan_view {
            world_box(&mut g, v3(-span_pipe, h - 0.025, z - 0.025), v3(span_pipe, h + 0.025, z + 0.025), pipe_m, obj);
            // Lift lines up to the grid.
            let top = if v.has_fly_tower() { v.grid_height } else if open_end { ph } else { th - 1.0 };
            if o.show_rigging {
                for k in 0..5 {
                    let x = -span_pipe + 2.0 * span_pipe * k as f32 / 4.0;
                    g.line(v3(x, h, z), v3(x, top, z), [120, 120, 130, 110]);
                }
            }
        }
        let ybot = (h - b.goods_h).max(v.deck_y(z));
        if o.plan_view {
            let col = match b.kind {
                BattenKind::Lighting => [120, 200, 255, 230],
                BattenKind::Scenery => [160, 160, 160, 200],
                _ => [230, 120, 120, 220],
            };
            g.line(v3(-span, 0.02, z), v3(span, 0.02, z), col);
        } else {
            match b.kind {
                BattenKind::HouseCurtain | BattenKind::Tabs => {
                    let hwc = span;
                    let pleat_d = 0.06 + 0.12 * open;
                    match (b.kind, v.curtain_style) {
                        (BattenKind::HouseCurtain, CurtainStyle::Guillotine) => {
                            let lift = open * (ph + 0.2);
                            drape(&mut g, -hwc, hwc, (ybot + lift).min(h), h + lift, z, 0.3, 0.06, goods_m, obj);
                        }
                        _ => {
                            let half = hwc * (1.0 - 0.82 * open);
                            drape(&mut g, -hwc, -hwc + half, ybot, h, z, 0.25, pleat_d, goods_m, obj);
                            drape(&mut g, hwc - half, hwc, ybot, h, z, 0.25, pleat_d, goods_m, obj);
                        }
                    }
                    if b.kind == BattenKind::HouseCurtain {
                        // Valance across the top.
                        drape(&mut g, -hwc, hwc, (ph - 0.6).max(h - 1.0).min(h), h + 0.3, z + 0.12, 0.3, 0.05, goods_m, obj);
                    }
                }
                BattenKind::Border => {
                    g.quad(v3(-span, ybot, z), v3(span, ybot, z), v3(span, h, z), v3(-span, h, z), goods_m, obj);
                }
                BattenKind::Legs | BattenKind::LegsAndBorder => {
                    let inner = if b.opening > 0.1 { b.opening * 0.5 } else { pw - 0.25 };
                    let lw = if b.opening > 0.1 { (hw - inner).clamp(0.6, 3.0) } else { 1.4 };
                    for side in [-1.0f32, 1.0] {
                        let x0 = side * inner;
                        let x1 = side * (inner + lw);
                        g.quad(v3(x0, ybot, z), v3(x1, ybot, z), v3(x1, h, z), v3(x0, h, z), goods_m, obj);
                    }
                    if b.kind == BattenKind::LegsAndBorder {
                        let bh = if b.border_h > 0.05 { b.border_h } else if v.has_fly_tower() { 1.6 } else { 0.8 };
                        g.quad(v3(-span, h - bh, z + 0.08), v3(span, h - bh, z + 0.08), v3(span, h, z + 0.08), v3(-span, h, z + 0.08), goods_m, obj);
                    }
                }
                BattenKind::Drop | BattenKind::Cyclorama => {
                    let sp = if b.opening > 0.1 {
                        b.opening * 0.5
                    } else if b.kind == BattenKind::Cyclorama {
                        span + 0.5
                    } else {
                        span
                    };
                    // A few vertical strips so lighting can vary across it.
                    let strips = 6;
                    for k in 0..strips {
                        let xa = -sp + 2.0 * sp * k as f32 / strips as f32;
                        let xb = -sp + 2.0 * sp * (k + 1) as f32 / strips as f32;
                        g.quad(v3(xa, ybot, z), v3(xb, ybot, z), v3(xb, h, z), v3(xa, h, z), goods_m, obj);
                    }
                }
                BattenKind::Lighting | BattenKind::Scenery => {}
            }
        }
        // Height readouts: the selected line, plus goods that are flown in (in view).
        let flown_in = h < v.max_trim() - 0.4 && !matches!(b.kind, BattenKind::Lighting | BattenKind::Scenery | BattenKind::HouseCurtain | BattenKind::LegsAndBorder | BattenKind::Legs | BattenKind::Border);
        if o.show_labels && (sel || (o.show_rigging && flown_in && !o.plan_view)) {
            let fly_note = if v.has_fly_tower() { "" } else { " (dead-hung)" };
            labels.push(Label {
                pos: v3(span_pipe + 0.1, h, z),
                text: format!("{} {}{}", b.name, fmt_len(h, feet), fly_note),
                color: if sel { Color32::from_rgb(255, 170, 60) } else { Color32::from_rgb(170, 190, 215) },
            });
        }
    }

    // ---------------- Lights ----------------
    let body_m = g.mat([30, 30, 34], 0.4, 0);
    let mut foh_drawn: Vec<f32> = Vec::new();
    for (li, l) in s.lights.iter().enumerate() {
        let pos = light_pos(l, &s.battens, d);
        if let LightMount::Batten { index, .. } = l.mount {
            if let Some(b) = s.battens.get(index) {
                if !b.visible {
                    continue;
                }
            }
        }
        let target = V3::from_arr(l.target);
        let dir = (target - pos).norm();
        let obj = OBJ_LIGHT + li as u32;
        let lit = l.intensity > 0.01;
        let lc = [
            (l.color[0] * 255.0) as u8,
            (l.color[1] * 255.0) as u8,
            (l.color[2] * 255.0) as u8,
        ];
        let lens_m = if lit && !o.work_mode {
            g.mat_emissive(lc, 1.5 * l.intensity, F_LENS | F_NO_EDGE)
        } else {
            g.mat([70, 70, 74], 0.6, 0)
        };
        if !o.plan_view || true {
            let len = match l.kind {
                LightKind::Profile => 0.55,
                LightKind::Flood => 0.3,
                _ => 0.35,
            };
            let rad = match l.kind {
                LightKind::Flood => 0.2,
                _ => 0.13,
            };
            tube(&mut g, pos - dir * (len * 0.3), pos + dir * (len * 0.7), rad, 8, body_m, obj, Some(lens_m));
        }
        if let LightMount::Fixed { pos: fp, position_name } = &l.mount {
            if position_name.contains("house wall") && !o.plan_view {
                // Perch / box boom: a vertical pipe on the auditorium side wall.
                let pm = g.mat([50, 50, 56], 0.3, 0);
                let y0 = crate::venues::house_floor_at(v, fp[2]) + 1.5;
                tube(&mut g, v3(fp[0], y0.min(fp[1] - 0.5), fp[2]), v3(fp[0], fp[1] + 0.6, fp[2]), 0.03, 6, pm, 0, None);
            } else if position_name.contains("boom") {
                g.line(v3(fp[0], v.deck_y(fp[2]), fp[2]), v3(fp[0], fp[1] + 0.6, fp[2]), [150, 150, 160, 255]);
                let pm = g.mat([50, 50, 56], 0.3, 0);
                world_box(&mut g, v3(fp[0] - 0.25, v.deck_y(fp[2]), fp[2] - 0.25), v3(fp[0] + 0.25, v.deck_y(fp[2]) + 0.04, fp[2] + 0.25), pm, 0);
                tube(&mut g, v3(fp[0], v.deck_y(fp[2]), fp[2]), v3(fp[0], fp[1] + 0.6, fp[2]), 0.025, 6, pm, 0, None);
            }
            if position_name.contains("FOH") && !foh_drawn.iter().any(|z| (z - fp[2]).abs() < 0.05) && !o.plan_view {
                foh_drawn.push(fp[2]);
                let y = fp[1] + 0.35;
                if position_name.contains("gallery") {
                    // Two parallel bars along a lighting gallery.
                    for dz in [-0.3f32, 0.3] {
                        world_box(&mut g, v3(-house_hw + 0.15, y - 0.05, fp[2] + dz - 0.04), v3(house_hw - 0.15, y + 0.03, fp[2] + dz + 0.04), steel_m, 0);
                    }
                } else if position_name.contains("bar") {
                    // A plain lighting bar hung under the ceiling.
                    world_box(&mut g, v3(-house_hw + 0.15, y - 0.05, fp[2] - 0.04), v3(house_hw - 0.15, y + 0.03, fp[2] + 0.04), steel_m, 0);
                } else {
                    world_box(&mut g, v3(-house_hw + 0.3, y, fp[2] - 0.4), v3(house_hw - 0.3, y + 0.25, fp[2] + 0.4), steel_m, 0);
                }
            }
        }
        if o.selected == obj {
            g.lines.push(Line3 { a: pos, b: target, color: [255, 200, 60, 255], xray: true });
            let r = (target - pos).len() * (l.beam_deg * 0.5).to_radians().tan();
            let u = dir.cross(v3(0.0, 1.0, 0.0)).norm();
            let w = dir.cross(u).norm();
            for k in 0..24 {
                let t0 = k as f32 / 24.0 * std::f32::consts::TAU;
                let t1 = (k + 1) as f32 / 24.0 * std::f32::consts::TAU;
                g.lines.push(Line3 {
                    a: target + u * (r * t0.cos()) + w * (r * t0.sin()),
                    b: target + u * (r * t1.cos()) + w * (r * t1.sin()),
                    color: [255, 200, 60, 220],
                    xray: true,
                });
            }
            if o.show_labels {
                labels.push(Label { pos, text: l.name.clone(), color: Color32::from_rgb(255, 210, 120) });
            }
        }
        if lit {
            let half = (l.beam_deg * 0.5).to_radians();
            let softness = match l.kind {
                LightKind::Profile => 0.85,
                LightKind::Par => 0.6,
                LightKind::Fresnel => 0.35,
                LightKind::Flood => 0.3,
            };
            let k = l.intensity * 2.4;
            g.lights.push(ShadeLight {
                pos,
                dir,
                cos_outer: half.cos(),
                cos_inner: (half * softness).cos(),
                color: [l.color[0] * k, l.color[1] * k, l.color[2] * k],
            });
            if o.show_beams {
                // Beam length: to the target, then a bit further.
                let len = ((target - pos).len() * 1.15).min(30.0);
                let r = len * half.tan();
                let u = dir.cross(if dir.y.abs() < 0.9 { v3(0.0, 1.0, 0.0) } else { v3(1.0, 0.0, 0.0) }).norm();
                let w = dir.cross(u).norm();
                let end = pos + dir * len;
                let n = 20;
                let col = [l.color[0] * l.intensity, l.color[1] * l.intensity, l.color[2] * l.intensity];
                for kk in 0..n {
                    let t0 = kk as f32 / n as f32 * std::f32::consts::TAU;
                    let t1 = (kk + 1) as f32 / n as f32 * std::f32::consts::TAU;
                    let a = end + u * (r * t0.cos()) + w * (r * t0.sin());
                    let bpt = end + u * (r * t1.cos()) + w * (r * t1.sin());
                    let nrm = (a - pos).cross(bpt - pos).norm();
                    g.beams.push(BeamTri { p: [pos, a, bpt], w: [1.0, 0.18, 0.18], n: nrm, color: col });
                }
            }
        }
    }

    // ---------------- Set pieces ----------------
    for (pi, p) in s.pieces.iter().enumerate() {
        if !p.visible {
            continue;
        }
        let (base, rot) = piece_world(p, v, d, &s.battens);
        piece_geo(&mut g, p, base, rot, OBJ_PIECE + pi as u32, false);
        if p.flown_on.is_some() && !o.plan_view {
            let top = base.y + p.size[1];
            g.line(base + v3(-p.size[0] * 0.4, p.size[1], 0.0), v3(base.x - p.size[0] * 0.4, top, base.z), [150, 150, 160, 200]);
        }
    }
    // Hinges between flats joined together.
    for (ci, ch) in s.pieces.iter().enumerate() {
        let Some(j) = &ch.hinged_to else { continue };
        let Some(par) = s.pieces.iter().find(|q| q.id == j.to) else { continue };
        if !ch.visible || !par.visible {
            continue;
        }
        let (pb, prot) = piece_world(par, v, d, &s.battens);
        let (cb, crot) = piece_world(ch, v, d, &s.battens);
        hinge_hardware(&mut g, par, pb, prot, ch, cb, crot, j, OBJ_PIECE + ci as u32);
    }
    if let Some(dp) = &o.draft {
        let (base, rot) = piece_world(dp, v, d, &s.battens);
        piece_geo(&mut g, dp, base, rot, OBJ_DRAFT, true);
        if o.show_labels {
            labels.push(Label {
                pos: base + v3(0.0, dp.size[1] + 0.2, 0.0),
                text: format!("Preview: {}", dp.name),
                color: Color32::from_rgb(140, 220, 255),
            });
        }
    }

    // ---------------- Sightlines ----------------
    if o.show_sightlines && !bb {
        let seat_z = house_start + 0.5;
        for side in [-1.0f32, 1.0] {
            let seat_x = side * (house_hw - 1.4).min(pw + 2.5);
            let edge_x = -side * pw;
            // Line from seat through the opposite proscenium edge, extended upstage to the back wall.
            let dx = edge_x - seat_x;
            let dz = 0.0 - seat_z;
            let t = (-depth - seat_z) / dz;
            let far_x = seat_x + dx * t;
            let far_x = far_x.clamp(-hw, hw);
            let far_z = if (seat_x + dx * t).abs() > hw { seat_z + dz * ((side * -hw - seat_x) / dx) } else { -depth };
            g.lines.push(Line3 { a: v3(edge_x, 0.03, 0.0), b: v3(far_x, 0.03, far_z), color: [255, 200, 40, 255], xray: true });
            g.lines.push(Line3 { a: v3(seat_x, -sh + 1.15, seat_z), b: v3(edge_x, 0.03, 0.0), color: [255, 200, 40, 120], xray: true });
        }
        if o.show_labels {
            labels.push(Label {
                pos: v3(0.0, 0.05, -depth + 0.3),
                text: "Sightlines from extreme front-row seats".into(),
                color: Color32::from_rgb(255, 210, 80),
            });
        }
    }

    if o.show_labels {
        let col = Color32::from_rgb(200, 200, 210);
        let lbl_y = 0.05;
        if !bb {
            labels.push(Label { pos: v3(hw - 0.4, lbl_y, -depth * 0.5), text: "STAGE LEFT".into(), color: col });
            labels.push(Label { pos: v3(-hw + 0.4, lbl_y, -depth * 0.5), text: "STAGE RIGHT".into(), color: col });
            if v.has_crossover {
                labels.push(Label { pos: v3(pw * 0.75, v.deck_y(-depth) + 0.05, -depth + 0.3), text: "crossover".into(), color: Color32::from_gray(150) });
            }
        }
        labels.push(Label { pos: v3(0.0, lbl_y, apron.max(0.2) - 0.15), text: "DOWNSTAGE".into(), color: col });
        labels.push(Label { pos: v3(0.0, v.deck_y(-depth) + lbl_y, -depth + 0.9), text: "UPSTAGE".into(), color: col });
    }

    (g, labels)
}
