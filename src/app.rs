//! The Stagecraft 3D application: state, camera, 3D viewport and interaction.

use crate::checks::{self, Warning};
use crate::library;
use crate::math::*;
use crate::model::*;
use crate::raster::{self, Cam, CamMats, Frame};
use crate::scene::{self, *};
use crate::venues;
use egui::{Color32, Pos2, Rect, Sense, Vec2};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CamPreset {
    Director,
    Overview,
    TopDown,
    WingSL,
    WingSR,
    Free,
}

impl CamPreset {
    pub fn label(self) -> &'static str {
        match self {
            CamPreset::Director => "🎬 Director / Audience",
            CamPreset::Overview => "🧊 3/4 Overview",
            CamPreset::TopDown => "🗺 Ground Plan",
            CamPreset::WingSL => "◀ SL Wing",
            CamPreset::WingSR => "▶ SR Wing",
            CamPreset::Free => "✋ Free Orbit",
        }
    }
    pub const ALL: [CamPreset; 6] = [
        CamPreset::Director,
        CamPreset::Overview,
        CamPreset::TopDown,
        CamPreset::WingSL,
        CamPreset::WingSR,
        CamPreset::Free,
    ];
}

#[derive(Clone, Copy, Debug)]
pub struct Orbit {
    pub target: V3,
    pub yaw: f32,
    pub pitch: f32,
    pub dist: f32,
    pub fov: f32,
    pub ortho: bool,
    pub ortho_h: f32,
}

