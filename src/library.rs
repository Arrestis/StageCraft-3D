// Copyright (C) 2026 Doug Rumble
// SPDX-License-Identifier: GPL-3.0-only

//! Preset library of furniture and set pieces. Every preset is a template: a list of
//! settings (sizes, counts, options, how far a door is open) and a generator that rebuilds
//! the piece's parts from them, so any piece can be resized and its doors opened or shut.

use crate::math::v3;
use crate::model::*;

// ------------------------------------------------------------------ part helpers

fn part(shape: Shape, x: f32, y: f32, z: f32, w: f32, h: f32, d: f32) -> Part {
    Part { shape, offset: [x, y, z], size: [w.max(0.005), h.max(0.002), d.max(0.002)], rot: 0.0, color: None, material: None, hinge: None }
}
fn bx(x: f32, y: f32, z: f32, w: f32, h: f32, d: f32) -> Part {
    part(Shape::Box, x, y, z, w, h, d)
}
fn cyl(x: f32, y: f32, z: f32, w: f32, h: f32, d: f32) -> Part {
    part(Shape::Cylinder, x, y, z, w, h, d)
}
fn sph(x: f32, y: f32, z: f32, w: f32, h: f32, d: f32) -> Part {
    part(Shape::Sphere, x, y, z, w, h, d)
}
trait Tint {
    fn c(self, rgb: [u8; 3]) -> Part;
    fn m(self, m: Material) -> Part;
    fn r(self, deg: f32) -> Part;
}
impl Tint for Part {
    fn c(mut self, rgb: [u8; 3]) -> Part {
        self.color = Some(rgb);
        self
    }
    fn m(mut self, m: Material) -> Part {
        self.material = Some(m);
        if self.color.is_none() {
            self.color = Some(m.default_color());
        }
        self
    }
    fn r(mut self, deg: f32) -> Part {
        self.rot = deg;
        self
    }
}

/// Make a group of parts swing together about one vertical hinge line at (hx, hz)
/// (side -1 = hinge on the leaf's left, +1 = right).
fn hinged(parts: Vec<Part>, hx: f32, hz: f32, side: i8, open: f32, outwards: bool, max: f32, label: &str) -> Vec<Part> {
    parts
        .into_iter()
        .map(|mut p| {
            p.hinge = Some(Hinge {
                side,
                open_deg: open,
                outwards,
                max_deg: max,
                label: label.into(),
                pivot: Some([hx - p.offset[0], 0.0, hz - p.offset[2]]),
            });
            p
        })
        .collect()
}

/// A lid / trapdoor lifting about its own back bottom edge.
fn lid(mut p: Part, open: f32, max: f32, label: &str) -> Part {
    p.hinge = Some(Hinge { side: 0, open_deg: open, outwards: true, max_deg: max, label: label.into(), pivot: None });
    p
}

fn legs4(w: f32, d: f32, h: f32, t: f32, color: [u8; 3]) -> Vec<Part> {
    let x = w * 0.5 - t * 0.5;
    let z = d * 0.5 - t * 0.5;
    vec![
        bx(-x, 0.0, -z, t, h, t).c(color),
        bx(x, 0.0, -z, t, h, t).c(color),
        bx(-x, 0.0, z, t, h, t).c(color),
        bx(x, 0.0, z, t, h, t).c(color),
    ]
}

// ------------------------------------------------------------------ setting helpers

fn pm(key: &str, label: &str, group: &str, value: f32, min: f32, max: f32, kind: ParamKind) -> Param {
    Param { key: key.into(), label: label.into(), value, min, max, kind, group: group.into() }
}
fn len(key: &str, label: &str, group: &str, value: f32, min: f32, max: f32) -> Param {
    pm(key, label, group, value, min, max, ParamKind::Length)
}
fn cnt(key: &str, label: &str, group: &str, value: f32, min: f32, max: f32) -> Param {
    pm(key, label, group, value, min, max, ParamKind::Count)
}
fn ang(key: &str, label: &str, group: &str, value: f32, min: f32, max: f32) -> Param {
    pm(key, label, group, value, min, max, ParamKind::Angle)
}
fn tog(key: &str, label: &str, group: &str, on: bool) -> Param {
    pm(key, label, group, if on { 1.0 } else { 0.0 }, 0.0, 1.0, ParamKind::Toggle)
}
fn swing(key: &str, label: &str, group: &str, value: f32, max: f32) -> Param {
    pm(key, label, group, value, 0.0, max, ParamKind::Swing)
}
fn tpl(key: &str, params: Vec<Param>) -> Template {
    Template { key: key.into(), params }
}

/// Is this setting an open/shut control (door swing, drawer pull)?
pub fn is_open_param(p: &Param) -> bool {
    p.kind == ParamKind::Swing || p.key.ends_with("_open")
}

// ------------------------------------------------------------------ colours

const DARK_WOOD: [u8; 3] = [92, 60, 38];
const CUSHION: [u8; 3] = [168, 92, 92];
const TRIM: [u8; 3] = [235, 230, 220];
const DOOR: [u8; 3] = [120, 70, 40];
const BRASS: [u8; 3] = [200, 170, 90];

// ------------------------------------------------------------------ generators

