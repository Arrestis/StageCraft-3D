// Copyright (C) 2026 Doug Rumble
// SPDX-License-Identifier: GPL-3.0-only

//! The start-up screen (the app's picture or your own, your theatre company's name, the app's
//! name and its author), its settings, and the About box with the licence.

use crate::app::App;
use crate::panels::ACCENT;
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontId, Pos2, Rect, RichText, Sense, Stroke, Vec2};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const AUTHOR: &str = "Doug Rumble";
pub const COPYRIGHT: &str = "Copyright © 2026 Doug Rumble";
const LICENSE_TEXT: &str = include_str!("../LICENSE");
const GOLD: Color32 = Color32::from_rgb(206, 166, 86);

/// Settings kept on this computer for every project (not saved in project files).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Prefs {
    /// Theatre company shown on the start-up screen.
    pub company: String,
    /// Show the start-up screen when the app opens.
    pub show_splash: bool,
    /// How long it stays up, in seconds.
    pub splash_secs: f32,
    /// Show your own picture instead of the app's picture.
    pub use_own_picture: bool,
    /// Your picture, copied into the settings folder.
    pub picture: String,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs { company: String::new(), show_splash: true, splash_secs: 4.0, use_own_picture: false, picture: String::new() }
    }
}

/// Where Stagecraft keeps its settings: the usual place on each system.
pub fn settings_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let dir = std::env::var_os("APPDATA").map(|p| PathBuf::from(p).join("Stagecraft 3D"));
    #[cfg(target_os = "macos")]
    let dir = std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library").join("Application Support").join("Stagecraft 3D"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let dir = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .map(|p| p.join("stagecraft-3d"));
    dir
}

impl Prefs {
    fn file() -> Option<PathBuf> {
        settings_dir().map(|d| d.join("settings.json"))
    }
    pub fn load() -> Prefs {
        Self::file().and_then(|f| std::fs::read_to_string(f).ok()).and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default()
    }
    pub fn save(&self) -> Result<(), String> {
        let f = Self::file().ok_or("this system has no settings folder")?;
        if let Some(d) = f.parent() {
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        let txt = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(f, txt).map_err(|e| e.to_string())
    }
}

/// The start-up screen while it shows.
pub struct Splash {
    started: Option<f64>,
    /// Stay up until clicked (used for test pictures).
    hold: bool,
}

impl Splash {
    pub fn new(hold: bool) -> Self {
        Splash { started: None, hold }
    }
}

fn texture(ctx: &egui::Context, name: &str, img: &image::RgbaImage) -> egui::TextureHandle {
    let size = [img.width() as usize, img.height() as usize];
    let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
    ctx.load_texture(name, color, egui::TextureOptions::LINEAR)
}

/// The app's picture, large (made smaller once so it stays crisp on screen).
fn load_big_logo(ctx: &egui::Context) -> Option<egui::TextureHandle> {
    let img = image::load_from_memory(include_bytes!("../assets/icon-1024.png")).ok()?;
    let img = img.resize(640, 640, image::imageops::FilterType::CatmullRom).to_rgba8();
    Some(texture(ctx, "stagecraft-big-logo", &img))
}

fn load_picture(ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
    let img = image::open(path).ok()?;
    let img = if img.width() > 1200 || img.height() > 1200 { img.resize(1200, 1200, image::imageops::FilterType::Triangle) } else { img };
    Some(texture(ctx, "stagecraft-own-picture", &img.to_rgba8()))
}

/// Keep a copy of a picture (PNG or JPG) in `dir` for the start-up screen: a PNG no bigger than
/// 1600 pixels, so it loads quickly and still shows if the original is moved. Returns where it is.
pub fn keep_picture(src: &std::path::Path, dir: &std::path::Path) -> Result<PathBuf, String> {
    let img = image::open(src).map_err(|e| e.to_string())?;
    let img = if img.width() > 1600 || img.height() > 1600 { img.resize(1600, 1600, image::imageops::FilterType::Triangle) } else { img };
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let dest = dir.join("start-up picture.png");
    img.to_rgba8().save(&dest).map_err(|e| e.to_string())?;
    Ok(dest)
}

/// A soft round glow: a fan of triangles fading from the centre colour to clear.
fn glow(p: &egui::Painter, c: Pos2, rx: f32, ry: f32, col: Color32) {
    let mut m = egui::Mesh::default();
    m.colored_vertex(c, col);
    let n = 64;
    for i in 0..=n {
        let t = i as f32 / n as f32 * std::f32::consts::TAU;
        m.colored_vertex(Pos2::new(c.x + rx * t.cos(), c.y + ry * t.sin()), Color32::TRANSPARENT);
    }
    for i in 0..n as u32 {
        m.add_triangle(0, i + 1, i + 2);
    }
    p.add(egui::Shape::mesh(m));
}

fn centred(p: &egui::Painter, x: f32, y: f32, g: std::sync::Arc<egui::Galley>) -> f32 {
    let h = g.size().y;
    p.galley(Pos2::new(x - g.size().x * 0.5, y), g, Color32::WHITE);
    h
}

impl App {
    fn save_prefs(&mut self, now: f64) {
        if self.test_mode {
            return;
        }
        if let Err(e) = self.prefs.save() {
            self.notify(format!("Couldn't save the start-up settings: {e}"), now);
        }
    }