impl Orbit {
    pub fn eye(&self) -> V3 {
        let (sy, cy) = self.yaw.to_radians().sin_cos();
        let (sp, cp) = self.pitch.to_radians().sin_cos();
        self.target + v3(cp * sy, sp, cp * cy) * self.dist
    }
    pub fn cam(&self) -> Cam {
        Cam { eye: self.eye(), target: self.target, fov: self.fov, ortho: self.ortho, ortho_h: self.ortho_h }
    }
    pub fn from_eye(eye: V3, target: V3, fov: f32) -> Orbit {
        let d = eye - target;
        let dist = d.len().max(0.5);
        let pitch = (d.y / dist).clamp(-1.0, 1.0).asin().to_degrees();
        let yaw = d.x.atan2(d.z).to_degrees();
        Orbit { target, yaw, pitch, dist, fov, ortho: false, ortho_h: 10.0 }
    }
    fn lerp(&self, o: &Orbit, t: f32) -> Orbit {
        let mut dy = o.yaw - self.yaw;
        while dy > 180.0 {
            dy -= 360.0;
        }
        while dy < -180.0 {
            dy += 360.0;
        }
        Orbit {
            target: self.target.lerp(o.target, t),
            yaw: self.yaw + dy * t,
            pitch: self.pitch + (o.pitch - self.pitch) * t,
            dist: self.dist + (o.dist - self.dist) * t,
            fov: self.fov + (o.fov - self.fov) * t,
            ortho: o.ortho,
            ortho_h: self.ortho_h + (o.ortho_h - self.ortho_h) * t,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LeftTab {
    Venue,
    Library,
    Lighting,
    Scenes,
}

pub struct Builder {
    pub item: LibItem,
    pub editing_piece: Option<u32>,
    pub editing_lib: Option<usize>,
    pub preview_on_stage: bool,
}

enum Drag {
    Orbit,
    Pan,
    Piece { idx: usize, offset: V3, plane_y: f32, vertical: bool, start_elev: f32 },
    Batten { idx: usize },
    LightAim { idx: usize },
    Revolve,
    Pit,
    Lift,
}

struct Shot {
    path: String,
    frames: u32,
    requested: bool,
    /// Crop to the viewport (snapshot export) or keep the whole window.
    crop: Option<Rect>,
    ppp: f32,
}

pub struct App {
    pub project: Project,
    pub venues: Vec<Venue>,
    pub library: Vec<LibItem>,
    pub disp: Display,
    pub orbit: Orbit,
    pub orbit_goal: Option<Orbit>,
    pub cam_preset: CamPreset,
    fb: Frame,
    tex: Option<egui::TextureHandle>,
    last_cm: Option<CamMats>,
    last_view_rect: Rect,
    pub render_scale: f32,
    pub sel: u32,
    pub work_mode: bool,
    pub show_beams: bool,
    pub snap: bool,
    pub snap_step: f32,
    pub rot_step: f32,
    pub show_warnings: bool,
    pub show_labels: bool,
    pub show_grid: bool,
    pub show_sightlines: bool,
    pub show_rigging: bool,
    pub feet: bool,
    pub animate: bool,
    pub tab: LeftTab,
    pub builder: Option<Builder>,
    drag: Option<Drag>,
    pub aim_mode: bool,
    pub toast: Option<(String, f64)>,
    shot: Option<Shot>,
    shot_slot: Arc<Mutex<Option<Arc<egui::ColorImage>>>>,
    pub warnings: Vec<Warning>,
    pub lib_cat: usize,
    pub lib_filter: String,
    pub show_warn_list: bool,
    last_render_ms: f32,
    exit_after_shot: bool,
    /// The app icon, shown next to the name in the top bar.
    pub logo: Option<egui::TextureHandle>,
}

fn load_logo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let img = image::load_from_memory(include_bytes!("../assets/icon-256.png")).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    Some(ctx.load_texture("stagecraft-logo", color, egui::TextureOptions::LINEAR))
}

pub fn deg_wrap(a: f32) -> f32 {
    let mut a = a % 360.0;
    if a > 180.0 {
        a -= 360.0;
    }
    if a < -180.0 {
        a += 360.0;
    }
    a
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, screenshot: Option<String>) -> Self {
        crate::style::apply(&cc.egui_ctx);
        let venues = venues::presets();
        let venue = venues.iter().find(|v| v.key == "armidale_playhouse").cloned().unwrap_or_else(|| venues[0].clone());
        let project = crate::sample::sample_project(venue);
        let mut app = App {
            project,
            venues,
            library: library::presets(),
            disp: Display::default(),
            orbit: Orbit::from_eye(v3(10.0, 10.0, 18.0), v3(0.0, 1.0, -4.0), 50.0),
            orbit_goal: None,
            cam_preset: CamPreset::Overview,
            fb: Frame::new(),
            tex: None,
            last_cm: None,
            last_view_rect: Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0)),
            render_scale: 1.0,
            sel: 0,
            work_mode: true,
            show_beams: true,
            snap: true,
            snap_step: 0.25,
            rot_step: 15.0,
            show_warnings: true,
            show_labels: true,
            show_grid: true,
            show_sightlines: false,
            show_rigging: true,
            feet: false,
            animate: true,
            tab: LeftTab::Library,
            builder: None,
            drag: None,
            aim_mode: false,
            toast: None,
            shot: None,
            shot_slot: Arc::new(Mutex::new(None)),
            warnings: Vec::new(),
            lib_cat: 0,
            lib_filter: String::new(),
            show_warn_list: false,
            last_render_ms: 0.0,
            exit_after_shot: false,
            logo: load_logo(&cc.egui_ctx),
        };
        app.snap_display();
        app.apply_cam_preset(CamPreset::Overview, false, 1.6);
        // Optional start-up state for testing views, e.g. STAGECRAFT_DEMO="venue=black_box,cue,cam=director".
        if let Ok(demo) = std::env::var("STAGECRAFT_DEMO") {
            for tok in demo.split(',') {
                let (k, val) = tok.split_once('=').unwrap_or((tok, ""));
                match k {
                    "venue" => {
                        if let Some(v) = app.venues.iter().find(|v| v.key == val).cloned() {
                            app.project = crate::sample::sample_project(v);
                            app.snap_display();
                        }
                    }
                    "cue" => app.work_mode = false,
                    "nobeams" => app.show_beams = false,
                    "sightlines" => app.show_sightlines = true,
                    "feet" => app.feet = true,
                    "scene" => {
                        app.project.current = val.parse::<usize>().unwrap_or(0).min(app.project.scenes.len() - 1);
                        app.snap_display();
                    }
                    "sel" => {
                        let (kind, i) = val.split_once(':').unwrap_or((val, "0"));
                        let i: u32 = i.parse().unwrap_or(0);
                        app.sel = match kind {
                            "piece" => OBJ_PIECE + i,
                            "batten" => OBJ_BATTEN + i,
                            "light" => OBJ_LIGHT + i,
                            "revolve" => OBJ_REVOLVE,
                            "pit" => OBJ_PIT,
                            _ => 0,
                        };
                    }
                    "tab" => {
                        app.tab = match val {
                            "venue" => LeftTab::Venue,
                            "lights" => LeftTab::Lighting,
                            "scenes" => LeftTab::Scenes,
                            _ => LeftTab::Library,
                        }
                    }
                    "builder" => app.open_builder(None),
                    "cam" => {
                        let p = match val {
                            "director" => CamPreset::Director,
                            "plan" => CamPreset::TopDown,
                            "sl" => CamPreset::WingSL,
                            "sr" => CamPreset::WingSR,
                            _ => CamPreset::Overview,
                        };
                        app.apply_cam_preset(p, false, 1.05);
                    }
                    _ => {}
                }
            }
        }
        if let Some(path) = screenshot {
            app.shot = Some(Shot { path, frames: 0, requested: false, crop: None, ppp: 1.0 });
            app.exit_after_shot = true;
        }
        app
    }

    pub fn state(&self) -> &StageState {
        let i = self.project.current.min(self.project.scenes.len() - 1);
        &self.project.scenes[i].state
    }
    pub fn state_mut(&mut self) -> &mut StageState {
        let i = self.project.current.min(self.project.scenes.len() - 1);
        &mut self.project.scenes[i].state
    }

    pub fn view_rect(&self) -> Rect {
        self.last_view_rect
    }

    pub fn notify(&mut self, msg: impl Into<String>, now: f64) {
        self.toast = Some((msg.into(), now + 3.5));
    }

    // ---------------------------------------------------------------- display animation

    pub fn snap_display(&mut self) {
        let s = self.state().clone();
        self.disp.batten_h = s.battens.iter().map(|b| b.height).collect();
        self.disp.batten_open = s.battens.iter().map(|b| b.open).collect();
        self.disp.revolve = s.revolve_angle;
        self.disp.pit = s.pit_lift;
        self.disp.lift = s.stage_lift;
    }

    /// Glide shown values towards the scene's values. Returns true while moving.
    fn animate_display(&mut self, dt: f32) -> bool {
        let s = self.state().clone();
        let n = s.battens.len();
        if self.disp.batten_h.len() != n {
            self.disp.batten_h.resize(n, 0.0);
            self.disp.batten_open.resize(n, 1.0);
            for (i, b) in s.battens.iter().enumerate() {
                self.disp.batten_h[i] = b.height;
                self.disp.batten_open[i] = b.open;
            }
        }
        if !self.animate {
            self.snap_display();
            return false;
        }
        let mut moving = false;
        let step = |cur: &mut f32, goal: f32, speed: f32, moving: &mut bool| {
            let d = goal - *cur;
            if d.abs() > 1e-4 {
                let m = speed * dt;
                *cur += d.clamp(-m, m);
                *moving = true;
            } else {
                *cur = goal;
            }
        };
        for (i, b) in s.battens.iter().enumerate() {
            step(&mut self.disp.batten_h[i], b.height, 1.6, &mut moving);
            step(&mut self.disp.batten_open[i], b.open, 0.9, &mut moving);
        }
        // Revolve takes the short way round.
        let goal = s.revolve_angle;
        let mut cur = self.disp.revolve;
        let diff = goal - cur;
        if diff.abs() > 1e-3 {
            let m = 40.0 * dt;
            cur += diff.clamp(-m, m);
            moving = true;
        } else {
            cur = goal;
        }
        self.disp.revolve = cur;
        step(&mut self.disp.pit, s.pit_lift, 0.6, &mut moving);
        step(&mut self.disp.lift, s.stage_lift, 0.6, &mut moving);
        moving
    }

    // ---------------------------------------------------------------- camera

    pub fn apply_cam_preset(&mut self, p: CamPreset, animate: bool, aspect: f32) {
        let v = &self.project.venue;
        let depth = v.stage_depth;
        let hw = v.stage_width() * 0.5;
        let pw = v.pros_width * 0.5;
        let sh = v.stage_height;
        let goal = match p {
            CamPreset::Director => {
                let rows = v.rows() as f32;
                // Sit far enough back to see the whole opening (and a little of the room)
                // across the view, like a director at the production desk.
                let hfov = 2.0 * ((23.0f32).to_radians().tan() * aspect.max(0.5)).atan();
                let need = (v.pros_width + 2.5) / (2.0 * (hfov * 0.5).tan());
                let mut r = (rows * 0.4).floor().min(12.0);
                while r < rows - 1.0 && v.house_start_z() + v.row_offset(r as u32) + 0.6 < need {
                    r += 1.0;
                }
                let z = v.house_start_z() + v.row_offset(r as u32) + 0.6;
                let y = -sh + v.row_rise(r as u32) + 1.15;
                let tz = -depth * 0.38;
                Orbit::from_eye(v3(0.0, y, z), v3(0.0, 1.5 + v.deck_y(tz), tz), 46.0)
            }
            CamPreset::Overview => {
                let big = (v.stage_width().max(depth + v.apron_depth)).max(6.0);
                let big = if big < 11.0 { big * 0.8 } else { big };
                // Low rooms have lighting bars over the house: look from higher and further back.
                let big = if v.open_end_stage { big.max(13.0) } else { big };
                Orbit::from_eye(
                    v3(hw * 0.55 + big * 0.2, v.pros_height * 0.6 + big * 0.62, v.apron_depth + big * 0.72),
                    v3(0.0, 0.6, -depth * 0.45),
                    48.0,
                )
            }
            CamPreset::TopDown => {
                let cz = (v.apron_depth + 1.0 - depth) * 0.5;
                let span_z = depth + v.apron_depth + 2.5;
                let span_x = v.stage_width().max(v.house_width * 0.6) + 1.0;
                let half = (span_z * 0.5).max(span_x * 0.5 / aspect.max(0.3)) * 1.08;
                let mut o = Orbit::from_eye(v3(0.0, 80.0, cz + 0.001), v3(0.0, 0.0, cz), 40.0);
                o.yaw = 0.0;
                o.pitch = 89.9;
                o.ortho = true;
                o.ortho_h = half;
                o
            }
            CamPreset::WingSL => Orbit::from_eye(
                v3(hw - 0.25, 2.3 + v.deck_y(-depth * 0.3), -depth * 0.3),
                v3(-pw * 0.5, 0.9, -depth * 0.5),
                64.0,
            ),
            CamPreset::WingSR => Orbit::from_eye(
                v3(-hw + 0.25, 2.3 + v.deck_y(-depth * 0.3), -depth * 0.3),
                v3(pw * 0.5, 0.9, -depth * 0.5),
                64.0,
            ),
            CamPreset::Free => {
                self.cam_preset = CamPreset::Free;
                let mut o = self.orbit;
                o.ortho = false;
                self.orbit = o;
                return;
            }
        };
        self.cam_preset = p;
        if animate {
            if self.orbit.ortho != goal.ortho {
                self.orbit = goal;
                self.orbit_goal = None;
            } else {
                self.orbit_goal = Some(goal);
            }
        } else {
            self.orbit = goal;
            self.orbit_goal = None;
        }
    }

    // ---------------------------------------------------------------- pieces

    pub fn add_piece_from(&mut self, item: &LibItem) {
        let id = self.project.next_id;
        self.project.next_id += 1;
        let v = &self.project.venue;
        let n = self.state().pieces.len() as f32;
        let x = ((n * 0.9) % 3.0) - 1.0;
        let z = -v.stage_depth * 0.45 + ((n * 0.6) % 1.8);
        let p = library::make_piece(item, id, [x, 0.0, z]);
        self.state_mut().pieces.push(p);
        self.sel = OBJ_PIECE + (self.state().pieces.len() - 1) as u32;
    }

    pub fn duplicate_piece(&mut self, idx: usize) {
        if let Some(p) = self.state().pieces.get(idx).cloned() {
            let mut q = p;
            q.id = self.project.next_id;
            self.project.next_id += 1;
            q.pos[0] += 0.5;
            q.pos[2] += 0.5;
            q.name = format!("{} copy", q.name);
            self.state_mut().pieces.push(q);
            self.sel = OBJ_PIECE + (self.state().pieces.len() - 1) as u32;
        }
    }

    pub fn delete_selected(&mut self) {
        let sel = self.sel;
        let kind = sel & OBJ_MASK;
        let idx = (sel & !OBJ_MASK) as usize;
        if kind == OBJ_PIECE && idx < self.state().pieces.len() {
            self.state_mut().pieces.remove(idx);
            self.sel = 0;
        } else if kind == OBJ_LIGHT && idx < self.state().lights.len() {
            self.state_mut().lights.remove(idx);
            self.sel = 0;
        } else if kind == OBJ_BATTEN && idx < self.state().battens.len() {
            self.remove_batten(idx);
            self.sel = 0;
        }
    }

    pub fn remove_batten(&mut self, idx: usize) {
        let s = self.state_mut();
        s.battens.remove(idx);
        s.lights.retain(|l| !matches!(l.mount, LightMount::Batten { index, .. } if index == idx));
        for l in s.lights.iter_mut() {
            if let LightMount::Batten { index, .. } = &mut l.mount {
                if *index > idx {
                    *index -= 1;
                }
            }
        }
        for p in s.pieces.iter_mut() {
            match p.flown_on {
                Some(b) if b == idx => p.flown_on = None,
                Some(b) if b > idx => p.flown_on = Some(b - 1),
                _ => {}
            }
        }
        self.snap_display();
    }

    /// Set a piece's world X/Z (handles revolve-relative storage).
    pub fn set_piece_world_xz(&mut self, idx: usize, x: f32, z: f32) {
        let v = self.project.venue.clone();
        let rev = self.disp.revolve;
        if let Some(p) = self.state_mut().pieces.get_mut(idx) {
            if p.on_revolve && v.has_revolve {
                let l = v3(x, 0.0, z - v.revolve_z).rot_y(-rev);
                p.pos[0] = l.x;
                p.pos[2] = l.z;
            } else {
                p.pos[0] = x;
                p.pos[2] = z;
            }
        }
    }

    pub fn set_on_revolve(&mut self, idx: usize, on: bool) {
        let v = self.project.venue.clone();
        let d = self.disp.clone();
        let battens = self.state().battens.clone();
        let Some(p) = self.state().pieces.get(idx).cloned() else { return };
        if p.on_revolve == on {
            return;
        }
        let (base, rot) = piece_world(&p, &v, &d, &battens);
        let pm = &mut self.state_mut().pieces[idx];
        pm.on_revolve = on;
        if on {
            let l = v3(base.x, 0.0, base.z - v.revolve_z).rot_y(-d.revolve);
            pm.pos[0] = l.x;
            pm.pos[2] = l.z;
            pm.rot = deg_wrap(rot - d.revolve);
        } else {
            pm.pos[0] = base.x;
            pm.pos[2] = base.z;
            pm.rot = deg_wrap(rot);
        }
    }

    fn snapv(&self, x: f32) -> f32 {
        if self.snap {
            (x / self.snap_step).round() * self.snap_step
        } else {
            x
        }
    }

    // ---------------------------------------------------------------- main frame

    pub fn frame(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let now = ctx.input(|i| i.time);
        let dt = ctx.input(|i| i.stable_dt).min(0.1);

        if self.animate_display(dt) {
            ctx.request_repaint();
        }
        if let Some(goal) = self.orbit_goal {
            let t = (dt * 6.0).min(1.0);
            self.orbit = self.orbit.lerp(&goal, t);
            if (self.orbit.target - goal.target).len() < 0.01 && (self.orbit.dist - goal.dist).abs() < 0.01 && (self.orbit.yaw - goal.yaw).abs() < 0.05 {
                self.orbit = goal;
                self.orbit_goal = None;
            }
            ctx.request_repaint();
        }
        self.warnings = checks::check(&self.project, &self.disp);

        self.keyboard(&ctx, now);

        egui::Panel::top("top").show(ui, |ui| self.top_bar(ui, now));
        egui::Panel::bottom("status").show(ui, |ui| self.status_bar(ui));
        egui::Panel::left("left").resizable(true).default_size(300.0).min_size(250.0).show(ui, |ui| {
            self.left_panel(ui, now);
        });
        egui::Panel::right("right").resizable(true).default_size(300.0).min_size(250.0).show(ui, |ui| {
            self.right_panel(ui, now);
        });
        egui::CentralPanel::default().frame(egui::Frame::NONE.fill(Color32::from_rgb(10, 11, 14))).show(ui, |ui| {
            self.viewport(ui, now);
        });
        self.builder_window(&ctx, now);

        // Toast.
        if let Some((msg, until)) = self.toast.clone() {
            if now < until {
                let r = self.last_view_rect;
                let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Foreground, egui::Id::unique("toast")));
                let g = painter.layout_no_wrap(msg, egui::FontId::proportional(15.0), Color32::WHITE);
                let pos = Pos2::new(r.center().x - g.size().x * 0.5, r.bottom() - 54.0);
                painter.rect_filled(Rect::from_min_size(pos - Vec2::new(12.0, 7.0), g.size() + Vec2::new(24.0, 14.0)), 8.0, Color32::from_rgba_unmultiplied(20, 22, 30, 235));
                painter.galley(pos, g, Color32::WHITE);
                ctx.request_repaint();
            } else {
                self.toast = None;
            }
        }

        self.handle_shot(&ctx);
    }

    fn handle_shot(&mut self, ctx: &egui::Context) {
        let Some(shot) = self.shot.as_mut() else { return };
        ctx.request_repaint();
        shot.frames += 1;
        if self.exit_after_shot && shot.frames % 30 == 0 && std::env::var("STAGECRAFT_TIMING").is_ok() {
            eprintln!("frame {} render {:.1} ms buffer {}x{}", shot.frames, self.last_render_ms, self.fb.w, self.fb.h);
            if shot.frames == 30 {
                let st = &self.project.scenes[self.project.current].state;
                for w in &self.warnings {
                    eprintln!("  warn: {} -> {}", st.pieces.get(w.piece).map(|p| p.name.as_str()).unwrap_or("?"), w.text);
                }
            }
        }
        if !shot.requested && shot.frames >= if self.exit_after_shot { 120 } else { 2 } {
            shot.requested = true;
            let slot = self.shot_slot.clone();
            ctx.request_screenshot(move |img| {
                *slot.lock().unwrap() = Some(img);
            });
        }
        let img = self.shot_slot.lock().unwrap().take();
        if let Some(img) = img {
            let shot = self.shot.take().unwrap();
            let [w, h] = img.size;
            let (mut x0, mut y0, mut x1, mut y1) = (0usize, 0usize, w, h);
            if let Some(r) = shot.crop {
                x0 = ((r.min.x * shot.ppp) as usize).min(w);
                y0 = ((r.min.y * shot.ppp) as usize).min(h);
                x1 = ((r.max.x * shot.ppp) as usize).clamp(x0 + 1, w);
                y1 = ((r.max.y * shot.ppp) as usize).clamp(y0 + 1, h);
            }
            let mut buf = image::RgbaImage::new((x1 - x0) as u32, (y1 - y0) as u32);
            for y in y0..y1 {
                for x in x0..x1 {
                    let c = img.pixels[y * w + x];
                    buf.put_pixel((x - x0) as u32, (y - y0) as u32, image::Rgba([c.r(), c.g(), c.b(), 255]));
                }
            }
            if let Some(parent) = std::path::Path::new(&shot.path).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            let res = buf.save(&shot.path);
            if self.exit_after_shot {
                if let Err(e) = res {
                    eprintln!("Could not save screenshot: {e}");
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                let now = ctx.input(|i| i.time);
                match res {
                    Ok(_) => self.notify(format!("Snapshot saved: {}", shot.path), now),
                    Err(e) => self.notify(format!("Could not save snapshot: {e}"), now),
                }
            }
        }
    }

    pub fn export_png(&mut self, ctx: &egui::Context) {
        if let Some(path) = rfd::FileDialog::new().add_filter("PNG image", &["png"]).set_file_name("stagecraft-view.png").save_file() {
            let ppp = ctx.pixels_per_point();
            self.shot = Some(Shot {
                path: path.to_string_lossy().to_string(),
                frames: 0,
                requested: false,
                crop: Some(self.last_view_rect),
                ppp,
            });
        }
    }

    pub fn save_json(&mut self, now: f64) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Stagecraft project", &["json"])
            .set_file_name(format!("{}.json", self.project.name.replace(['/', '\\', ':'], "-")))
            .save_file()
        {
            match serde_json::to_string_pretty(&self.project) {
                Ok(txt) => match std::fs::write(&path, txt) {
                    Ok(_) => self.notify("Project saved", now),
                    Err(e) => self.notify(format!("Save failed: {e}"), now),
                },
                Err(e) => self.notify(format!("Save failed: {e}"), now),
            }
        }
    }

    pub fn open_json(&mut self, now: f64) {
        if let Some(path) = rfd::FileDialog::new().add_filter("Stagecraft project", &["json"]).pick_file() {
            match std::fs::read_to_string(&path).map_err(|e| e.to_string()).and_then(|t| serde_json::from_str::<Project>(&t).map_err(|e| e.to_string())) {
                Ok(mut p) => {
                    if p.scenes.is_empty() {
                        p.scenes.push(SceneState { name: "Scene 1".into(), state: venues::fresh_state(&p.venue) });
                    }
                    p.current = p.current.min(p.scenes.len() - 1);
                    self.project = p;
                    self.sel = 0;
                    self.snap_display();
                    self.apply_cam_preset(self.cam_preset, false, 1.6);
                    self.notify("Project opened", now);
                }
                Err(e) => self.notify(format!("Couldn't open that file: {e}"), now),
            }
        }
    }

    pub fn set_venue(&mut self, v: Venue, now: f64) {
        self.project.venue = v;
        let venue = self.project.venue.clone();
        for sc in self.project.scenes.iter_mut() {
            let fresh = venues::fresh_state(&venue);
            sc.state.battens = fresh.battens;
            sc.state.lights = fresh.lights;
            sc.state.pit_lift = fresh.pit_lift;
            sc.state.stage_lift = 0.0;
            for p in sc.state.pieces.iter_mut() {
                p.flown_on = None;
                if !venue.has_revolve {
                    p.on_revolve = false;
                }
            }
        }
        self.sel = 0;
        self.snap_display();
        self.apply_cam_preset(if self.cam_preset == CamPreset::Free { CamPreset::Overview } else { self.cam_preset }, true, 1.6);
        self.notify(format!("Venue: {} – rigging and lights rebuilt", venue.name), now);
    }

    pub fn go_to_scene(&mut self, i: usize) {
        if i < self.project.scenes.len() {
            self.project.current = i;
            self.sel = 0;
            // Displayed values glide to the new scene's values (fly cues, revolve, lifts).
            if !self.animate {
                self.snap_display();
            }
        }
    }

    // ---------------------------------------------------------------- keyboard

    fn keyboard(&mut self, ctx: &egui::Context, now: f64) {
        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let (left, right, up, down, pgup, pgdn, q, e, del, dup, esc, f) = ctx.input(|i| {
            (
                i.key_pressed(egui::Key::ArrowLeft),
                i.key_pressed(egui::Key::ArrowRight),
                i.key_pressed(egui::Key::ArrowUp),
                i.key_pressed(egui::Key::ArrowDown),
                i.key_pressed(egui::Key::PageUp),
                i.key_pressed(egui::Key::PageDown),
                i.key_pressed(egui::Key::Q),
                i.key_pressed(egui::Key::E),
                i.key_pressed(egui::Key::Delete),
                i.modifiers.command && i.key_pressed(egui::Key::D),
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::F),
            )
        });
        let fine = ctx.input(|i| i.modifiers.shift);
        let step = if fine { 0.01 } else if self.snap { self.snap_step } else { 0.05 };
        let rstep = if fine { 1.0 } else { self.rot_step };
        if esc {
            self.sel = 0;
            self.aim_mode = false;
        }
        let kind = self.sel & OBJ_MASK;
        let idx = (self.sel & !OBJ_MASK) as usize;
        if kind == OBJ_PIECE && idx < self.state().pieces.len() {
            if self.state().pieces[idx].locked {
                return;
            }
            let (mut dx, mut dz, mut dy, mut dr) = (0.0, 0.0, 0.0, 0.0);
            if left {
                dx -= step;
            }
            if right {
                dx += step;
            }
            if up {
                dz -= step;
            }
            if down {
                dz += step;
            }
            if pgup {
                dy += step;
            }
            if pgdn {
                dy -= step;
            }
            if q {
                dr += rstep;
            }
            if e {
                dr -= rstep;
            }
            if dx != 0.0 || dz != 0.0 {
                let v = self.project.venue.clone();
                let d = self.disp.clone();
                let b = self.state().battens.clone();
                let (base, _) = piece_world(&self.state().pieces[idx], &v, &d, &b);
                self.set_piece_world_xz(idx, base.x + dx, base.z + dz);
            }
            let p = &mut self.state_mut().pieces[idx];
            p.pos[1] = (p.pos[1] + dy).max(-3.0);
            if dr != 0.0 {
                p.rot = deg_wrap(p.rot + dr);
            }
            if del {
                self.delete_selected();
            } else if dup {
                self.duplicate_piece(idx);
                self.notify("Duplicated", now);
            }
        } else if del && self.sel != 0 {
            self.delete_selected();
        }
        if f {
            self.focus_selected();
        }
        if kind == OBJ_REVOLVE && (q || e) {
            let s = self.state_mut();
            s.revolve_angle += if q { rstep } else { -rstep };
        }
    }

    pub fn focus_selected(&mut self) {
        let kind = self.sel & OBJ_MASK;
        let idx = (self.sel & !OBJ_MASK) as usize;
        let mut target = None;
        if kind == OBJ_PIECE {
            if let Some(p) = self.state().pieces.get(idx) {
                let (b, _) = piece_world(p, &self.project.venue, &self.disp, &self.state().battens);
                target = Some(b + v3(0.0, p.size[1] * 0.5, 0.0));
            }
        } else if kind == OBJ_LIGHT {
            if let Some(l) = self.state().lights.get(idx) {
                target = Some(V3::from_arr(l.target));
            }
        }
        if let Some(t) = target {
            let mut g = self.orbit;
            g.target = t;
            if !g.ortho {
                g.dist = g.dist.min(9.0);
            }
            self.orbit_goal = Some(g);
            self.cam_preset = CamPreset::Free;
        }
    }

    // ---------------------------------------------------------------- viewport

    fn viewport(&mut self, ui: &mut egui::Ui, now: f64) {
        let rect = ui.max_rect();
        self.last_view_rect = rect;
        let resp = ui.allocate_rect(rect, Sense::click_and_drag());
        let ppp = ui.ctx().pixels_per_point();
        let scale = ppp * self.render_scale;
        let bw = ((rect.width() * scale) as usize).max(16);
        let bh = ((rect.height() * scale) as usize).max(16);

        // ---- interaction (uses the previous frame's buffers) ----
        let to_buf = |p: Pos2| -> (f32, f32) { ((p.x - rect.min.x) * scale, (p.y - rect.min.y) * scale) };
        let ray_at = |cm: &CamMats, p: Pos2| -> (V3, V3) {
            let (x, y) = to_buf(p);
            cm.ray(x, y, bw as f32, bh as f32)
        };
        let hit_plane = |o: V3, d: V3, y: f32| -> Option<V3> {
            if d.y.abs() < 1e-5 {
                return None;
            }
            let t = (y - o.y) / d.y;
            if t < 0.0 {
                return None;
            }
            Some(o + d * t)
        };

        if resp.hovered() {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll.abs() > 0.0 {
                let f = (-scroll * 0.0015).exp();
                self.orbit_goal = None;
                if self.orbit.ortho {
                    self.orbit.ortho_h = (self.orbit.ortho_h * f).clamp(1.0, 80.0);
                } else {
                    self.orbit.dist = (self.orbit.dist * f).clamp(0.6, 120.0);
                }
            }
        }

        let shift = ui.input(|i| i.modifiers.shift);
        if resp.drag_started() {
            self.orbit_goal = None;
            let pos = resp.interact_pointer_pos().unwrap_or(rect.center());
            let primary = ui.input(|i| i.pointer.button_down(egui::PointerButton::Primary));
            let (bx, by) = to_buf(pos);
            let picked = if self.fb.w == bw && self.fb.h == bh { self.fb.pick(bx as usize, by as usize) } else { 0 };
            let kind = picked & OBJ_MASK;
            let idx = (picked & !OBJ_MASK) as usize;
            self.drag = Some(if !primary {
                Drag::Pan
            } else if kind == OBJ_PIECE && idx < self.state().pieces.len() && !self.state().pieces[idx].locked {
                self.sel = picked;
                let p = self.state().pieces[idx].clone();
                let (base, _) = piece_world(&p, &self.project.venue, &self.disp, &self.state().battens);
                let mut offset = V3::ZERO;
                if let Some(cm) = &self.last_cm {
                    let (o, d) = ray_at(cm, pos);
                    if let Some(h) = hit_plane(o, d, base.y) {
                        offset = base - h;
                    }
                }
                Drag::Piece { idx, offset, plane_y: base.y, vertical: shift, start_elev: p.pos[1] }
            } else if kind == OBJ_BATTEN {
                self.sel = picked;
                Drag::Batten { idx }
            } else if kind == OBJ_LIGHT {
                self.sel = picked;
                Drag::LightAim { idx }
            } else if kind == OBJ_REVOLVE {
                self.sel = picked;
                Drag::Revolve
            } else if kind == OBJ_PIT {
                self.sel = picked;
                Drag::Pit
            } else if kind == OBJ_LIFT {
                self.sel = picked;
                Drag::Lift
            } else if shift {
                Drag::Pan
            } else {
                Drag::Orbit
            });
        }
        if resp.dragged() {
            let delta = resp.drag_delta();
            let pos = resp.interact_pointer_pos().unwrap_or(rect.center());
            let max_trim = self.project.venue.max_trim();
            match self.drag.take() {
                Some(Drag::Orbit) => {
                    if self.orbit.ortho {
                        // In plan view, dragging empty space pans.
                        let k = self.orbit.ortho_h * 2.0 / rect.height();
                        self.orbit.target = self.orbit.target + v3(-delta.x * k, 0.0, -delta.y * k);
                    } else {
                        self.orbit.yaw -= delta.x * 0.3;
                        self.orbit.pitch = (self.orbit.pitch + delta.y * 0.3).clamp(-10.0, 89.0);
                        self.cam_preset = CamPreset::Free;
                    }
                    self.drag = Some(Drag::Orbit);
                }
                Some(Drag::Pan) => {
                    let cam = self.orbit.cam();
                    let f = (cam.target - cam.eye).norm();
                    let right = f.cross(cam.up()).norm();
                    let up = right.cross(f).norm();
                    let k = if self.orbit.ortho { self.orbit.ortho_h * 2.0 / rect.height() } else { self.orbit.dist * 0.0018 };
                    self.orbit.target = self.orbit.target - right * (delta.x * k) + up * (delta.y * k);
                    if !self.orbit.ortho {
                        self.cam_preset = CamPreset::Free;
                    }
                    self.drag = Some(Drag::Pan);
                }
                Some(Drag::Piece { idx, offset, plane_y, vertical, start_elev }) => {
                    if vertical {
                        if let Some(p) = self.state_mut().pieces.get_mut(idx) {
                            p.pos[1] = (p.pos[1] - delta.y * 0.01).max(-3.0);
                        }
                    } else if let Some(cm) = &self.last_cm {
                        let (o, d) = ray_at(cm, pos);
                        if let Some(h) = hit_plane(o, d, plane_y) {
                            let x = self.snapv(h.x + offset.x);
                            let z = self.snapv(h.z + offset.z);
                            let flown = self.state().pieces.get(idx).map(|p| p.flown_on.is_some()).unwrap_or(false);
                            if flown {
                                if let Some(p) = self.state_mut().pieces.get_mut(idx) {
                                    p.pos[0] = x;
                                }
                            } else {
                                self.set_piece_world_xz(idx, x, z);
                            }
                        }
                    }
                    let _ = start_elev;
                    self.drag = Some(Drag::Piece { idx, offset, plane_y, vertical, start_elev });
                }
                Some(Drag::Batten { idx }) => {
                    if let Some(b) = self.state_mut().battens.get_mut(idx) {
                        b.height = (b.height - delta.y * 0.02).clamp(0.3, max_trim);
                    }
                    if let Some(h) = self.disp.batten_h.get_mut(idx) {
                        *h = self.project.scenes[self.project.current].state.battens[idx].height;
                    }
                    self.drag = Some(Drag::Batten { idx });
                }
                Some(Drag::LightAim { idx }) => {
                    if let Some(cm) = &self.last_cm {
                        let (o, d) = ray_at(cm, pos);
                        if let Some(h) = hit_plane(o, d, 0.0) {
                            if let Some(l) = self.state_mut().lights.get_mut(idx) {
                                l.target = [h.x, 0.0, h.z];
                            }
                        }
                    }
                    self.drag = Some(Drag::LightAim { idx });
                }
                Some(Drag::Revolve) => {
                    let s = self.state_mut();
                    s.revolve_angle += delta.x * 0.4;
                    self.disp.revolve = self.state().revolve_angle;
                    self.drag = Some(Drag::Revolve);
                }
                Some(Drag::Pit) => {
                    let lo = -(self.project.venue.stage_height + 2.5);
                    let s = self.state_mut();
                    s.pit_lift = (s.pit_lift - delta.y * 0.01).clamp(lo, 0.0);
                    self.disp.pit = self.state().pit_lift;
                    self.drag = Some(Drag::Pit);
                }
                Some(Drag::Lift) => {
                    let s = self.state_mut();
                    s.stage_lift = (s.stage_lift - delta.y * 0.01).clamp(-3.0, 2.5);
                    self.disp.lift = self.state().stage_lift;
                    self.drag = Some(Drag::Lift);
                }
                None => {}
            }
        }
        if resp.drag_stopped() {
            // Snap rotation-free placement height to snap grid for tidy numbers.
            if let Some(Drag::Piece { idx, vertical: true, .. }) = self.drag {
                let snap = self.snap;
                let step = self.snap_step;
                if let Some(p) = self.state_mut().pieces.get_mut(idx) {
                    if snap {
                        p.pos[1] = (p.pos[1] / (step * 0.2)).round() * step * 0.2;
                    }
                }
            }
            self.drag = None;
        }
        if resp.clicked() {
            if let Some(pos) = resp.interact_pointer_pos() {
                let (bx, by) = to_buf(pos);
                if self.aim_mode && (self.sel & OBJ_MASK) == OBJ_LIGHT {
                    let idx = (self.sel & !OBJ_MASK) as usize;
                    if let Some(cm) = &self.last_cm {
                        let i = by as usize * self.fb.w + bx as usize;
                        if self.fb.w == bw && i < self.fb.depth.len() && self.fb.depth[i] < f32::MAX {
                            let nx = (bx + 0.0) / bw as f32 * 2.0 - 1.0;
                            let ny = 1.0 - by / bh as f32 * 2.0;
                            let p = cm.unproject(nx, ny, self.fb.depth[i]);
                            if let Some(l) = self.state_mut().lights.get_mut(idx) {
                                l.target = p.arr();
                            }
                            self.notify("Light re-aimed", now);
                        }
                    }
                    self.aim_mode = false;
                } else if self.fb.w == bw && self.fb.h == bh {
                    self.sel = self.fb.pick(bx as usize, by as usize);
                    if self.sel == OBJ_DRAFT {
                        self.sel = 0;
                    }
                }
            }
        }
        if resp.secondary_clicked() {
            self.sel = 0;
        }

        // ---- render ----
        let mut opts = BuildOpts {
            show_beams: self.show_beams,
            plan_view: self.orbit.ortho,
            work_mode: self.work_mode,
            show_sightlines: self.show_sightlines,
            show_labels: self.show_labels,
            show_rigging: self.show_rigging,
            selected: self.sel,
            draft: None,
            feet: self.feet,
        };
        if let Some(b) = &self.builder {
            if b.preview_on_stage && !b.item.parts.is_empty() {
                let pos = [0.0, 0.0, -self.project.venue.stage_depth * 0.5];
                opts.draft = Some(library::make_piece(&b.item, 0, pos));
            }
        }
        let (geo, labels) = scene::build(&self.project, &self.disp, &opts);
        let mut flagged: Vec<u32> = if self.show_warnings { self.warnings.iter().map(|w| OBJ_PIECE + w.piece as u32).collect() } else { vec![] };
        flagged.sort_unstable();
        flagged.dedup();
        let settings = raster::Settings {
            work_mode: self.work_mode,
            grid: self.show_grid,
            grid_step: if self.feet { 0.3048 * 2.0 } else { 1.0 },
            selected: self.sel,
            flagged,
            house_lights: if self.work_mode { 1.0 } else { self.state().house_lights },
            show_beams: self.show_beams,
            outlines: true,
        };
        let cam = self.orbit.cam();
        let t0 = std::time::Instant::now();
        let cm = raster::render(&geo, &cam, &settings, bw, bh, &mut self.fb);
        self.last_render_ms = t0.elapsed().as_secs_f32() * 1000.0;
        let img = egui::ColorImage { size: [self.fb.w, self.fb.h], source_size: Vec2::new(self.fb.w as f32, self.fb.h as f32), pixels: self.fb.color.clone() };
        match &mut self.tex {
            Some(t) => t.set(img, egui::TextureOptions::LINEAR),
            None => self.tex = Some(ui.ctx().load_texture("viewport", img, egui::TextureOptions::LINEAR)),
        }
        let painter = ui.painter_at(rect);
        if let Some(t) = &self.tex {
            painter.image(t.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        }

        // ---- labels ----
        if self.show_labels {
            let font = egui::FontId::proportional(11.5);
            for l in &labels {
                if let Some((x, y, z)) = cm.project(l.pos, bw as f32, bh as f32) {
                    if z > 1.0 || x < 0.0 || y < 0.0 || x >= bw as f32 || y >= bh as f32 {
                        continue;
                    }
                    // Hide labels that sit behind scenery.
                    let di = y as usize * self.fb.w + x as usize;
                    if let Some(&dz) = self.fb.depth.get(di) {
                        if dz < 1.0 && cm.lin_depth(dz) < cm.lin_depth(z) - 0.35 {
                            continue;
                        }
                    }
                    let p = Pos2::new(rect.min.x + x / scale, rect.min.y + y / scale);
                    let g = painter.layout_no_wrap(l.text.clone(), font.clone(), l.color);
                    let r = Rect::from_min_size(p - Vec2::new(g.size().x * 0.5 + 4.0, g.size().y * 0.5 + 2.0), g.size() + Vec2::new(8.0, 4.0));
                    painter.rect_filled(r, 4.0, Color32::from_rgba_unmultiplied(8, 10, 16, 170));
                    painter.galley(r.min + Vec2::new(4.0, 2.0), g, l.color);
                }
            }
        }
        self.last_cm = Some(cm);

        // ---- HUD ----
        let hud = format!(
            "{}  ·  {}  ·  {}",
            self.project.venue.name,
            self.project.scenes[self.project.current].name,
            if self.work_mode { "Work light" } else { "Cue lighting" }
        );
        let g = painter.layout_no_wrap(hud, egui::FontId::proportional(14.0), Color32::from_rgb(235, 225, 205));
        let r = Rect::from_min_size(rect.min + Vec2::new(10.0, 10.0), g.size() + Vec2::new(16.0, 10.0));
        painter.rect_filled(r, 6.0, Color32::from_rgba_unmultiplied(12, 14, 20, 200));
        painter.galley(r.min + Vec2::new(8.0, 5.0), g, Color32::WHITE);
        let view = self.cam_preset.label();
        let g = painter.layout_no_wrap(view.to_string(), egui::FontId::proportional(12.0), Color32::from_rgb(180, 190, 210));
        let r2 = Rect::from_min_size(Pos2::new(rect.min.x + 10.0, r.max.y + 6.0), g.size() + Vec2::new(12.0, 6.0));
        painter.rect_filled(r2, 5.0, Color32::from_rgba_unmultiplied(12, 14, 20, 170));
        painter.galley(r2.min + Vec2::new(6.0, 3.0), g, Color32::WHITE);
        if self.orbit.ortho {
            // Scale bar for the ground plan.
            let px_per_m = rect.height() / (self.orbit.ortho_h * 2.0);
            let len_m = if self.feet { 3.048 } else { 2.0 };
            let x0 = rect.max.x - 30.0 - px_per_m * len_m;
            let y0 = rect.max.y - 30.0;
            painter.line_segment([Pos2::new(x0, y0), Pos2::new(x0 + px_per_m * len_m, y0)], egui::Stroke::new(3.0, Color32::WHITE));
            painter.text(Pos2::new(x0, y0 - 6.0), egui::Align2::LEFT_BOTTOM, if self.feet { "10 ft" } else { "2 m" }, egui::FontId::proportional(12.0), Color32::WHITE);
            painter.text(Pos2::new(rect.center().x, rect.max.y - 14.0), egui::Align2::CENTER_CENTER, "▼ AUDIENCE ▼", egui::FontId::proportional(12.0), Color32::from_rgb(220, 190, 140));
        }
        if self.aim_mode {
            painter.text(rect.center_top() + Vec2::new(0.0, 24.0), egui::Align2::CENTER_CENTER, "Click on the stage to aim the selected light (Esc to cancel)", egui::FontId::proportional(15.0), Color32::from_rgb(255, 210, 90));
        }
    }
}