/// Rebuild the parts of a template from its settings.
pub fn generate(t: &Template) -> Vec<Part> {
    let g = |k: &str| t.get(k);
    let on = |k: &str| t.on(k);
    let mut v: Vec<Part> = Vec::new();
    match t.key.as_str() {
        "sofa" => {
            let (w, d, sh, bh, ah) = (g("width"), g("depth"), g("seat_h"), g("back_h"), g("arm_h"));
            let n = g("seats").round().max(1.0);
            let leg = 0.12f32.min(sh * 0.3);
            let arm_w = if on("arms") { 0.18f32.min(w * 0.2) } else { 0.0 };
            let inner = (w - 2.0 * arm_w).max(0.3);
            let base_h = (sh - 0.12 - leg).max(0.05);
            v.push(bx(0.0, leg, 0.03, inner, base_h, d - 0.06));
            v.push(bx(0.0, leg, -d * 0.5 + 0.1, w, (bh - leg).max(0.1), 0.2));
            if arm_w > 0.0 {
                for s in [-1.0, 1.0] {
                    v.push(bx(s * (w * 0.5 - arm_w * 0.5), leg, 0.0, arm_w, (ah - leg).max(0.1), d));
                }
            }
            let cw = inner / n;
            for i in 0..n as usize {
                let x = -inner * 0.5 + cw * (i as f32 + 0.5);
                v.push(bx(x, leg + base_h, 0.1, cw - 0.02, 0.12, (d - 0.24).max(0.2)).c(CUSHION));
            }
            v.extend(legs4(w - 0.1, d - 0.1, leg, 0.06, DARK_WOOD));
        }
        "table" => {
            let (w, d, h) = (g("width"), g("depth"), g("height"));
            let top = g("top_t").max(0.015);
            if on("round") {
                v.push(cyl(0.0, h - top, 0.0, w, top, d));
                v.push(cyl(0.0, 0.04, 0.0, 0.12, h - top - 0.04, 0.12).c(DARK_WOOD));
                v.push(cyl(0.0, 0.0, 0.0, w * 0.45, 0.04, d * 0.45).c(DARK_WOOD));
            } else {
                v.push(bx(0.0, h - top, 0.0, w, top, d));
                v.extend(legs4(w - 0.05, d - 0.05, h - top, 0.06, DARK_WOOD));
                if on("shelf") {
                    v.push(bx(0.0, 0.12, 0.0, w - 0.12, 0.03, d - 0.12));
                }
            }
        }
        "chair" => {
            let (w, d, sh, bh) = (g("width"), g("depth"), g("seat_h"), g("back_h"));
            v.push(bx(0.0, sh - 0.05, 0.0, w, 0.05, d));
            v.push(bx(0.0, sh, -d * 0.5 + 0.02, w, (bh - sh).max(0.05), 0.04));
            v.extend(legs4(w - 0.02, d - 0.02, sh - 0.05, 0.04, DARK_WOOD));
            if on("arms") {
                for s in [-1.0, 1.0] {
                    v.push(bx(s * (w * 0.5 - 0.02), sh + 0.2, 0.0, 0.05, 0.04, d));
                    v.push(bx(s * (w * 0.5 - 0.02), sh, d * 0.5 - 0.04, 0.04, 0.2, 0.04));
                }
            }
        }
        "desk" => {
            let (w, d, h) = (g("width"), g("depth"), g("height"));
            let n = g("drawers").round().max(0.0) as usize;
            let s = if on("ped_right") { 1.0 } else { -1.0 };
            let pw = 0.45f32.min(w * 0.45);
            let px = s * (w * 0.5 - pw * 0.5 - 0.02);
            v.push(bx(0.0, h - 0.04, 0.0, w, 0.04, d));
            v.push(bx(px, 0.0, 0.0, pw, h - 0.04, d - 0.04));
            v.push(bx(-s * (w * 0.5 - 0.05), 0.0, 0.0, 0.06, h - 0.04, d - 0.04));
            if n > 0 {
                let dh = (h - 0.12) / n as f32;
                let out = g("drawer_open");
                for i in 0..n {
                    let y = h - 0.06 - (i as f32 + 1.0) * dh + 0.01;
                    let o = if i == 0 { out } else { 0.0 };
                    v.push(bx(px, y, d * 0.5 - 0.02 + 0.01 + o, pw - 0.05, dh - 0.02, 0.02).c([170, 140, 90]));
                    v.push(bx(px, y + dh * 0.45, d * 0.5 + 0.02 + o, 0.1, 0.02, 0.02).c(BRASS));
                    if o > 0.01 {
                        let body = (d - 0.1).min(0.5);
                        v.push(bx(px, y + 0.01, d * 0.5 - 0.01 + o - body * 0.5, pw - 0.08, dh - 0.05, body).c([150, 116, 74]));
                    }
                }
            }
        }
        "bed" => {
            let (w, l, h, hb) = (g("width"), g("length"), g("height"), g("head_h"));
            v.push(bx(0.0, 0.0, 0.0, w, 0.3, l).c(DARK_WOOD));
            v.push(bx(0.0, 0.3, 0.03, w - 0.05, (h - 0.3).max(0.05), l - 0.1).c([236, 232, 222]));
            v.push(bx(0.0, h, l * 0.125, w - 0.03, 0.04, l * 0.75).c([90, 110, 150]));
            let n = if w > 1.2 { 2 } else { 1 };
            for i in 0..n {
                let x = if n == 1 { 0.0 } else { (i as f32 - 0.5) * w * 0.47 };
                v.push(bx(x, h, -l * 0.5 + 0.25, (w * 0.37).min(0.6), 0.12, 0.35).c([245, 245, 240]));
            }
            v.push(bx(0.0, 0.0, -l * 0.5 + 0.02, w + 0.05, hb, 0.06).c(DARK_WOOD));
            if on("footboard") {
                v.push(bx(0.0, 0.0, l * 0.5 - 0.02, w + 0.05, (h + 0.15).min(hb), 0.06).c(DARK_WOOD));
            }
        }
        "shelf" => {
            let (w, h, d) = (g("width"), g("height"), g("depth"));
            let n = g("shelves").round().max(2.0) as usize;
            v.push(bx(-w * 0.5 + 0.02, 0.0, 0.0, 0.04, h, d));
            v.push(bx(w * 0.5 - 0.02, 0.0, 0.0, 0.04, h, d));
            v.push(bx(0.0, 0.0, -d * 0.5 + 0.01, w, h, 0.02));
            let cup = on("cupboard");
            let ch = (h * 0.4).min(0.85);
            let step = (h - 0.05) / (n - 1) as f32;
            let cols = [[150, 40, 40], [40, 80, 130], [60, 110, 60], [170, 140, 60], [90, 50, 110]];
            let inner = w - 0.08;
            for i in 0..n {
                let y = 0.02 + i as f32 * step;
                v.push(bx(0.0, y, 0.0, inner, 0.03, d - 0.02));
                if i + 1 < n && on("books") && (!cup || y >= ch - 0.05) {
                    let k = (inner / 0.145).floor().max(1.0) as usize;
                    for j in 0..k {
                        let x = -inner * 0.5 + 0.07 + j as f32 * (inner - 0.14) / (k.max(2) - 1) as f32;
                        let bh = (0.26 + ((i * 7 + j * 3) % 5) as f32 * 0.025).min(step - 0.06);
                        v.push(bx(x, y + 0.03, 0.01, 0.11, bh.max(0.05), (d - 0.11).max(0.05)).c(cols[(i + j) % 5]));
                    }
                }
            }
            if cup {
                let lw = inner * 0.5 - 0.005;
                let open = g("doors_open");
                let z = d * 0.5 + 0.012;
                for s in [-1.0f32, 1.0] {
                    let cx = s * inner * 0.25;
                    let hx = s * inner * 0.5;
                    let side = if s < 0.0 { -1 } else { 1 };
                    v.extend(hinged(
                        vec![bx(cx, 0.02, z, lw, ch - 0.04, 0.02), bx(cx - s * (lw * 0.5 - 0.05), ch * 0.5, z + 0.02, 0.02, 0.1, 0.02).c(BRASS)],
                        hx,
                        z,
                        side,
                        open,
                        true,
                        110.0,
                        "Cupboard doors",
                    ));
                }
            }
        }
        "lamp" => {
            let (h, sd, sh) = (g("height"), g("shade_d"), g("shade_h"));
            v.push(cyl(0.0, 0.0, 0.0, 0.3, 0.03, 0.3).c([50, 50, 50]));
            v.push(cyl(0.0, 0.03, 0.0, 0.03, (h - sh * 0.9 - 0.03).max(0.1), 0.03).c([60, 60, 60]));
            v.push(part(Shape::Cone, 0.0, h - sh, 0.0, sd, sh, sd).c([235, 220, 180]));
        }
        "piano" => {
            let (w, h, d) = (g("width"), g("height"), g("depth"));
            let bd = (d - 0.25).max(0.3);
            let bz = -d * 0.5 + bd * 0.5;
            let kz = -d * 0.5 + bd + 0.125;
            v.push(bx(0.0, 0.0, bz, w, h - 0.03, bd));
            v.push(lid(bx(0.0, h - 0.03, bz, w, 0.03, bd), g("lid_open"), 90.0, "Top lid"));
            v.push(bx(0.0, 0.65, kz, w - 0.05, 0.08, 0.25));
            v.push(bx(0.0, 0.73, kz + 0.01, w - 0.2, 0.02, 0.22).c([240, 240, 235]));
            for s in [-1.0, 1.0] {
                v.push(bx(s * (w * 0.5 - 0.08), 0.0, kz, 0.08, 0.65, 0.08));
            }
        }
        "wardrobe" => {
            let (w, h, d) = (g("width"), g("height"), g("depth"));
            let n = g("doors").round().clamp(1.0, 2.0) as usize;
            let open = g("doors_open");
            v.push(bx(0.0, 0.0, -0.015, w, h, d - 0.03));
            v.push(bx(0.0, h, 0.0, w + 0.04, 0.06, d + 0.02));
            v.push(bx(0.0, 0.08, -0.015, w - 0.06, 0.03, d - 0.1).c([60, 40, 26]));
            let z = d * 0.5 - 0.015;
            let lw = w / n as f32 - 0.01;
            for i in 0..n {
                let (side, hx) = if n == 1 || i == 0 { (-1i8, -w * 0.5) } else { (1, w * 0.5) };
                let s = side as f32;
                let cx = hx - s * (lw * 0.5 + 0.005);
                let kx = cx - s * (lw * 0.5 - 0.06);
                v.extend(hinged(
                    vec![bx(cx, 0.1, z, lw, h - 0.14, 0.025), bx(kx, h * 0.48, z + 0.025, 0.025, 0.14, 0.025).c(BRASS)],
                    hx,
                    z,
                    side,
                    open,
                    true,
                    110.0,
                    "Wardrobe doors",
                ));
            }
        }
        "chest" => {
            let (w, h, d) = (g("width"), g("height"), g("depth"));
            v.push(bx(0.0, 0.0, 0.0, w, h - 0.07, d));
            v.push(lid(bx(0.0, h - 0.07, 0.0, w + 0.02, 0.07, d + 0.02), g("lid_open"), 110.0, "Lid"));
            for s in [-1.0, 1.0] {
                v.push(bx(s * w * 0.3, 0.0, d * 0.5 + 0.006, 0.05, h - 0.07, 0.012).m(Material::Metal));
            }
        }
        "flat" => {
            let (w, h, th) = (g("width"), g("height"), g("thick"));
            v.push(bx(0.0, 0.0, 0.0, w, h, th));
            if on("brace") {
                let jd = (h * 0.32).clamp(0.3, 1.2);
                for s in [-1.0, 1.0] {
                    v.push(part(Shape::Ramp, s * w * 0.3, 0.0, -th * 0.5 - jd * 0.5, 0.04, h * 0.6, jd).r(180.0).c([120, 100, 80]));
                    v.push(bx(s * w * 0.3, 0.0, -th * 0.5 - jd + 0.12, 0.2, 0.12, 0.3).c([40, 40, 44]));
                }
            }
        }
        "doorflat" => {
            let (w, h, th) = (g("width"), g("height"), g("thick"));
            let dw = g("door_w").min(w - 0.1).max(0.4);
            let dh = g("door_h").min(h - 0.05).max(1.0);
            let lim = (w * 0.5 - dw * 0.5 - 0.05).max(0.0);
            let dx = g("door_x").clamp(-lim, lim);
            let lw = dx - dw * 0.5 + w * 0.5;
            if lw > 0.005 {
                v.push(bx(-w * 0.5 + lw * 0.5, 0.0, 0.0, lw, h, th));
            }
            let rw = w * 0.5 - (dx + dw * 0.5);
            if rw > 0.005 {
                v.push(bx(w * 0.5 - rw * 0.5, 0.0, 0.0, rw, h, th));
            }
            if h - dh > 0.005 {
                v.push(bx(dx, dh, 0.0, dw, h - dh, th));
            }
            if on("arch") {
                let at = th + 0.04;
                v.push(bx(dx - dw * 0.5 - 0.03, 0.0, 0.0, 0.06, dh + 0.06, at).c(TRIM));
                v.push(bx(dx + dw * 0.5 + 0.03, 0.0, 0.0, 0.06, dh + 0.06, at).c(TRIM));
                v.push(bx(dx, dh, 0.0, dw + 0.12, 0.08, at).c(TRIM));
            }
            let open = g("door_open");
            let out = on("opens_out");
            let leaf = |cx: f32, lw: f32, side: i8| -> Vec<Part> {
                let s = side as f32;
                let hx = cx + s * lw * 0.5;
                let kx = cx - s * (lw * 0.5 - 0.08);
                hinged(
                    vec![
                        bx(cx, 0.005, 0.0, lw, dh - 0.015, 0.04).c(DOOR),
                        bx(cx, dh * 0.55, 0.022, lw - 0.16, dh * 0.35, 0.006).c([104, 60, 34]),
                        bx(cx, dh * 0.1, 0.022, lw - 0.16, dh * 0.35, 0.006).c([104, 60, 34]),
                        sph(kx, 0.95, 0.04, 0.06, 0.06, 0.05).c(BRASS),
                        sph(kx, 0.95, -0.04, 0.06, 0.06, 0.05).c(BRASS),
                    ],
                    hx,
                    0.0,
                    side,
                    open,
                    out,
                    110.0,
                    "Door",
                )
            };
            if on("double") {
                let lw = (dw - 0.02) * 0.5;
                v.extend(leaf(dx - dw * 0.25, lw, -1));
                v.extend(leaf(dx + dw * 0.25, lw, 1));
            } else {
                let side = if on("hinge_right") { 1 } else { -1 };
                v.extend(leaf(dx, dw - 0.02, side));
            }
        }
        "windowflat" => {
            let (w, h, th) = (g("width"), g("height"), g("thick"));
            let ww = g("win_w").min(w - 0.1).max(0.3);
            let sill = g("sill_h").clamp(0.0, h - 0.4);
            let wh = g("win_h").min(h - sill - 0.05).max(0.3);
            let lim = (w * 0.5 - ww * 0.5 - 0.05).max(0.0);
            let wx = g("win_x").clamp(-lim, lim);
            if sill > 0.005 {
                v.push(bx(0.0, 0.0, 0.0, w, sill, th));
            }
            let top = sill + wh;
            if h - top > 0.005 {
                v.push(bx(0.0, top, 0.0, w, h - top, th));
            }
            let lw = wx - ww * 0.5 + w * 0.5;
            if lw > 0.005 {
                v.push(bx(-w * 0.5 + lw * 0.5, sill, 0.0, lw, wh, th));
            }
            let rw = w * 0.5 - (wx + ww * 0.5);
            if rw > 0.005 {
                v.push(bx(w * 0.5 - rw * 0.5, sill, 0.0, rw, wh, th));
            }
            if on("sill") {
                v.push(bx(wx, sill - 0.05, 0.05, ww + 0.12, 0.05, th + 0.1).c(TRIM));
            }
            let open = g("win_open");
            let out = on("opens_out");
            let leaf_w = ww * 0.5 - 0.01;
            for side in [-1i8, 1] {
                let s = side as f32;
                let hx = wx + s * ww * 0.5;
                let cx = hx - s * (leaf_w * 0.5 + 0.005);
                let mut ps = vec![bx(cx, sill + 0.01, 0.0, leaf_w, wh - 0.02, 0.02).m(Material::Glass)];
                // casement frame
                ps.push(bx(cx - leaf_w * 0.5 + 0.02, sill + 0.01, 0.0, 0.04, wh - 0.02, 0.04).c(TRIM));
                ps.push(bx(cx + leaf_w * 0.5 - 0.02, sill + 0.01, 0.0, 0.04, wh - 0.02, 0.04).c(TRIM));
                ps.push(bx(cx, sill + 0.01, 0.0, leaf_w, 0.04, 0.04).c(TRIM));
                ps.push(bx(cx, sill + wh - 0.05, 0.0, leaf_w, 0.04, 0.04).c(TRIM));
                if on("bars") {
                    ps.push(bx(cx, sill + wh * 0.5, 0.0, leaf_w, 0.03, 0.035).c(TRIM));
                }
                v.extend(hinged(ps, hx, 0.0, side, open, out, 110.0, "Casements"));
            }
        }
        "arch" => {
            let (w, ow, sh, th, h) = (g("width"), g("open_w"), g("spring_h"), g("thick"), g("height"));
            let ow = ow.min(w - 0.1).max(0.3);
            let pw = (w - ow) * 0.5;
            let ytop = sh + ow * 0.5;
            for s in [-1.0, 1.0] {
                v.push(bx(s * (ow * 0.5 + pw * 0.5), 0.0, 0.0, pw, ytop, th));
                v.push(bx(s * (ow * 0.5 + pw * 0.5), sh - 0.05, 0.0, pw + 0.1, 0.1, th + 0.06));
            }
            v.push(part(Shape::ArchTop, 0.0, sh, 0.0, ow, ow * 0.5, th));
            if h > ytop + 0.005 {
                v.push(bx(0.0, ytop, 0.0, w, h - ytop, th));
            }
        }
        "column" => {
            let (h, dia) = (g("height"), g("diameter"));
            let pl = if on("plinth") { 0.2 } else { 0.0 };
            let cap = if on("capital") { 0.2 } else { 0.0 };
            if pl > 0.0 {
                v.push(bx(0.0, 0.0, 0.0, dia * 1.4, pl, dia * 1.4));
            }
            v.push(cyl(0.0, pl, 0.0, dia, (h - pl - cap).max(0.1), dia));
            if cap > 0.0 {
                v.push(bx(0.0, h - cap, 0.0, dia * 1.4, cap, dia * 1.4));
            }
        }
        "angled" => {
            let (pw, h, th, a) = (g("panel_w"), g("height"), g("thick"), g("angle"));
            let c = a.to_radians().cos();
            v.push(bx(-pw * 0.5 * c, 0.0, 0.0, pw, h, th).r(a));
            v.push(bx(pw * 0.5 * c, 0.0, 0.0, pw, h, th).r(-a));
        }
        "platform" => {
            let (w, d, h) = (g("width"), g("depth"), g("height"));
            v.push(bx(0.0, 0.0, 0.0, w, h, d));
            if on("trap") {
                let tw = g("trap_w").min(w - 0.1).min(d - 0.1).max(0.3);
                v.push(bx(0.0, h, 0.0, tw, 0.004, tw).c([8, 8, 10]));
                v.push(lid(bx(0.0, h + 0.004, 0.0, tw, 0.03, tw).c([90, 86, 82]), g("trap_open"), 110.0, "Trap door"));
            }
        }
        "stairs" => {
            let (w, rise, going) = (g("width"), g("rise"), g("going"));
            let n = g("treads").round().clamp(1.0, 30.0) as u8;
            v.push(part(Shape::Stairs(n), 0.0, 0.0, 0.0, w, rise, going));
            if on("stringers") {
                for s in [-1.0, 1.0] {
                    v.push(part(Shape::Ramp, s * (w * 0.5 + 0.02), 0.0, 0.0, 0.04, rise + 0.05, going).c([60, 58, 56]));
                }
            }
        }
        "splitlevel" => {
            let (w, rh, ld, sw, sg) = (g("width"), g("rise"), g("landing_d"), g("stair_w"), g("going"));
            let n = g("treads").round().clamp(1.0, 30.0) as u8;
            let sw = sw.min(w);
            let lim = (w - sw) * 0.5;
            let sx = g("stair_x").clamp(-lim, lim);
            let total = ld + sg;
            v.push(bx(0.0, 0.0, -total * 0.5 + ld * 0.5, w, rh, ld));
            v.push(part(Shape::Stairs(n), sx, 0.0, -total * 0.5 + ld + sg * 0.5, sw, rh, sg));
        }
        "escape" => {
            let (w, rise, going, ld) = (g("width"), g("rise"), g("going"), g("landing_d"));
            let n = g("treads").round().clamp(1.0, 30.0) as u8;
            let total = going + ld;
            let zl = -total * 0.5 + ld * 0.5;
            v.push(part(Shape::Stairs(n), 0.0, 0.0, -total * 0.5 + ld + going * 0.5, w, rise, going));
            v.push(bx(0.0, 0.0, zl, w, rise, ld));
            if on("rail") {
                let y = [200, 200, 60];
                v.push(bx(-w * 0.5 + 0.015, rise, zl, 0.03, 0.9, ld).c(y));
                v.push(bx(0.0, rise, -total * 0.5 + 0.015, w, 0.9, 0.03).c(y));
                v.push(bx(w * 0.5 - 0.015, rise, zl, 0.03, 0.9, ld).c(y));
            }
        }
        "ramp" => {
            v.push(part(Shape::Ramp, 0.0, 0.0, 0.0, g("width"), g("rise"), g("length")));
        }
        "tree" => {
            let (h, cw, tr) = (g("height"), g("canopy"), g("trunk"));
            let s = h / 3.7;
            let k = cw / 1.9;
            v.push(cyl(0.0, 0.0, 0.0, tr, 2.2 * s, tr).c([90, 64, 42]));
            v.push(sph(0.0, 1.6 * s, 0.0, 1.9 * k, 1.6 * s, 1.9 * k));
            v.push(sph(0.35 * k, 2.5 * s, 0.1 * k, 1.3 * k, 1.2 * s, 1.3 * k).c([84, 136, 70]));
            v.push(sph(-0.4 * k, 2.3 * s, -0.2 * k, 1.1 * k, 1.0 * s, 1.1 * k).c([60, 108, 52]));
        }
        "rock" => {
            let (kx, ky, kz) = (g("width") / 1.55, g("height") / 0.9, g("depth") / 1.1);
            v.push(sph(-0.075 * kx, 0.0, -0.05 * kz, 1.4 * kx, 0.9 * ky, 1.0 * kz));
            v.push(sph(0.375 * kx, 0.0, 0.2 * kz, 0.8 * kx, 0.6 * ky, 0.7 * kz));
        }
        "lamppost" => {
            let (h, ls, bd) = (g("height"), g("lantern"), g("base_d"));
            let lh = ls * 1.25;
            let post_top = (h - 0.25 - lh).max(0.6);
            v.push(cyl(0.0, 0.0, 0.0, bd, 0.3, bd));
            v.push(cyl(0.0, 0.3, 0.0, 0.1, post_top - 0.3, 0.1));
            v.push(bx(0.0, post_top, 0.0, ls, lh, ls).m(Material::Glass).c([255, 230, 160]));
            v.push(part(Shape::Cone, 0.0, post_top + lh, 0.0, ls * 1.4, 0.25, ls * 1.4));
        }
        "bench" => {
            let (l, sh, d, bh) = (g("length"), g("seat_h"), g("depth"), g("back_h"));
            v.push(bx(0.0, sh - 0.05, 0.0, l, 0.05, d));
            if on("back") {
                v.push(bx(0.0, sh + 0.08, -d * 0.5 + 0.02, l, (bh - sh - 0.08).max(0.1), 0.04));
            }
            v.extend(legs4(l - 0.1, d - 0.05, sh - 0.05, 0.06, [40, 40, 40]));
        }
        "fence" => {
            let (l, h, sp) = (g("length"), g("height"), g("spacing").max(0.12));
            let gate = on("gate");
            let gw = g("gate_w").min(l - 0.2).max(0.5);
            let lim = (l * 0.5 - gw * 0.5 - 0.05).max(0.0);
            let gx = g("gate_x").clamp(-lim, lim);
            let (g0, g1) = (gx - gw * 0.5, gx + gw * 0.5);
            let mut x = -l * 0.5 + 0.045;
            while x <= l * 0.5 - 0.04 {
                if !gate || x < g0 - 0.05 || x > g1 + 0.05 {
                    v.push(bx(x, 0.0, 0.0, 0.09, h, 0.03));
                }
                x += sp;
            }
            let rail = |v: &mut Vec<Part>, a: f32, b: f32| {
                if b - a > 0.02 {
                    v.push(bx((a + b) * 0.5, h * 0.25, -0.03, b - a, 0.08, 0.03));
                    v.push(bx((a + b) * 0.5, h * 0.75, -0.03, b - a, 0.08, 0.03));
                }
            };
            if gate {
                rail(&mut v, -l * 0.5, g0 - 0.05);
                rail(&mut v, g1 + 0.05, l * 0.5);
                for px in [g0 - 0.045, g1 + 0.045] {
                    v.push(bx(px, 0.0, -0.015, 0.09, h + 0.12, 0.09));
                }
                let mut ps = Vec::new();
                let k = ((gw - 0.1) / sp).floor().max(1.0) as usize;
                for i in 0..=k {
                    let x = g0 + 0.05 + i as f32 * (gw - 0.1) / k as f32;
                    ps.push(bx(x, 0.05, 0.0, 0.09, h - 0.05, 0.03));
                }
                ps.push(bx(gx, h * 0.25, -0.03, gw - 0.02, 0.08, 0.03));
                ps.push(bx(gx, h * 0.75, -0.03, gw - 0.02, 0.08, 0.03));
                v.extend(hinged(ps, g0, 0.0, -1, g("gate_open"), on("opens_out"), 110.0, "Gate"));
            } else {
                rail(&mut v, -l * 0.5, l * 0.5);
            }
        }
        "block" => {
            v.push(bx(0.0, 0.0, 0.0, g("width"), g("height"), g("depth")));
        }
        "actor" => {
            let s = g("height") / 1.75;
            let k = g("build") / 0.42;
            v.push(bx(-0.1 * k, 0.0, 0.0, 0.14 * k, 0.85 * s, 0.18 * k).c([50, 50, 70]));
            v.push(bx(0.1 * k, 0.0, 0.0, 0.14 * k, 0.85 * s, 0.18 * k).c([50, 50, 70]));
            v.push(bx(0.0, 0.85 * s, 0.0, 0.42 * k, 0.6 * s, 0.24 * k));
            v.push(bx(-0.27 * k, 0.8 * s, 0.0, 0.1 * k, 0.62 * s, 0.12 * k));
            v.push(bx(0.27 * k, 0.8 * s, 0.0, 0.1 * k, 0.62 * s, 0.12 * k));
            v.push(sph(0.0, 1.5 * s, 0.0, 0.22 * s, 0.25 * s, 0.24 * s).c([224, 186, 150]));
        }
        "spike" => {
            let (sz, tw) = (g("size"), g("tape_w"));
            v.push(bx(0.0, 0.0, 0.0, sz, 0.005, tw));
            v.push(bx(0.0, 0.0, 0.0, tw, 0.005, sz));
        }
        _ => {}
    }
    v
}