    /// The picture for the start-up screen: your own if chosen (and readable), else the app's.
    pub fn startup_picture(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        if self.prefs.use_own_picture && !self.prefs.picture.is_empty() {
            let fresh = self.own_picture.as_ref().map(|(p, _)| p == &self.prefs.picture).unwrap_or(false);
            if !fresh {
                let tex = load_picture(ctx, &self.prefs.picture);
                self.own_picture = Some((self.prefs.picture.clone(), tex));
            }
            if let Some((_, Some(t))) = &self.own_picture {
                return Some(t.clone());
            }
        }
        if self.big_logo.is_none() {
            self.big_logo = load_big_logo(ctx);
        }
        self.big_logo.clone()
    }

    /// Pick a PNG or JPG, keep a copy in the settings folder and use it on the start-up screen.
    fn choose_picture(&mut self, now: f64) {
        let Some(path) = rfd::FileDialog::new().add_filter("Picture (PNG or JPG)", &["png", "jpg", "jpeg"]).pick_file() else { return };
        let Some(dir) = settings_dir() else {
            self.notify("This system has no settings folder to keep the picture in", now);
            return;
        };
        match keep_picture(&path, &dir) {
            Ok(dest) => {
                self.prefs.picture = dest.to_string_lossy().to_string();
                self.prefs.use_own_picture = true;
                self.own_picture = None;
                self.save_prefs(now);
                self.notify("Start-up picture set", now);
            }
            Err(e) => self.notify(format!("Couldn't use that picture: {e}"), now),
        }
    }

