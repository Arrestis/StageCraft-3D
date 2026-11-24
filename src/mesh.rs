// Copyright (C) 2026 Doug Rumble
// SPDX-License-Identifier: GPL-3.0-only

//! Primitive shapes in unit space (x, z in -0.5..0.5, y in 0..1), placed by a transform.

use crate::math::*;
use crate::model::Shape;
use crate::raster::Geo;
use std::f32::consts::PI;

pub fn add_shape(g: &mut Geo, shape: Shape, xf: &dyn Fn(V3) -> V3, mat: u32, obj: u32) {
    match shape {
        Shape::Box => unit_box(g, xf, v3(-0.5, 0.0, -0.5), v3(0.5, 1.0, 0.5), mat, obj),
        Shape::Cylinder => cylinder(g, xf, 18, 1.0, mat, obj),
        Shape::Cone => cylinder(g, xf, 18, 0.0, mat, obj),
        Shape::Sphere => sphere(g, xf, mat, obj),
        Shape::Ramp => ramp(g, xf, mat, obj),
        Shape::Stairs(n) => {
            let n = n.max(1) as f32;
            for i in 0..n as usize {
                let i = i as f32;
                unit_box(
                    g,
                    xf,
                    v3(-0.5, 0.0, 0.5 - (i + 1.0) / n),
                    v3(0.5, (i + 1.0) / n, 0.5 - i / n),
                    mat,
                    obj,
                );
            }
        }
        Shape::ArchTop => arch_top(g, xf, mat, obj),
    }
}

pub fn unit_box(g: &mut Geo, xf: &dyn Fn(V3) -> V3, lo: V3, hi: V3, mat: u32, obj: u32) {
    let c = |x: f32, y: f32, z: f32| xf(v3(x, y, z));
    let (a, b) = (lo, hi);
    let p = [
        c(a.x, a.y, a.z),
        c(b.x, a.y, a.z),
        c(b.x, a.y, b.z),
        c(a.x, a.y, b.z),
        c(a.x, b.y, a.z),
        c(b.x, b.y, a.z),
        c(b.x, b.y, b.z),
        c(a.x, b.y, b.z),
    ];
    let faces = [[0, 1, 2, 3], [4, 7, 6, 5], [0, 4, 5, 1], [3, 2, 6, 7], [0, 3, 7, 4], [1, 5, 6, 2]];
    for f in faces {
        g.quad(p[f[0]], p[f[1]], p[f[2]], p[f[3]], mat, obj);
    }
}

/// World-space axis-aligned box.
pub fn world_box(g: &mut Geo, lo: V3, hi: V3, mat: u32, obj: u32) {
    unit_box(g, &|p| p, lo, hi, mat, obj);
}

fn cylinder(g: &mut Geo, xf: &dyn Fn(V3) -> V3, n: usize, top_scale: f32, mat: u32, obj: u32) {
    let ring = |y: f32, s: f32| -> Vec<V3> {
        (0..n)
            .map(|i| {
                let t = i as f32 / n as f32 * 2.0 * PI;
                xf(v3(0.5 * s * t.cos(), y, 0.5 * s * t.sin()))
            })
            .collect()
    };
    let bot = ring(0.0, 1.0);
    let top = ring(1.0, top_scale);
    let cb = xf(v3(0.0, 0.0, 0.0));
    let ct = xf(v3(0.0, 1.0, 0.0));
    for i in 0..n {
        let j = (i + 1) % n;
        if top_scale > 0.0 {
            g.quad(bot[i], bot[j], top[j], top[i], mat, obj);
            g.tri(ct, top[i], top[j], mat, obj);
        } else {
            g.tri(bot[i], bot[j], ct, mat, obj);
        }
        g.tri(cb, bot[j], bot[i], mat, obj);
    }
}

fn sphere(g: &mut Geo, xf: &dyn Fn(V3) -> V3, mat: u32, obj: u32) {
    let (la, lo) = (7usize, 12usize);
    let pt = |i: usize, j: usize| -> V3 {
        let th = i as f32 / la as f32 * PI;
        let ph = j as f32 / lo as f32 * 2.0 * PI;
        // Slight deterministic wobble makes rocks and foliage less perfect.
        let wob = 1.0 + 0.06 * ((i * 7 + j * 13) % 5) as f32 / 4.0 - 0.03;
        let wob = if i == 0 || i == la { 1.0 } else { wob };
        xf(v3(0.5 * th.sin() * ph.cos() * wob, 0.5 + 0.5 * th.cos() * wob, 0.5 * th.sin() * ph.sin() * wob))
    };
    for i in 0..la {
        for j in 0..lo {
            let a = pt(i, j);
            let b = pt(i + 1, j);
            let c = pt(i + 1, (j + 1) % lo);
            let d = pt(i, (j + 1) % lo);
            if i == 0 {
                g.tri(a, b, c, mat, obj);
            } else if i == la - 1 {
                g.tri(a, b, d, mat, obj);
            } else {
                g.quad(a, b, c, d, mat, obj);
            }
        }
    }
}