fn item(name: &str, cat: &str, mat: Material, color: [u8; 3], t: Template) -> LibItem {
    let parts = generate(&t);
    LibItem { name: name.into(), category: cat.into(), material: mat, color, parts, template: Some(t) }
}

pub const CATEGORIES: [&str; 5] = ["Furniture", "Walls & Doors", "Levels & Stairs", "Exterior", "Rehearsal"];

const S: &str = "Size";

fn sofa(w: f32, d: f32, back: f32, seats: f32) -> Template {
    tpl(
        "sofa",
        vec![
            len("width", "Width", S, w, 0.6, 4.5),
            len("depth", "Depth", S, d, 0.5, 1.4),
            len("seat_h", "Seat height", S, 0.54, 0.3, 0.8),
            len("back_h", "Back height", S, back, 0.4, 1.5),
            len("arm_h", "Arm height", S, 0.62, 0.3, 1.0),
            cnt("seats", "Seat cushions", "Details", seats, 1.0, 5.0),
            tog("arms", "Arms", "Details", true),
        ],
    )
}

fn table(w: f32, d: f32, h: f32, shelf: bool) -> Template {
    tpl(
        "table",
        vec![
            len("width", "Width", S, w, 0.3, 4.0),
            len("depth", "Depth", S, d, 0.3, 2.0),
            len("height", "Height", S, h, 0.2, 1.2),
            len("top_t", "Top thickness", "Details", 0.05, 0.015, 0.15),
            tog("round", "Round / oval top", "Details", false),
            tog("shelf", "Lower shelf", "Details", shelf),
        ],
    )
}