    /// Draw the start-up screen over everything; a click or key press (or the time running
    /// out) closes it.
    pub fn splash_overlay(&mut self, ctx: &egui::Context, now: f64) {
        let Some(sp) = self.splash.as_mut() else { return };
        let start = *sp.started.get_or_insert(now);
        let hold = sp.hold;
        let t = (now - start) as f32;
        let dur = self.prefs.splash_secs.clamp(1.0, 30.0);
        let pressed = ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Key { pressed: true, .. } | egui::Event::PointerButton { pressed: true, .. })));
        if (!hold && t >= dur) || (t > 0.3 && pressed) {
            self.splash = None;
            return;
        }
        let a = if hold { 1.0 } else { (t / 0.3).clamp(0.0, 1.0).min(((dur - t) / 0.5).clamp(0.0, 1.0)) };
        let left = if hold { None } else { Some(1.0 - t / dur) };
        let pic = self.startup_picture(ctx);
        let company = self.prefs.company.trim().to_string();
        let screen = ctx.content_rect();
        egui::Area::new(egui::Id::unique("stagecraft-splash"))
            .order(egui::Order::Foreground)
            .fixed_pos(screen.min)
            .constrain(false)
            .show(ctx, |ui| {
                let (r, _) = ui.allocate_exact_size(screen.size(), Sense::click());
                paint_splash(ui.painter(), r, a, pic.as_ref(), &company, left);
            });
        ctx.request_repaint();
    }

    /// File ▸ Start-up screen: company name, picture, on/off and how long.
    pub fn startup_window(&mut self, ctx: &egui::Context, now: f64) {
        if !self.show_startup_win {
            return;
        }
        let thumb = self.startup_picture(ctx);
        let has_own = !self.prefs.picture.is_empty();
        let own_ok = self.own_picture.as_ref().map(|(_, t)| t.is_some()).unwrap_or(true);
        let (mut open, mut changed, mut choose, mut remove, mut preview) = (true, false, false, false, false);
        egui::Window::new("🖼 Start-up screen")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(440.0)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| {
                ui.label(RichText::new("Shown for a few seconds each time Stagecraft 3D opens. Click it or press a key to skip it.").color(Color32::from_gray(170)));
                ui.add_space(6.0);
                ui.label(RichText::new("Theatre company").strong());
                changed |= ui.add(egui::TextEdit::singleline(&mut self.prefs.company).hint_text("Your theatre company's name").desired_width(f32::INFINITY)).changed();
                ui.add_space(8.0);
                ui.label(RichText::new("Picture").strong());
                ui.horizontal(|ui| {
                    if let Some(t) = &thumb {
                        ui.add(egui::Image::new(t).fit_to_exact_size(Vec2::new(104.0, 104.0)));
                    }
                    ui.vertical(|ui| {
                        changed |= ui.radio_value(&mut self.prefs.use_own_picture, false, "The Stagecraft donkey (the app's picture)").changed();
                        ui.add_enabled_ui(has_own, |ui| {
                            changed |= ui.radio_value(&mut self.prefs.use_own_picture, true, "My own picture").changed();
                        });
                        ui.horizontal(|ui| {
                            if ui.button("📂 Choose a picture…").on_hover_text("A PNG or JPG: your logo, a poster or a photo").clicked() {
                                choose = true;
                            }
                            if has_own && ui.small_button("Remove").clicked() {
                                remove = true;
                            }
                        });
                        if has_own && !own_ok {
                            ui.label(RichText::new("That picture can't be read any more – choose it again.").color(Color32::from_rgb(255, 150, 120)));
                        }
                    });
                });
                ui.add_space(8.0);
                changed |= ui.checkbox(&mut self.prefs.show_splash, "Show the start-up screen when the app opens").changed();
                changed |= ui.add(egui::Slider::new(&mut self.prefs.splash_secs, 1.0..=10.0).step_by(0.5).suffix(" s").text("Show it for")).changed();
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("👁 Preview").clicked() {
                        preview = true;
                    }
                    ui.label(RichText::new("Kept on this computer for every project.").size(11.0).color(Color32::from_gray(140)));
                });
            });
        if choose {
            self.choose_picture(now);
        }
        if remove {
            if let Some(dir) = settings_dir() {
                let _ = std::fs::remove_file(dir.join("start-up picture.png"));
            }
            self.prefs.picture.clear();
            self.prefs.use_own_picture = false;
            self.own_picture = None;
            changed = true;
        }
        if changed {
            self.save_prefs(now);
        }
        if preview {
            self.splash = Some(Splash::new(false));
        }
        if !open {
            self.show_startup_win = false;
        }
    }

    /// File ▸ About: version, author, copyright and the GNU GPL v3 licence.
    pub fn about_window(&mut self, ctx: &egui::Context) {
        if !self.show_about {
            return;
        }
        if self.big_logo.is_none() {
            self.big_logo = load_big_logo(ctx);
        }
        let logo = self.big_logo.clone();
        let mut open = true;
        egui::Window::new("ℹ About Stagecraft 3D")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .default_width(480.0)
            .pivot(egui::Align2::CENTER_CENTER)
            .default_pos(ctx.content_rect().center())
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(t) = &logo {
                        ui.add(egui::Image::new(t).fit_to_exact_size(Vec2::new(112.0, 112.0)));
                    }
                    ui.vertical(|ui| {
                        ui.label(RichText::new("Stagecraft 3D").size(26.0).color(ACCENT));
                        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                        ui.label(RichText::new(format!("by {AUTHOR}")).italics().size(15.0));
                        ui.add_space(4.0);
                        ui.label(RichText::new("Plan and preview theatre sets in 3D inside real venues.").color(Color32::from_gray(170)));
                    });
                });
                ui.separator();
                ui.label(RichText::new("Free and open-source software").strong().color(GOLD));
                ui.label(format!("{COPYRIGHT}."));
                ui.label(
                    "Stagecraft 3D is free software: you can use it, share it and change it under the terms of the \
                     GNU General Public License, version 3, as published by the Free Software Foundation.",
                );
                ui.label(
                    RichText::new(
                        "It comes with ABSOLUTELY NO WARRANTY, without even the implied warranty of MERCHANTABILITY \
                         or FITNESS FOR A PARTICULAR PURPOSE. See the licence below for the details.",
                    )
                    .size(12.0)
                    .color(Color32::from_gray(165)),
                );
                ui.add_space(2.0);
                ui.hyperlink_to("Source code on GitHub", env!("CARGO_PKG_REPOSITORY"));
                egui::CollapsingHeader::new("Read the full licence (GNU GPL v3)").default_open(false).show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(280.0).show(ui, |ui| {
                        ui.label(RichText::new(LICENSE_TEXT.replace('\r', "")).monospace().size(11.0));
                    });
                });
                ui.add_space(4.0);
                ui.label(
                    RichText::new("Made with Rust and egui (MIT / Apache-2.0 licences). The app's picture is the project's own artwork.")
                        .size(11.0)
                        .color(Color32::from_gray(140)),
                );
            });
        if !open {
            self.show_about = false;
        }
    }
}