fn ramp(g: &mut Geo, xf: &dyn Fn(V3) -> V3, mat: u32, obj: u32) {
    let c = |x: f32, y: f32, z: f32| xf(v3(x, y, z));
    let a = c(-0.5, 0.0, 0.5);
    let b = c(0.5, 0.0, 0.5);
    let bb = c(0.5, 0.0, -0.5);
    let ab = c(-0.5, 0.0, -0.5);
    let at = c(-0.5, 1.0, -0.5);
    let bt = c(0.5, 1.0, -0.5);
    g.quad(a, b, bb, ab, mat, obj);
    g.quad(a, b, bt, at, mat, obj);
    g.quad(ab, bb, bt, at, mat, obj);
    g.tri(a, ab, at, mat, obj);
    g.tri(b, bb, bt, mat, obj);
}

fn arch_top(g: &mut Geo, xf: &dyn Fn(V3) -> V3, mat: u32, obj: u32) {
    let n = 12;
    let e = |t: f32| (0.5 * t.cos(), t.sin());
    let r = |t: f32| {
        let (c, s) = (t.cos(), t.sin());
        let k = 1.0 / c.abs().max(s).max(1e-4);
        (0.5 * c * k, s * k)
    };
    for i in 0..n {
        let t0 = i as f32 / n as f32 * PI;
        let t1 = (i + 1) as f32 / n as f32 * PI;
        let (e0, e1, r0, r1) = (e(t0), e(t1), r(t0), r(t1));
        for z in [0.5f32, -0.5] {
            g.quad(
                xf(v3(e0.0, e0.1, z)),
                xf(v3(e1.0, e1.1, z)),
                xf(v3(r1.0, r1.1, z)),
                xf(v3(r0.0, r0.1, z)),
                mat,
                obj,
            );
        }
        g.quad(
            xf(v3(e0.0, e0.1, 0.5)),
            xf(v3(e1.0, e1.1, 0.5)),
            xf(v3(e1.0, e1.1, -0.5)),
            xf(v3(e0.0, e0.1, -0.5)),
            mat,
            obj,
        );
    }
    let c = |x: f32, y: f32, z: f32| xf(v3(x, y, z));
    g.quad(c(-0.5, 1.0, 0.5), c(0.5, 1.0, 0.5), c(0.5, 1.0, -0.5), c(-0.5, 1.0, -0.5), mat, obj);
    g.quad(c(0.5, 0.0, 0.5), c(0.5, 1.0, 0.5), c(0.5, 1.0, -0.5), c(0.5, 0.0, -0.5), mat, obj);
    g.quad(c(-0.5, 0.0, 0.5), c(-0.5, 1.0, 0.5), c(-0.5, 1.0, -0.5), c(-0.5, 0.0, -0.5), mat, obj);
}

/// A pleated drape between x0 and x1 (world), hanging at z.
pub fn drape(g: &mut Geo, x0: f32, x1: f32, y0: f32, y1: f32, z: f32, pleat: f32, depth: f32, mat: u32, obj: u32) {
    if x1 - x0 < 0.01 || y1 - y0 < 0.01 {
        return;
    }
    let n = (((x1 - x0) / pleat).ceil() as usize).max(2) * 2;
    let pts: Vec<(f32, f32)> = (0..=n)
        .map(|i| {
            let x = x0 + (x1 - x0) * i as f32 / n as f32;
            let dz = if i % 2 == 0 { depth } else { -depth };
            (x, z + dz)
        })
        .collect();
    for i in 0..n {
        let (xa, za) = pts[i];
        let (xb, zb) = pts[i + 1];
        g.quad(v3(xa, y0, za), v3(xb, y0, zb), v3(xb, y1, zb), v3(xa, y1, za), mat, obj);
    }
}

/// A cylinder from a to b with radius r (for lights and pipes).
pub fn tube(g: &mut Geo, a: V3, b: V3, r: f32, n: usize, mat: u32, obj: u32, cap_mat: Option<u32>) {
    let ax = (b - a).norm();
    let tmp = if ax.y.abs() < 0.9 { v3(0.0, 1.0, 0.0) } else { v3(1.0, 0.0, 0.0) };
    let u = ax.cross(tmp).norm();
    let w = ax.cross(u).norm();
    let ring = |c: V3| -> Vec<V3> {
        (0..n)
            .map(|i| {
                let t = i as f32 / n as f32 * 2.0 * PI;
                c + u * (r * t.cos()) + w * (r * t.sin())
            })
            .collect()
    };
    let ra = ring(a);
    let rb = ring(b);
    for i in 0..n {
        let j = (i + 1) % n;
        g.quad(ra[i], ra[j], rb[j], rb[i], mat, obj);
        g.tri(a, ra[j], ra[i], mat, obj);
        g.tri(b, rb[i], rb[j], cap_mat.unwrap_or(mat), obj);
    }
}