fn flat(w: f32, h: f32, th: f32) -> Template {
    tpl(
        "flat",
        vec![
            len("width", "Width", S, w, 0.3, 12.0),
            len("height", "Height", S, h, 0.3, 10.0),
            len("thick", "Thickness", S, th, 0.02, 0.4),
            tog("brace", "Stage braces & weights behind", "Details", false),
        ],
    )
}

fn door_flat(w: f32, dw: f32, double: bool, open: f32) -> Template {
    tpl(
        "doorflat",
        vec![
            len("width", "Flat width", "Flat", w, 0.6, 12.0),
            len("height", "Flat height", "Flat", 2.7, 1.2, 10.0),
            len("thick", "Thickness", "Flat", 0.08, 0.02, 0.4),
            len("door_w", "Door opening width", "Door", dw, 0.4, 4.0),
            len("door_h", "Door height", "Door", 2.1, 1.0, 6.0),
            len("door_x", "Door position (+ = right)", "Door", 0.0, -6.0, 6.0),
            tog("double", "Double doors", "Door", double),
            tog("hinge_right", "Hinged on the right", "Door", false),
            tog("opens_out", "Opens towards the front", "Door", false),
            tog("arch", "Architrave (trim)", "Door", true),
            swing("door_open", "Door open", "Door", open, 110.0),
        ],
    )
}