/// The start-up screen: a dark stage with a soft red glow, and a card with the picture, the
/// theatre company's name, "Stagecraft 3D" and "by Doug Rumble" in small writing.
fn paint_splash(p: &egui::Painter, r: Rect, a: f32, pic: Option<&egui::TextureHandle>, company: &str, left: Option<f32>) {
    let al = |c: Color32| c.gamma_multiply(a);
    p.rect_filled(r, 0.0, al(Color32::from_rgb(10, 8, 12)));
    let c = r.center();
    glow(p, c, r.width() * 0.55, r.height() * 0.62, al(Color32::from_rgba_unmultiplied(120, 14, 36, 150)));
    glow(p, Pos2::new(c.x, r.top() - r.height() * 0.05), r.width() * 0.32, r.height() * 0.5, al(Color32::from_rgba_unmultiplied(255, 214, 150, 26)));

    let card_w = (r.width() - 60.0).clamp(320.0, 620.0);
    let max_pic_h = (r.height() * 0.42).clamp(110.0, 330.0);
    let pic_size = pic.map(|t| {
        let s = t.size_vec2();
        s * (max_pic_h / s.y.max(1.0)).min((card_w - 64.0) / s.x.max(1.0))
    });
    let wrap = card_w - 64.0;
    let g_company = if company.is_empty() {
        let mut job = LayoutJob::default();
        job.append("Your theatre company's name", 0.0, TextFormat { font_id: FontId::proportional(19.0), color: al(Color32::from_gray(112)), italics: true, ..Default::default() });
        p.layout_job(job)
    } else {
        p.layout(company.to_string(), FontId::proportional(29.0), al(Color32::from_rgb(244, 224, 182)), wrap)
    };
    let g_title = p.layout_no_wrap("Stagecraft 3D".into(), FontId::proportional(48.0), al(ACCENT));
    let mut job = LayoutJob::default();
    job.append(&format!("by {AUTHOR}"), 0.0, TextFormat { font_id: FontId::proportional(15.0), color: al(Color32::from_rgb(204, 194, 178)), italics: true, ..Default::default() });
    let g_by = p.layout_job(job);
    let g_foot = p.layout_no_wrap(
        format!("Version {}  ·  free and open-source software (GNU GPL v3)", env!("CARGO_PKG_VERSION")),
        FontId::proportional(11.5),
        al(Color32::from_gray(140)),
    );
    let hint = if company.is_empty() { "Add your company's name and picture in File → Start-up screen  ·  click to start" } else { "Click or press any key to start" };
    let g_hint = p.layout_no_wrap(hint.into(), FontId::proportional(11.5), al(Color32::from_gray(112)));

    let pad = 30.0;
    let pic_h = pic_size.map(|s| s.y + 22.0).unwrap_or(0.0);
    let h = pad + pic_h + g_company.size().y + 14.0 + 10.0 + g_title.size().y + 2.0 + g_by.size().y + 20.0 + g_foot.size().y + 4.0 + g_hint.size().y + pad;
    let card = Rect::from_center_size(c, Vec2::new(card_w, h));
    p.rect(card, 18.0, al(Color32::from_rgb(25, 20, 26)), Stroke::new(1.5, al(GOLD)), egui::StrokeKind::Inside);
    p.rect_stroke(card.shrink(6.0), 13.0, Stroke::new(0.6, al(GOLD.gamma_multiply(0.45))), egui::StrokeKind::Inside);

    let mut y = card.top() + pad;
    if let (Some(t), Some(s)) = (pic, pic_size) {
        let pr = Rect::from_min_size(Pos2::new(c.x - s.x * 0.5, y), s);
        p.image(t.id(), pr, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), al(Color32::WHITE));
        y += s.y + 22.0;
    }
    y += centred(p, c.x, y, g_company) + 14.0;
    // A gold rule between the company and the app's name.
    p.line_segment([Pos2::new(c.x - 70.0, y), Pos2::new(c.x + 70.0, y)], Stroke::new(1.0, al(GOLD)));
    p.circle_filled(Pos2::new(c.x, y), 3.0, al(GOLD));
    y += 10.0;
    y += centred(p, c.x, y, g_title) + 2.0;
    y += centred(p, c.x, y, g_by) + 20.0;
    y += centred(p, c.x, y, g_foot) + 4.0;
    centred(p, c.x, y, g_hint);
    if let Some(f) = left {
        let w = (card_w - 48.0) * f.clamp(0.0, 1.0);
        let yb = card.bottom() - 12.0;
        p.line_segment([Pos2::new(c.x - w * 0.5, yb), Pos2::new(c.x + w * 0.5, yb)], Stroke::new(2.0, al(ACCENT.gamma_multiply(0.8))));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_big_jpg_is_kept_as_a_smaller_png() {
        let dir = std::env::temp_dir().join(format!("stagecraft-picture-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("company logo.jpg");
        image::RgbImage::from_fn(2400, 1200, |x, y| image::Rgb([(x % 256) as u8, (y % 256) as u8, 90])).save(&src).unwrap();
        let kept = keep_picture(&src, &dir.join("settings")).unwrap();
        let back = image::open(&kept).unwrap();
        assert_eq!((back.width(), back.height()), (1600, 800));
        assert!(kept.extension().map(|e| e == "png").unwrap_or(false));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn settings_from_an_older_file_fill_in_defaults() {
        let p: Prefs = serde_json::from_str(r#"{ "company": "Armidale Players" }"#).unwrap();
        assert_eq!(p.company, "Armidale Players");
        assert!(p.show_splash);
        assert_eq!(p.splash_secs, 4.0);
        assert!(!p.use_own_picture);
    }
}
