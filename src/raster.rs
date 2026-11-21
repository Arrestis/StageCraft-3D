//! A small multi-threaded software renderer: depth-buffered triangles, deferred
//! per-pixel stage lighting (spotlights with soft cones), additive light beams,
//! depth-tested lines, edge outlines and an object-id buffer for picking.

use crate::math::*;
use egui::Color32;
use rayon::prelude::*;

pub const F_DECK_GRID: u32 = 1;
pub const F_HOUSE: u32 = 2;
pub const F_LENS: u32 = 4;
pub const F_NO_EDGE: u32 = 8;

#[derive(Clone, Copy, Debug)]
pub struct Mat {
    pub albedo: [f32; 3],
    pub spec: f32,
    pub flags: u32,
    pub emissive: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Tri {
    pub p: [V3; 3],
    pub n: V3,
    pub mat: u32,
    pub obj: u32,
    pub cull: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Line3 {
    pub a: V3,
    pub b: V3,
    pub color: [u8; 4],
    /// Draw even when hidden behind geometry (faintly).
    pub xray: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct BeamTri {
    pub p: [V3; 3],
    pub w: [f32; 3],
    pub n: V3,
    pub color: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct ShadeLight {
    pub pos: V3,
    pub dir: V3,
    pub cos_outer: f32,
    pub cos_inner: f32,
    pub color: [f32; 3],
}

#[derive(Default)]
pub struct Geo {
    pub tris: Vec<Tri>,
    pub mats: Vec<Mat>,
    pub lines: Vec<Line3>,
    pub beams: Vec<BeamTri>,
    pub lights: Vec<ShadeLight>,
}

impl Geo {
    pub fn mat(&mut self, rgb: [u8; 3], spec: f32, flags: u32) -> u32 {
        self.mats.push(Mat {
            albedo: [srgb_to_lin(rgb[0]), srgb_to_lin(rgb[1]), srgb_to_lin(rgb[2])],
            spec,
            flags,
            emissive: 0.0,
        });
        (self.mats.len() - 1) as u32
    }
    pub fn mat_emissive(&mut self, rgb: [u8; 3], emissive: f32, flags: u32) -> u32 {
        let m = self.mat(rgb, 0.0, flags);
        self.mats[m as usize].emissive = emissive;
        m
    }
    pub fn tri(&mut self, a: V3, b: V3, c: V3, mat: u32, obj: u32) {
        let n = (b - a).cross(c - a).norm();
        self.tris.push(Tri { p: [a, b, c], n, mat, obj, cull: false });
    }
    pub fn quad(&mut self, a: V3, b: V3, c: V3, d: V3, mat: u32, obj: u32) {
        self.tri(a, b, c, mat, obj);
        self.tri(a, c, d, mat, obj);
    }
    /// One-sided quad, visible only from the side `n` points to.
    pub fn quad_one_sided(&mut self, a: V3, b: V3, c: V3, d: V3, n: V3, mat: u32, obj: u32) {
        self.tris.push(Tri { p: [a, b, c], n, mat, obj, cull: true });
        self.tris.push(Tri { p: [a, c, d], n, mat, obj, cull: true });
    }
    pub fn line(&mut self, a: V3, b: V3, color: [u8; 4]) {
        self.lines.push(Line3 { a, b, color, xray: false });
    }
}

pub fn srgb_to_lin(c: u8) -> f32 {
    let c = c as f32 / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn lin_to_srgb_u8(c: f32) -> u8 {
    let c = c.clamp(0.0, 1.0);
    let s = if c <= 0.0031308 { c * 12.92 } else { 1.055 * c.powf(1.0 / 2.4) - 0.055 };
    (s * 255.0 + 0.5) as u8
}

#[derive(Clone, Copy, Debug)]
pub struct Cam {
    pub eye: V3,
    pub target: V3,
    pub fov: f32,
    pub ortho: bool,
    pub ortho_h: f32,
}

pub struct CamMats {
    pub vp: M4,
    pub inv_vp: M4,
    pub near: f32,
    pub far: f32,
    pub ortho: bool,
    pub eye: V3,
    pub forward: V3,
}

impl Cam {
    pub fn up(&self) -> V3 {
        let f = (self.target - self.eye).norm();
        if f.y.abs() > 0.995 {
            v3(0.0, 0.0, -1.0)
        } else {
            v3(0.0, 1.0, 0.0)
        }
    }
    pub fn mats(&self, aspect: f32) -> CamMats {
        let view = M4::look_at(self.eye, self.target, self.up());
        let (near, far) = if self.ortho { (0.1, 400.0) } else { (0.08, 400.0) };
        let proj = if self.ortho {
            M4::ortho(self.ortho_h, aspect, near, far)
        } else {
            M4::perspective(self.fov, aspect, near, far)
        };
        let vp = proj.mul(&view);
        CamMats {
            inv_vp: vp.inverse(),
            vp,
            near,
            far,
            ortho: self.ortho,
            eye: self.eye,
            forward: (self.target - self.eye).norm(),
        }
    }
}

impl CamMats {
    pub fn unproject(&self, ndc_x: f32, ndc_y: f32, ndc_z: f32) -> V3 {
        let r = self.inv_vp.transform4(v3(ndc_x, ndc_y, ndc_z));
        let w = if r[3].abs() < 1e-12 { 1e-12 } else { r[3] };
        v3(r[0] / w, r[1] / w, r[2] / w)
    }
    /// Ray through a pixel (in render-buffer pixels).
    pub fn ray(&self, px: f32, py: f32, w: f32, h: f32) -> (V3, V3) {
        let nx = px / w * 2.0 - 1.0;
        let ny = 1.0 - py / h * 2.0;
        let a = self.unproject(nx, ny, -1.0);
        let b = self.unproject(nx, ny, 1.0);
        (a, (b - a).norm())
    }
    /// Project a world point to buffer pixels; None if behind the camera.
    pub fn project(&self, p: V3, w: f32, h: f32) -> Option<(f32, f32, f32)> {
        let c = self.vp.transform4(p);
        if c[3] <= 1e-4 {
            return None;
        }
        let x = c[0] / c[3];
        let y = c[1] / c[3];
        let z = c[2] / c[3];
        Some(((x + 1.0) * 0.5 * w, (1.0 - y) * 0.5 * h, z))
    }
    pub fn lin_depth(&self, z_ndc: f32) -> f32 {
        if self.ortho {
            (z_ndc + 1.0) * 0.5 * (self.far - self.near) + self.near
        } else {
            2.0 * self.far * self.near / ((self.far + self.near) - z_ndc * (self.far - self.near))
        }
    }
}

pub struct Settings {
    pub work_mode: bool,
    pub grid: bool,
    pub grid_step: f32,
    pub selected: u32,
    pub flagged: Vec<u32>,
    pub house_lights: f32,
    pub show_beams: bool,
    pub outlines: bool,
}

pub struct Frame {
    pub w: usize,
    pub h: usize,
    pub depth: Vec<f32>,
    pub tri: Vec<u32>,
    pub color: Vec<Color32>,
    accum: Vec<[f32; 3]>,
    pub tri_obj: Vec<u32>,
}

impl Frame {
    pub fn new() -> Self {
        Frame { w: 0, h: 0, depth: vec![], tri: vec![], color: vec![], accum: vec![], tri_obj: vec![] }
    }
    pub fn pick(&self, x: usize, y: usize) -> u32 {
        if x >= self.w || y >= self.h {
            return 0;
        }
        let t = self.tri[y * self.w + x];
        if t == u32::MAX {
            0
        } else {
            self.tri_obj.get(t as usize).copied().unwrap_or(0)
        }
    }
}

#[derive(Clone, Copy)]
struct SVert {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy)]
struct STri {
    v: [SVert; 3],
    id: u32,
    w: [f32; 3],
    min_y: i32,
    max_y: i32,
}

const BAND: usize = 8;

fn clip_poly(poly: &[([f32; 4], f32)]) -> Vec<([f32; 4], f32)> {
    // Clip against the near plane z + w >= 0 (with a tiny margin).
    let mut out = Vec::with_capacity(5);
    let n = poly.len();
    for i in 0..n {
        let (a, aw) = poly[i];
        let (b, bw) = poly[(i + 1) % n];
        let da = a[2] + a[3] - 1e-5;
        let db = b[2] + b[3] - 1e-5;
        if da >= 0.0 {
            out.push((a, aw));
        }
        if (da >= 0.0) != (db >= 0.0) {
            let t = da / (da - db);
            let mut c = [0.0; 4];
            for k in 0..4 {
                c[k] = a[k] + (b[k] - a[k]) * t;
            }
            out.push((c, aw + (bw - aw) * t));
        }
    }
    out
}

fn setup(
    p: [V3; 3],
    wts: [f32; 3],
    id: u32,
    cm: &CamMats,
    w: f32,
    h: f32,
    out: &mut Vec<STri>,
) {
    let c: Vec<([f32; 4], f32)> = (0..3).map(|i| (cm.vp.transform4(p[i]), wts[i])).collect();
    let all_in = c.iter().all(|(v, _)| v[2] + v[3] >= 1e-5);
    let poly = if all_in { c } else { clip_poly(&c) };
    if poly.len() < 3 {
        return;
    }
    let sv: Vec<(SVert, f32)> = poly
        .iter()
        .map(|(v, wt)| {
            let iw = 1.0 / v[3].max(1e-6);
            (
                SVert { x: (v[0] * iw + 1.0) * 0.5 * w, y: (1.0 - v[1] * iw) * 0.5 * h, z: v[2] * iw },
                *wt,
            )
        })
        .collect();
    for i in 1..sv.len() - 1 {
        let tri = [sv[0], sv[i], sv[i + 1]];
        let min_x = tri.iter().map(|t| t.0.x).fold(f32::MAX, f32::min);
        let max_x = tri.iter().map(|t| t.0.x).fold(f32::MIN, f32::max);
        let min_y = tri.iter().map(|t| t.0.y).fold(f32::MAX, f32::min);
        let max_y = tri.iter().map(|t| t.0.y).fold(f32::MIN, f32::max);
        if max_x < 0.0 || min_x > w || max_y < 0.0 || min_y > h {
            continue;
        }
        out.push(STri {
            v: [tri[0].0, tri[1].0, tri[2].0],
            w: [tri[0].1, tri[1].1, tri[2].1],
            id,
            min_y: min_y.floor().max(0.0) as i32,
            max_y: max_y.ceil().min(h - 1.0) as i32,
        });
    }
}

/// Visit every pixel centre covered by `t` inside rows [y0, y1).
#[inline]
fn raster_tri<F: FnMut(usize, usize, f32, f32, f32)>(t: &STri, wi: usize, y0: usize, y1: usize, mut f: F) {
    let [a, b, c] = t.v;
    let area = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
    if area.abs() < 1e-9 {
        return;
    }
    let inv = 1.0 / area;
    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as usize;
    let max_x = (a.x.max(b.x).max(c.x).ceil() as i64).min(wi as i64 - 1);
    if max_x < 0 {
        return;
    }
    let max_x = max_x as usize;
    let ys = (t.min_y.max(y0 as i32)) as usize;
    let ye = (t.max_y.min(y1 as i32 - 1)).max(-1);
    if ye < ys as i32 {
        return;
    }
    for y in ys..=(ye as usize) {
        let py = y as f32 + 0.5;
        // Find the x span on this row to avoid testing the whole box.
        let mut xs = max_x as f32 + 1.0;
        let mut xe = min_x as f32 - 1.0;
        for (p, q) in [(a, b), (b, c), (c, a)] {
            if (p.y <= py && q.y > py) || (q.y <= py && p.y > py) {
                let x = p.x + (py - p.y) / (q.y - p.y) * (q.x - p.x);
                xs = xs.min(x);
                xe = xe.max(x);
            }
        }
        if xe < xs {
            continue;
        }
        let sx = ((xs - 0.5).ceil().max(min_x as f32)) as usize;
        let ex_f = (xe - 0.5).floor().min(max_x as f32);
        if ex_f < sx as f32 {
            continue;
        }
        let ex = ex_f as usize;
        for x in sx..=ex {
            let px = x as f32 + 0.5;
            let w0 = ((b.x - px) * (c.y - py) - (b.y - py) * (c.x - px)) * inv;
            let w1 = ((c.x - px) * (a.y - py) - (c.y - py) * (a.x - px)) * inv;
            let w2 = 1.0 - w0 - w1;
            if w0 < -1e-4 || w1 < -1e-4 || w2 < -1e-4 {
                continue;
            }
            f(x, y, w0, w1, w2);
        }
    }
}

fn bin(stris: &[STri], bands: usize) -> Vec<Vec<u32>> {
    let mut bins: Vec<Vec<u32>> = vec![Vec::new(); bands];
    for (i, t) in stris.iter().enumerate() {
        let b0 = (t.min_y.max(0) as usize) / BAND;
        let b1 = ((t.max_y.max(0) as usize) / BAND).min(bands - 1);
        for b in bins.iter_mut().take(b1 + 1).skip(b0) {
            b.push(i as u32);
        }
    }
    bins
}

pub fn render(geo: &Geo, cam: &Cam, st: &Settings, w: usize, h: usize, fb: &mut Frame) -> CamMats {
    let w = w.max(16);
    let h = h.max(16);
    let n = w * h;
    if fb.w != w || fb.h != h {
        fb.w = w;
        fb.h = h;
        fb.depth = vec![0.0; n];
        fb.tri = vec![0; n];
        fb.color = vec![Color32::BLACK; n];
        fb.accum = vec![[0.0; 3]; n];
    }
    let cm = cam.mats(w as f32 / h as f32);
    let (wf, hf) = (w as f32, h as f32);

    // ---- Geometry setup ----
    let mut stris = Vec::with_capacity(geo.tris.len());
    for (i, t) in geo.tris.iter().enumerate() {
        if t.cull {
            let to_eye = if cm.ortho { -cm.forward } else { cm.eye - t.p[0] };
            if t.n.dot(to_eye) <= 0.0 {
                continue;
            }
        }
        setup(t.p, [0.0; 3], i as u32, &cm, wf, hf, &mut stris);
    }
    fb.tri_obj = geo.tris.iter().map(|t| t.obj).collect();
    let bands = (h + BAND - 1) / BAND;
    let bins = bin(&stris, bands);

    // ---- Depth + id pass ----
    fb.depth
        .par_chunks_mut(w * BAND)
        .zip(fb.tri.par_chunks_mut(w * BAND))
        .enumerate()
        .for_each(|(bi, (dch, tch))| {
            dch.fill(f32::MAX);
            tch.fill(u32::MAX);
            let y0 = bi * BAND;
            let y1 = (y0 + BAND).min(h);
            for &ti in &bins[bi] {
                let t = &stris[ti as usize];
                let (za, zb, zc) = (t.v[0].z, t.v[1].z, t.v[2].z);
                raster_tri(t, w, y0, y1, |x, y, b0, b1, b2| {
                    let z = za * b0 + zb * b1 + zc * b2;
                    let idx = (y - y0) * w + x;
                    if z < dch[idx] {
                        dch[idx] = z;
                        tch[idx] = t.id;
                    }
                });
            }
        });

    // ---- Beam pass (additive, depth tested) ----
    let have_beams = st.show_beams && !geo.beams.is_empty();
    if have_beams {
        let mut btris = Vec::with_capacity(geo.beams.len());
        for (i, b) in geo.beams.iter().enumerate() {
            setup(b.p, b.w, i as u32, &cm, wf, hf, &mut btris);
        }
        let bbins = bin(&btris, bands);
        let depth = &fb.depth;
        let strength = if st.work_mode { 0.05 } else { 0.11 };
        fb.accum.par_chunks_mut(w * BAND).enumerate().for_each(|(bi, ach)| {
            for a in ach.iter_mut() {
                *a = [0.0; 3];
            }
            let y0 = bi * BAND;
            let y1 = (y0 + BAND).min(h);
            for &ti in &bbins[bi] {
                let t = &btris[ti as usize];
                let bt = &geo.beams[t.id as usize];
                let centre = bt.p[0] * (1.0 / 3.0) + bt.p[1] * (1.0 / 3.0) + bt.p[2] * (1.0 / 3.0);
                let vdir = if cm.ortho { -cm.forward } else { (cm.eye - centre).norm() };
                let facing = bt.n.dot(vdir).abs().powf(1.5) * strength;
                let (za, zb, zc) = (t.v[0].z, t.v[1].z, t.v[2].z);
                raster_tri(t, w, y0, y1, |x, y, b0, b1, b2| {
                    let z = za * b0 + zb * b1 + zc * b2;
                    let gi = y * w + x;
                    if z < depth[gi] {
                        let wt = (t.w[0] * b0 + t.w[1] * b1 + t.w[2] * b2) * facing;
                        let a = &mut ach[(y - y0) * w + x];
                        a[0] += bt.color[0] * wt;
                        a[1] += bt.color[1] * wt;
                        a[2] += bt.color[2] * wt;
                    }
                });
            }
        });
    }

    // ---- Shading pass ----
    let depth = &fb.depth;
    let tri = &fb.tri;
    let accum = &fb.accum;
    let tris = &geo.tris;
    let mats = &geo.mats;
    let lights = &geo.lights;
    let key = v3(0.35, 0.9, 0.45).norm();
    let sel = st.selected;
    let flagged = &st.flagged;
    let is_flag = |o: u32| o != 0 && flagged.binary_search(&o).is_ok();
    let obj_at = |i: usize| -> u32 {
        let t = tri[i];
        if t == u32::MAX {
            0
        } else {
            tris[t as usize].obj
        }
    };
    let world_at = |x: usize, y: usize| -> V3 {
        let i = y * w + x;
        let nx = (x as f32 + 0.5) / wf * 2.0 - 1.0;
        let ny = 1.0 - (y as f32 + 0.5) / hf * 2.0;
        cm.unproject(nx, ny, depth[i])
    };
    let ambient = 0.02 + st.house_lights * 0.25;
    fb.color.par_chunks_mut(w).enumerate().for_each(|(y, row)| {
        for x in 0..w {
            let i = y * w + x;
            let ti = tri[i];
            let mut col;
            if ti == u32::MAX {
                let t = y as f32 / hf;
                col = if st.work_mode {
                    [0.035 + 0.03 * (1.0 - t), 0.04 + 0.035 * (1.0 - t), 0.055 + 0.05 * (1.0 - t)]
                } else {
                    [0.004, 0.004, 0.008]
                };
            } else {
                let t = &tris[ti as usize];
                let m = &mats[t.mat as usize];
                let p = world_at(x, y);
                let vdir = if cm.ortho { -cm.forward } else { (cm.eye - p).norm() };
                let mut nrm = t.n;
                if nrm.dot(vdir) < 0.0 {
                    nrm = -nrm;
                }
                let mut alb = m.albedo;
                if st.grid && (m.flags & F_DECK_GRID) != 0 && nrm.y > 0.5 {
                    let step = st.grid_step;
                    let lw = 0.012 + 0.0015 * (p - cm.eye).len().min(40.0);
                    let fx = (p.x / step).round() * step - p.x;
                    let fz = (p.z / step).round() * step - p.z;
                    let mut line = 0.0f32;
                    if fx.abs() < lw || fz.abs() < lw {
                        line = 0.35;
                    }
                    let major = step * 5.0;
                    let mx = (p.x / major).round() * major - p.x;
                    let mz = (p.z / major).round() * major - p.z;
                    if mx.abs() < lw * 1.4 || mz.abs() < lw * 1.4 {
                        line = 0.55;
                    }
                    let mut tint = [1.0, 1.0, 1.0];
                    if p.x.abs() < lw * 1.6 && ((p.z * 2.0).rem_euclid(1.0) < 0.6) {
                        line = 0.9;
                        tint = [1.0, 0.75, 0.25];
                    }
                    if p.z.abs() < lw * 1.6 {
                        line = 0.9;
                        tint = [1.0, 0.75, 0.25];
                    }
                    if line > 0.0 {
                        for k in 0..3 {
                            alb[k] = alb[k] * (1.0 - line) + 0.5 * tint[k] * line;
                        }
                    }
                }
                if st.work_mode {
                    let diff = 0.42 + 0.14 * nrm.y + 0.5 * nrm.dot(key).max(0.0) + 0.1 * nrm.dot(vdir).max(0.0);
                    let hv = (key + vdir).norm();
                    let sp = m.spec * nrm.dot(hv).max(0.0).powf(28.0) * 0.6;
                    col = [alb[0] * diff + sp, alb[1] * diff * 0.99 + sp, alb[2] * diff * 0.97 + sp];
                    if m.emissive > 0.0 {
                        for k in 0..3 {
                            col[k] += alb[k] * m.emissive * 0.4;
                        }
                    }
                } else {
                    let amb = if (m.flags & F_HOUSE) != 0 { ambient * 2.5 } else { ambient };
                    col = [alb[0] * amb, alb[1] * amb, alb[2] * amb];
                    for l in lights.iter() {
                        let lv = l.pos - p;
                        let d2 = lv.dot(lv);
                        let d = d2.sqrt();
                        let ld = lv / d;
                        let ndl = nrm.dot(ld);
                        if ndl <= 0.0 {
                            continue;
                        }
                        let ca = -ld.dot(l.dir);
                        if ca <= l.cos_outer {
                            continue;
                        }
                        let spot = smoothstep(l.cos_outer, l.cos_inner, ca);
                        let att = 26.0 / (6.0 + d2);
                        let k = ndl * spot * att;
                        let hv = (ld + vdir).norm();
                        let sp = m.spec * nrm.dot(hv).max(0.0).powf(32.0) * spot * att;
                        col[0] += (alb[0] * k + sp) * l.color[0];
                        col[1] += (alb[1] * k + sp) * l.color[1];
                        col[2] += (alb[2] * k + sp) * l.color[2];
                    }
                    if m.emissive > 0.0 {
                        for k in 0..3 {
                            col[k] += alb[k] * m.emissive;
                        }
                    }
                }
                // Edge outlines.
                if st.outlines && (m.flags & F_NO_EDGE) == 0 {
                    let mut edge = false;
                    let o = t.obj;
                    for (nx, ny) in [(x + 1, y), (x, y + 1)] {
                        if nx >= w || ny >= h {
                            continue;
                        }
                        let j = ny * w + nx;
                        let tj = tri[j];
                        if tj == u32::MAX {
                            edge = true;
                            break;
                        }
                        let t2 = &tris[tj as usize];
                        if t2.obj != o {
                            edge = true;
                            break;
                        }
                        let mut n2 = t2.n;
                        if n2.dot(vdir) < 0.0 {
                            n2 = -n2;
                        }
                        if n2.dot(nrm) < 0.9 {
                            edge = true;
                            break;
                        }
                        let p2 = world_at(nx, ny);
                        let dist = (p - cm.eye).len();
                        if (p2 - p).dot(nrm).abs() > 0.004 * dist + 0.01 {
                            edge = true;
                            break;
                        }
                    }
                    if edge {
                        let f = if st.work_mode { 0.45 } else { 0.7 };
                        for c in col.iter_mut() {
                            *c *= f;
                        }
                    }
                }
            }
            if have_beams {
                let a = accum[i];
                col[0] += a[0];
                col[1] += a[1];
                col[2] += a[2];
            }
            if !st.work_mode {
                for c in col.iter_mut() {
                    *c = 1.0 - (-*c * 1.1).exp();
                }
            }
            // Selection & warning highlights.
            let o = obj_at(i);
            if sel != 0 || !flagged.is_empty() {
                let me_sel = sel != 0 && o == sel;
                let me_flag = is_flag(o);
                let mut sel_edge = false;
                let mut flag_edge = false;
                for (dx, dy) in [(1i32, 0i32), (-1, 0), (0, 1), (0, -1), (2, 0), (0, 2), (-2, 0), (0, -2)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        continue;
                    }
                    let on = obj_at(ny as usize * w + nx as usize);
                    if sel != 0 && ((on == sel) != me_sel) {
                        sel_edge = true;
                    }
                    if (is_flag(on)) != me_flag && (me_flag || is_flag(on)) {
                        flag_edge = true;
                    }
                }
                if me_sel {
                    col[0] = col[0] * 0.85 + 0.12;
                    col[1] = col[1] * 0.85 + 0.06;
                    col[2] *= 0.85;
                }
                if flag_edge {
                    col = [0.95, 0.08, 0.06];
                }
                if sel_edge {
                    col = [1.0, 0.55, 0.05];
                }
            }
            row[x] = Color32::from_rgb(lin_to_srgb_u8(col[0]), lin_to_srgb_u8(col[1]), lin_to_srgb_u8(col[2]));
        }
    });

    // ---- Lines (depth tested) ----
    for l in &geo.lines {
        draw_line(fb, &cm, l);
    }
    cm
}

fn draw_line(fb: &mut Frame, cm: &CamMats, l: &Line3) {
    let (w, h) = (fb.w as f32, fb.h as f32);
    let mut a = cm.vp.transform4(l.a);
    let mut b = cm.vp.transform4(l.b);
    let da = a[2] + a[3];
    let db = b[2] + b[3];
    if da < 1e-4 && db < 1e-4 {
        return;
    }
    if da < 1e-4 || db < 1e-4 {
        let t = (1e-4 - da) / (db - da);
        let c: [f32; 4] = std::array::from_fn(|k| a[k] + (b[k] - a[k]) * t);
        if da < 1e-4 {
            a = c;
        } else {
            b = c;
        }
    }
    let to_s = |c: [f32; 4]| -> (f32, f32, f32) {
        let iw = 1.0 / c[3];
        ((c[0] * iw + 1.0) * 0.5 * w, (1.0 - c[1] * iw) * 0.5 * h, c[2] * iw)
    };
    let (x0, y0, z0) = to_s(a);
    let (x1, y1, z1) = to_s(b);
    let steps = ((x1 - x0).abs().max((y1 - y0).abs()).ceil() as usize).clamp(1, 6000);
    let alpha = l.color[3] as f32 / 255.0;
    let lc = [l.color[0] as f32, l.color[1] as f32, l.color[2] as f32];
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let x = x0 + (x1 - x0) * t;
        let y = y0 + (y1 - y0) * t;
        if x < 0.0 || y < 0.0 || x >= w || y >= h {
            continue;
        }
        let z = z0 + (z1 - z0) * t;
        let i = y as usize * fb.w + x as usize;
        let ld = cm.lin_depth(z);
        let sd = cm.lin_depth(fb.depth[i].min(1.0));
        let hidden = fb.depth[i] < f32::MAX && ld > sd + 0.03 + ld * 0.006;
        let a = if hidden {
            if l.xray {
                alpha * 0.25
            } else {
                continue;
            }
        } else {
            alpha
        };
        let c = fb.color[i];
        let mix = |o: u8, n: f32| -> u8 { (o as f32 * (1.0 - a) + n * a) as u8 };
        fb.color[i] = Color32::from_rgb(mix(c.r(), lc[0]), mix(c.g(), lc[1]), mix(c.b(), lc[2]));
    }
}