pub fn presets() -> Vec<LibItem> {
    let mut v = Vec::new();
    let fab = Material::Fabric;
    let tim = Material::Timber;
    let pw = Material::PaintedWood;

    // ---------- Furniture ----------
    v.push(item("Sofa (3-seat)", "Furniture", fab, [110, 52, 62], sofa(2.0, 0.85, 0.82, 2.0)));
    v.push(item("Armchair", "Furniture", fab, [70, 90, 110], sofa(0.92, 0.85, 0.97, 1.0)));
    v.push(item("Coffee table", "Furniture", tim, [130, 88, 54], table(1.1, 0.6, 0.43, true)));
    v.push(item("Dining table", "Furniture", tim, [130, 88, 54], table(1.8, 0.9, 0.76, false)));
    v.push(item(
        "Dining chair",
        "Furniture",
        tim,
        [120, 80, 50],
        tpl(
            "chair",
            vec![
                len("width", "Seat width", S, 0.44, 0.3, 0.9),
                len("depth", "Seat depth", S, 0.44, 0.3, 0.9),
                len("seat_h", "Seat height", S, 0.48, 0.25, 0.9),
                len("back_h", "Back top height", S, 0.93, 0.5, 1.6),
                tog("arms", "Arms (carver)", "Details", false),
            ],
        ),
    ));
    v.push(item(
        "Writing desk",
        "Furniture",
        tim,
        [110, 72, 44],
        tpl(
            "desk",
            vec![
                len("width", "Width", S, 1.4, 0.8, 3.0),
                len("depth", "Depth", S, 0.7, 0.4, 1.2),
                len("height", "Height", S, 0.76, 0.5, 1.1),
                cnt("drawers", "Drawers", "Drawers", 2.0, 0.0, 4.0),
                tog("ped_right", "Drawers on the right", "Drawers", true),
                len("drawer_open", "Top drawer pulled out", "Drawers", 0.0, 0.0, 0.4),
            ],
        ),
    ));
    v.push(item(
        "Double bed",
        "Furniture",
        tim,
        DARK_WOOD,
        tpl(
            "bed",
            vec![
                len("width", "Width", S, 1.5, 0.8, 2.2),
                len("length", "Length", S, 2.0, 1.6, 2.4),
                len("height", "Mattress top", S, 0.52, 0.35, 0.9),
                len("head_h", "Headboard height", "Details", 1.1, 0.5, 2.0),
                tog("footboard", "Footboard", "Details", false),
            ],
        ),
    ));
    v.push(item(
        "Bookshelf",
        "Furniture",
        tim,
        [100, 66, 40],
        tpl(
            "shelf",
            vec![
                len("width", "Width", S, 1.0, 0.4, 3.0),
                len("height", "Height", S, 2.0, 0.6, 3.5),
                len("depth", "Depth", S, 0.35, 0.2, 0.7),
                cnt("shelves", "Shelves", "Details", 5.0, 2.0, 10.0),
                tog("books", "Books", "Details", true),
                tog("cupboard", "Cupboard doors below", "Cupboard", false),
                swing("doors_open", "Cupboard doors open", "Cupboard", 0.0, 110.0),
            ],
        ),
    ));
    v.push(item(
        "Wardrobe",
        "Furniture",
        tim,
        [110, 74, 46],
        tpl(
            "wardrobe",
            vec![
                len("width", "Width", S, 1.2, 0.5, 3.0),
                len("height", "Height", S, 2.0, 1.2, 2.8),
                len("depth", "Depth", S, 0.6, 0.35, 0.9),
                cnt("doors", "Doors (1 or 2)", "Doors", 2.0, 1.0, 2.0),
                swing("doors_open", "Doors open", "Doors", 0.0, 110.0),
            ],
        ),
    ));
    v.push(item(
        "Chest / trunk",
        "Furniture",
        tim,
        [120, 80, 46],
        tpl(
            "chest",
            vec![
                len("width", "Width", S, 0.9, 0.4, 2.0),
                len("height", "Height", S, 0.5, 0.25, 1.0),
                len("depth", "Depth", S, 0.5, 0.3, 1.0),
                swing("lid_open", "Lid open", "Lid", 0.0, 110.0),
            ],
        ),
    ));
    v.push(item(
        "Standard lamp",
        "Furniture",
        Material::Metal,
        [60, 60, 60],
        tpl(
            "lamp",
            vec![
                len("height", "Height", S, 1.65, 0.6, 2.4),
                len("shade_d", "Shade width", S, 0.45, 0.2, 0.9),
                len("shade_h", "Shade height", S, 0.35, 0.15, 0.6),
            ],
        ),
    ));
    v.push(item(
        "Upright piano",
        "Furniture",
        tim,
        [40, 26, 20],
        tpl(
            "piano",
            vec![
                len("width", "Width", S, 1.5, 1.1, 1.8),
                len("height", "Height", S, 1.25, 1.0, 1.5),
                len("depth", "Depth (with keys)", S, 0.7, 0.55, 0.9),
                swing("lid_open", "Top lid open", "Lid", 0.0, 90.0),
            ],
        ),
    ));

    // ---------- Walls & doors ----------
    let flat_col = [190, 180, 150];
    v.push(item("Flat 1.2 x 2.4 m", "Walls & Doors", pw, flat_col, flat(1.2, 2.4, 0.06)));
    v.push(item("Wide wall 3.6 x 3 m", "Walls & Doors", pw, flat_col, flat(3.6, 3.0, 0.08)));
    v.push(item("Door flat", "Walls & Doors", pw, flat_col, door_flat(2.2, 1.0, false, 35.0)));
    v.push(item("Double doors flat", "Walls & Doors", pw, flat_col, door_flat(3.0, 1.6, true, 0.0)));
    v.push(item(
        "Window flat",
        "Walls & Doors",
        pw,
        flat_col,
        tpl(
            "windowflat",
            vec![
                len("width", "Flat width", "Flat", 2.0, 0.6, 12.0),
                len("height", "Flat height", "Flat", 2.7, 1.2, 10.0),
                len("thick", "Thickness", "Flat", 0.08, 0.02, 0.4),
                len("win_w", "Window width", "Window", 1.0, 0.3, 6.0),
                len("win_h", "Window height", "Window", 1.2, 0.3, 6.0),
                len("sill_h", "Sill height", "Window", 0.9, 0.0, 4.0),
                len("win_x", "Window position (+ = right)", "Window", 0.0, -6.0, 6.0),
                tog("sill", "Window sill", "Window", true),
                tog("bars", "Glazing bars", "Window", true),
                tog("opens_out", "Opens towards the front", "Window", true),
                swing("win_open", "Casements open", "Window", 0.0, 110.0),
            ],
        ),
    ));
    v.push(item(
        "Archway",
        "Walls & Doors",
        Material::Stone,
        [200, 190, 170],
        tpl(
            "arch",
            vec![
                len("width", "Overall width", S, 2.5, 0.8, 10.0),
                len("height", "Overall height", S, 3.3, 1.5, 10.0),
                len("thick", "Thickness", S, 0.3, 0.05, 1.0),
                len("open_w", "Opening width", "Opening", 1.5, 0.3, 8.0),
                len("spring_h", "Height to arch spring", "Opening", 2.2, 0.5, 6.0),
            ],
        ),
    ));
    v.push(item(
        "Column",
        "Walls & Doors",
        Material::Stone,
        [215, 208, 195],
        tpl(
            "column",
            vec![
                len("height", "Height", S, 3.0, 0.5, 10.0),
                len("diameter", "Diameter", S, 0.4, 0.1, 1.5),
                tog("plinth", "Plinth (base)", "Details", true),
                tog("capital", "Capital (top)", "Details", true),
            ],
        ),
    ));
    v.push(item(
        "Angled wall unit",
        "Walls & Doors",
        pw,
        flat_col,
        tpl(
            "angled",
            vec![
                len("panel_w", "Each panel width", S, 1.8, 0.3, 6.0),
                len("height", "Height", S, 2.7, 0.5, 10.0),
                len("thick", "Thickness", S, 0.08, 0.02, 0.4),
                ang("angle", "Fold angle", "Details", 25.0, 0.0, 80.0),
            ],
        ),
    ));

    // ---------- Levels & stairs ----------
    let platform = |w: f32, d: f32, h: f32| {
        tpl(
            "platform",
            vec![
                len("width", "Width", S, w, 0.2, 12.0),
                len("depth", "Depth", S, d, 0.2, 12.0),
                len("height", "Height", S, h, 0.05, 4.0),
                tog("trap", "Trap door in top", "Trap", false),
                len("trap_w", "Trap size", "Trap", 0.6, 0.3, 2.0),
                swing("trap_open", "Trap open", "Trap", 0.0, 110.0),
            ],
        )
    };
    v.push(item("Rostrum 2.4 x 1.2 x 0.4 m", "Levels & Stairs", pw, [60, 60, 66], platform(2.4, 1.2, 0.4)));
    v.push(item("Platform 2.4 x 2.4 x 0.9 m", "Levels & Stairs", pw, [70, 70, 76], platform(2.4, 2.4, 0.9)));
    v.push(item(
        "Stair unit (4 treads)",
        "Levels & Stairs",
        pw,
        [80, 76, 72],
        tpl(
            "stairs",
            vec![
                len("width", "Width", S, 1.0, 0.4, 6.0),
                len("rise", "Total rise", S, 0.8, 0.1, 5.0),
                len("going", "Run (depth)", S, 1.2, 0.2, 8.0),
                cnt("treads", "Treads", "Details", 4.0, 1.0, 30.0),
                tog("stringers", "Side stringers", "Details", false),
            ],
        ),
    ));
    v.push(item(
        "Split-level riser + stairs",
        "Levels & Stairs",
        pw,
        [70, 70, 76],
        tpl(
            "splitlevel",
            vec![
                len("width", "Riser width", "Riser", 2.4, 0.6, 12.0),
                len("landing_d", "Riser depth", "Riser", 1.2, 0.3, 8.0),
                len("rise", "Riser height", "Riser", 0.6, 0.1, 4.0),
                len("stair_w", "Stair width", "Stairs", 1.0, 0.4, 12.0),
                len("going", "Stair run", "Stairs", 0.9, 0.2, 6.0),
                cnt("treads", "Treads", "Stairs", 3.0, 1.0, 20.0),
                len("stair_x", "Stair position (+ = right)", "Stairs", 0.0, -6.0, 6.0),
            ],
        ),
    ));
    v.push(item(
        "Escape stairs (wing)",
        "Levels & Stairs",
        Material::Metal,
        [70, 72, 78],
        tpl(
            "escape",
            vec![
                len("width", "Width", S, 0.9, 0.5, 3.0),
                len("rise", "Height", S, 1.8, 0.3, 6.0),
                len("going", "Stair run", S, 2.0, 0.4, 8.0),
                len("landing_d", "Landing depth", S, 1.0, 0.4, 4.0),
                cnt("treads", "Treads", "Details", 8.0, 1.0, 30.0),
                tog("rail", "Safety rails", "Details", true),
            ],
        ),
    ));
    v.push(item(
        "Ramp 1.2 x 2.4 m",
        "Levels & Stairs",
        pw,
        [80, 76, 72],
        tpl(
            "ramp",
            vec![len("width", "Width", S, 1.2, 0.3, 8.0), len("rise", "Rise", S, 0.4, 0.05, 3.0), len("length", "Length", S, 2.4, 0.3, 12.0)],
        ),
    ));

    // ---------- Exterior ----------
    v.push(item(
        "Tree",
        "Exterior",
        Material::Foliage,
        [70, 120, 60],
        tpl(
            "tree",
            vec![
                len("height", "Height", S, 3.7, 1.0, 10.0),
                len("canopy", "Canopy width", S, 1.9, 0.5, 8.0),
                len("trunk", "Trunk width", S, 0.3, 0.08, 1.5),
            ],
        ),
    ));
    v.push(item(
        "Rock",
        "Exterior",
        Material::Stone,
        [130, 124, 116],
        tpl(
            "rock",
            vec![len("width", "Width", S, 1.55, 0.2, 6.0), len("height", "Height", S, 0.9, 0.1, 4.0), len("depth", "Depth", S, 1.1, 0.2, 6.0)],
        ),
    ));
    v.push(item(
        "Lamppost",
        "Exterior",
        Material::Metal,
        [40, 44, 40],
        tpl(
            "lamppost",
            vec![
                len("height", "Height", S, 3.95, 1.5, 7.0),
                len("lantern", "Lantern size", S, 0.32, 0.15, 0.8),
                len("base_d", "Base width", S, 0.3, 0.15, 0.8),
            ],
        ),
    ));
    v.push(item(
        "Park bench",
        "Exterior",
        tim,
        [130, 90, 50],
        tpl(
            "bench",
            vec![
                len("length", "Length", S, 1.6, 0.5, 5.0),
                len("depth", "Seat depth", S, 0.45, 0.25, 0.8),
                len("seat_h", "Seat height", S, 0.47, 0.25, 0.8),
                tog("back", "Backrest", "Details", true),
                len("back_h", "Back top height", "Details", 0.9, 0.6, 1.4),
            ],
        ),
    ));
    let fence = |gate: bool, l: f32| {
        tpl(
            "fence",
            vec![
                len("length", "Length", S, l, 0.6, 12.0),
                len("height", "Height", S, 1.0, 0.4, 2.5),
                len("spacing", "Picket spacing", S, 0.25, 0.12, 0.6),
                tog("gate", "Gate", "Gate", gate),
                len("gate_w", "Gate width", "Gate", 0.9, 0.5, 3.0),
                len("gate_x", "Gate position (+ = right)", "Gate", 0.0, -6.0, 6.0),
                tog("opens_out", "Opens towards the front", "Gate", false),
                swing("gate_open", "Gate open", "Gate", 0.0, 110.0),
            ],
        )
    };
    v.push(item("Picket fence", "Exterior", pw, [235, 235, 228], fence(false, 2.2)));
    v.push(item("Garden gate in fence", "Exterior", pw, [235, 235, 228], fence(true, 3.0)));

    // ---------- Rehearsal ----------
    let block = |w: f32| tpl("block", vec![len("width", "Width", S, w, 0.1, 6.0), len("height", "Height", S, 0.45, 0.05, 3.0), len("depth", "Depth", S, 0.45, 0.1, 6.0)]);
    v.push(item("Rehearsal block", "Rehearsal", Material::Masking, [36, 36, 40], block(0.45)));
    v.push(item("Long rehearsal block", "Rehearsal", Material::Masking, [36, 36, 40], block(0.9)));
    v.push(item(
        "Actor (scale figure 1.75 m)",
        "Rehearsal",
        fab,
        [70, 110, 160],
        tpl("actor", vec![len("height", "Height", S, 1.75, 0.9, 2.2), len("build", "Shoulder width", S, 0.42, 0.3, 0.6)]),
    ));
    v.push(item(
        "Spike mark (floor tape)",
        "Rehearsal",
        pw,
        [240, 200, 40],
        tpl("spike", vec![len("size", "Mark size", S, 0.3, 0.05, 1.0), len("tape_w", "Tape width", S, 0.05, 0.012, 0.1)]),
    ));
    v
}

pub fn make_piece(item: &LibItem, id: u32, pos: [f32; 3]) -> Piece {
    let (lo, hi) = item.bounds();
    let size = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    // Re-centre the parts so the piece origin is the centre of its footprint, base at 0.
    let cx = (lo[0] + hi[0]) * 0.5;
    let cz = (lo[2] + hi[2]) * 0.5;
    let parts = item
        .parts
        .iter()
        .map(|p| {
            let mut q = p.clone();
            q.offset[0] -= cx;
            q.offset[1] -= lo[1];
            q.offset[2] -= cz;
            q
        })
        .collect();
    Piece {
        id,
        name: item.name.clone(),
        category: item.category.clone(),
        parts,
        base_size: size,
        size,
        pos,
        rot: 0.0,
        material: item.material,
        color: item.color,
        visible: true,
        locked: false,
        on_revolve: false,
        flown_on: None,
        template: item.template.clone(),
        hinged_to: None,
    }
}

/// Rebuild a piece's parts from its settings, keeping any overall stretch and keeping the
/// piece where it stands (if its footprint centre moves, the piece is shifted to match).
pub fn regenerate(p: &mut Piece) {
    let Some(t) = &p.template else { return };
    let parts = generate(t);
    if parts.is_empty() {
        return;
    }
    let (lo, hi) = parts_bounds(&parts);
    let size = [hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]];
    let k = [0, 1, 2].map(|i| p.size[i] / p.base_size[i].max(1e-3));
    let cx = (lo[0] + hi[0]) * 0.5;
    let cz = (lo[2] + hi[2]) * 0.5;
    p.parts = parts
        .into_iter()
        .map(|mut q| {
            q.offset[0] -= cx;
            q.offset[1] -= lo[1];
            q.offset[2] -= cz;
            q
        })
        .collect();
    let d = v3(cx * k[0], 0.0, cz * k[2]).rot_y(p.rot);
    p.pos[0] += d.x;
    p.pos[2] += d.z;
    p.base_size = size;
    p.size = [size[0] * k[0], size[1] * k[1], size[2] * k[2]];
}

/// Set one setting of a templated piece and rebuild it. Returns false if it has no such setting.
pub fn set_param(p: &mut Piece, key: &str, value: f32) -> bool {
    let ok = p.template.as_mut().map(|t| t.set(key, value)).unwrap_or(false);
    if ok {
        regenerate(p);
    }
    ok
}

/// Does the piece have doors, lids, gates or drawers that open?
pub fn has_moving_parts(p: &Piece) -> bool {
    match &p.template {
        Some(t) => t.params.iter().any(is_open_param),
        None => p.parts.iter().any(|q| q.hinge.is_some()),
    }
}

/// Is anything on the piece open?
pub fn is_open(p: &Piece) -> bool {
    match &p.template {
        Some(t) => t.params.iter().any(|q| is_open_param(q) && q.value > q.min + 1e-3),
        None => p.parts.iter().any(|q| q.hinge.as_ref().map(|h| h.open_deg.abs() > 0.5).unwrap_or(false)),
    }
}

/// Open everything that opens, or shut it all if anything is open.
pub fn toggle_open(p: &mut Piece) -> bool {
    if !has_moving_parts(p) {
        return false;
    }
    let shut = is_open(p);
    if let Some(t) = p.template.as_mut() {
        for q in t.params.iter_mut().filter(|q| is_open_param(q)) {
            q.value = if shut {
                q.min
            } else if q.kind == ParamKind::Swing {
                q.max.min(90.0)
            } else {
                q.max
            };
        }
        regenerate(p);
    } else {
        for q in p.parts.iter_mut() {
            if let Some(h) = q.hinge.as_mut() {
                h.open_deg = if shut { 0.0 } else { h.max_deg.min(90.0) };
            }
        }
    }
    true
}
