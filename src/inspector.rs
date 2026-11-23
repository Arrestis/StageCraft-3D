//! Right-hand panel (inspector + outliner).

use crate::app::*;
use crate::model::*;
use crate::panels::*;
use crate::scene::*;
use egui::{Color32, RichText, Ui};

pub fn scaled_parts(p: &Piece) -> Vec<Part> {
    let sx = p.size[0] / p.base_size[0].max(1e-3);
    let sy = p.size[1] / p.base_size[1].max(1e-3);
    let sz = p.size[2] / p.base_size[2].max(1e-3);
    p.parts
        .iter()
        .map(|q| {
            let mut q = q.clone();
            q.offset = [q.offset[0] * sx, q.offset[1] * sy, q.offset[2] * sz];
            q.size = [q.size[0] * sx, q.size[1] * sy, q.size[2] * sz];
            if let Some(h) = q.hinge.as_mut() {
                if let Some(pv) = h.pivot.as_mut() {
                    *pv = [pv[0] * sx, pv[1] * sy, pv[2] * sz];
                }
            }
            q
        })
        .collect()
}

/// One big Open / Shut button for pieces with doors, lids, gates or drawers.
fn moving_parts_bar(ui: &mut Ui, p: &mut Piece) {
    if !crate::library::has_moving_parts(p) {
        return;
    }
    let open = crate::library::is_open(p);
    ui.horizontal(|ui| {
        let txt = if open { "🚪 Shut" } else { "🚪 Open" };
        if ui.add(egui::Button::new(RichText::new(txt).size(15.0)).min_size(egui::vec2(90.0, 26.0))).on_hover_text("Open or shut its doors, lids, gates and drawers (key O)").clicked() {
            crate::library::toggle_open(p);
        }
        ui.label(RichText::new(if open { "Currently open" } else { "Currently shut" }).color(Color32::from_gray(160)));
    });
}

/// Every setting of a library piece, grouped under headings; the piece is rebuilt on change.
fn template_editor(ui: &mut Ui, p: &mut Piece, feet: bool) {
    let Some(mut t) = p.template.clone() else { return };
    let mut groups: Vec<String> = Vec::new();
    for q in &t.params {
        if !groups.contains(&q.group) {
            groups.push(q.group.clone());
        }
    }
    let mut changed = false;
    for g in &groups {
        let title = if g.is_empty() { "Settings" } else { g.as_str() };
        egui::CollapsingHeader::new(title).id_salt(("tpl", g.as_str())).default_open(true).show(ui, |ui| {
            for q in t.params.iter_mut().filter(|q| &q.group == g) {
                let c = match q.kind {
                    ParamKind::Length => {
                        let mut c = len_edit(ui, &q.label, &mut q.value, feet, q.min, q.max);
                        if crate::library::is_open_param(q) {
                            ui.horizontal(|ui| {
                                if ui.small_button("Shut").clicked() {
                                    q.value = q.min;
                                    c = true;
                                }
                                if ui.small_button("Open").clicked() {
                                    q.value = q.max;
                                    c = true;
                                }
                            });
                        }
                        c
                    }
                    ParamKind::Angle => deg_edit(ui, &q.label, &mut q.value, q.min, q.max),
                    ParamKind::Count => {
                        let mut n = q.value.round() as i32;
                        let r = ui.add(egui::Slider::new(&mut n, q.min as i32..=q.max as i32).text(q.label.as_str()));
                        if r.changed() {
                            q.value = n as f32;
                        }
                        r.changed()
                    }
                    ParamKind::Toggle => {
                        let mut b = q.value > 0.5;
                        let r = ui.checkbox(&mut b, q.label.as_str()).changed();
                        if r {
                            q.value = if b { 1.0 } else { 0.0 };
                        }
                        r
                    }
                    ParamKind::Swing => {
                        let mut c = ui.add(egui::Slider::new(&mut q.value, q.min..=q.max).suffix("°").text(q.label.as_str())).changed();
                        ui.horizontal(|ui| {
                            for (lbl, v) in [("Shut", 0.0f32), ("Ajar", 20.0), ("Half", 45.0), ("Open", 90.0f32.min(q.max))] {
                                if ui.small_button(lbl).clicked() {
                                    q.value = v;
                                    c = true;
                                }
                            }
                        });
                        c
                    }
                };
                changed |= c;
            }
        });
    }
    if changed {
        p.template = Some(t);
        crate::library::regenerate(p);
    }
    if ui.small_button("↺ Reset settings to library defaults").clicked() {
        let key = p.template.as_ref().map(|t| t.key.clone()).unwrap_or_default();
        let lib = crate::library::presets();
        // Prefer the library item with the same name (several items share a template).
        let def = lib
            .iter()
            .find(|i| i.name == p.name && i.template.as_ref().map(|t| t.key == key).unwrap_or(false))
            .or_else(|| lib.iter().find(|i| i.template.as_ref().map(|t| t.key == key).unwrap_or(false)))
            .and_then(|i| i.template.clone());
        if let Some(d) = def {
            p.template = Some(d);
            crate::library::regenerate(p);
        }
    }
}

/// Hinged parts of a hand-built piece (made in the custom builder).
fn hinge_list(ui: &mut Ui, p: &mut Piece) {
    if !p.parts.iter().any(|q| q.hinge.is_some()) {
        return;
    }
    egui::CollapsingHeader::new("Doors & moving parts").default_open(true).show(ui, |ui| {
        // Parts sharing a name (a door leaf and its handles) move together.
        let name_of = |i: usize, h: &Hinge| if h.label.is_empty() { format!("Part {}", i + 1) } else { h.label.clone() };
        let mut names: Vec<(String, f32, f32)> = Vec::new();
        for (i, q) in p.parts.iter().enumerate() {
            if let Some(h) = &q.hinge {
                let n = name_of(i, h);
                if !names.iter().any(|x| x.0 == n) {
                    names.push((n, h.open_deg, h.max_deg));
                }
            }
        }
        for (n, mut val, max) in names {
            if ui.add(egui::Slider::new(&mut val, 0.0..=max.max(1.0)).suffix("°").text(n.as_str())).changed() {
                for (i, q) in p.parts.iter_mut().enumerate() {
                    let hit = q.hinge.as_ref().map(|h| name_of(i, h) == n).unwrap_or(false);
                    if hit {
                        if let Some(h) = q.hinge.as_mut() {
                            h.open_deg = val;
                        }
                    }
                }
            }
        }
    });
}

pub const GELS:[(&str, [f32; 3]); 10] = [
    ("Open white", [1.0, 0.97, 0.92]),
    ("Warm", [1.0, 0.8, 0.58]),
    ("Amber", [1.0, 0.62, 0.22]),
    ("Straw", [1.0, 0.9, 0.6]),
    ("Cool", [0.64, 0.78, 1.0]),
    ("Night blue", [0.25, 0.38, 1.0]),
    ("Lavender", [0.72, 0.55, 1.0]),
    ("Red", [1.0, 0.12, 0.1]),
    ("Green", [0.3, 1.0, 0.35]),
    ("Magenta", [1.0, 0.25, 0.75]),
];

impl App {
    pub fn right_panel(&mut self, ui: &mut Ui, now: f64) {
        let avail = ui.available_height();
        egui::ScrollArea::vertical().id_salt("insp").max_height(avail * 0.6).auto_shrink([false, true]).show(ui, |ui| {
            self.inspector(ui, now);
            if self.show_warn_list {
                heading(ui, "All set warnings");
                if self.warnings.is_empty() {
                    ui.label("Nothing to report – the set fits.");
                }
                let mut pick = None;
                for w in &self.warnings {
                    let name = self.state().pieces.get(w.piece).map(|p| p.name.clone()).unwrap_or_default();
                    if ui.link(format!("⚠ {name}: {}", w.text)).clicked() {
                        pick = Some(w.piece);
                    }
                }
                if let Some(p) = pick {
                    self.sel = OBJ_PIECE + p as u32;
                }
            }
        });
        ui.separator();
        heading(ui, "Scene outliner");
        egui::ScrollArea::vertical().id_salt("outl").auto_shrink([false, false]).show(ui, |ui| self.outliner(ui));
    }

    fn outliner(&mut self, ui: &mut Ui) {
        let sel = self.sel;
        let feet = self.feet;
        let mut pick = None;
        let warn_set: Vec<usize> = self.warnings.iter().map(|w| w.piece).collect();
        let np = self.state().pieces.len();
        egui::CollapsingHeader::new(format!("Set pieces ({np})")).default_open(true).show(ui, |ui| {
            let s = self.state_mut();
            for (i, p) in s.pieces.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut p.visible, "");
                    let mut t = RichText::new(&p.name);
                    if warn_set.contains(&i) {
                        t = t.color(Color32::from_rgb(255, 130, 110));
                    }
                    if ui.selectable_label(sel == OBJ_PIECE + i as u32, t).clicked() {
                        pick = Some(OBJ_PIECE + i as u32);
                    }
                    if p.locked {
                        ui.label("🔒");
                    }
                    if p.on_revolve {
                        ui.label("⟳");
                    }
                    if p.flown_on.is_some() {
                        ui.label("⬆");
                    }
                });
            }
        });
        let nb = self.state().battens.len();
        egui::CollapsingHeader::new(format!("Fly lines & soft goods ({nb})")).default_open(false).show(ui, |ui| {
            let s = self.state_mut();
            for (i, b) in s.battens.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut b.visible, "");
                    if ui.selectable_label(sel == OBJ_BATTEN + i as u32, format!("{} · {}", b.name, fmt_len(b.height, feet))).clicked() {
                        pick = Some(OBJ_BATTEN + i as u32);
                    }
                });
            }
        });
        let nl = self.state().lights.len();
        egui::CollapsingHeader::new(format!("Lights ({nl})")).default_open(false).show(ui, |ui| {
            for (i, l) in self.state().lights.iter().enumerate() {
                if ui.selectable_label(sel == OBJ_LIGHT + i as u32, format!("{} · {:.0}%", l.name, l.intensity * 100.0)).clicked() {
                    pick = Some(OBJ_LIGHT + i as u32);
                }
            }
        });
        let v = self.project.venue.clone();
        egui::CollapsingHeader::new("Stage machinery").default_open(true).show(ui, |ui| {
            if v.has_revolve && ui.selectable_label(sel == OBJ_REVOLVE, "⟳ Revolve").clicked() {
                pick = Some(OBJ_REVOLVE);
            }
            if v.has_pit && ui.selectable_label(sel == OBJ_PIT, "🎻 Orchestra pit lift").clicked() {
                pick = Some(OBJ_PIT);
            }
            if v.has_stage_lift && ui.selectable_label(sel == OBJ_LIFT, "⬍ Stage lift").clicked() {
                pick = Some(OBJ_LIFT);
            }
            if !v.has_revolve && !v.has_pit && !v.has_stage_lift {
                ui.label(RichText::new("This venue has no revolve, pit or lifts (switch them on in the Venue tab).").color(Color32::from_gray(150)));
            }
        });
        if let Some(p) = pick {
            self.sel = p;
        }
    }

    fn inspector(&mut self, ui: &mut Ui, now: f64) {
        let kind = self.sel & OBJ_MASK;
        let idx = (self.sel & !OBJ_MASK) as usize;
        let feet = self.feet;
        if kind == OBJ_PIECE && idx < self.state().pieces.len() {
            self.piece_inspector(ui, idx, now);
        } else if kind == OBJ_BATTEN && idx < self.state().battens.len() {
            self.batten_inspector(ui, idx);
        } else if kind == OBJ_LIGHT && idx < self.state().lights.len() {
            self.light_inspector(ui, idx);
        } else if kind == OBJ_REVOLVE {
            heading(ui, "⟳ Revolve");
            let s = self.state_mut();
            ui.add(egui::Slider::new(&mut s.revolve_angle, -360.0..=360.0).suffix("°").text("Angle"));
            ui.horizontal_wrapped(|ui| {
                for d in [-90.0f32, -15.0, -1.0, 1.0, 15.0, 90.0, 180.0] {
                    if ui.button(format!("{:+}°", d)).clicked() {
                        s.revolve_angle += d;
                    }
                }
                if ui.button("Reset 0°").clicked() {
                    s.revolve_angle = 0.0;
                }
            });
            ui.label(RichText::new("Pieces set to 'ride the revolve' turn with it. Drag the disc sideways in the view to turn it.").color(Color32::from_gray(160)));
        } else if kind == OBJ_PIT {
            heading(ui, "🎻 Orchestra pit");
            let sh = self.project.venue.stage_height;
            let is_lift = self.project.venue.pit_is_lift;
            let s = self.state_mut();
            ui.add(egui::Slider::new(&mut s.pit_lift, -(sh + 2.5)..=0.0).text("Height vs stage").suffix(" m"));
            ui.label(fmt_len(s.pit_lift, feet));
            ui.horizontal_wrapped(|ui| {
                if ui.button("Pit level").clicked() {
                    s.pit_lift = -(sh + 1.5);
                }
                if ui.button("Auditorium level").clicked() {
                    s.pit_lift = -sh;
                }
                if ui.button("Stage level (forestage)").clicked() {
                    s.pit_lift = 0.0;
                }
            });
            if !is_lift {
                ui.label(RichText::new("This venue's pit is fixed (no lift) – the control lets you test a pit cover or extension.").color(Color32::from_gray(160)));
            }
        } else if kind == OBJ_LIFT {
            heading(ui, "⬍ Stage lift");
            let s = self.state_mut();
            ui.add(egui::Slider::new(&mut s.stage_lift, -3.0..=2.5).text("Height").suffix(" m"));
            ui.horizontal_wrapped(|ui| {
                if ui.button("Sunk (-3 m)").clicked() {
                    s.stage_lift = -3.0;
                }
                if ui.button("Deck level").clicked() {
                    s.stage_lift = 0.0;
                }
                if ui.button("Raised 1.2 m").clicked() {
                    s.stage_lift = 1.2;
                }
            });
            ui.label(RichText::new("Pieces standing on the lift ride up and down with it.").color(Color32::from_gray(160)));
        } else {
            heading(ui, "Inspector");
            ui.label("Click anything in the 3D view or the outliner to edit it:");
            ui.label("• set pieces – sizes, doors open / shut, position, colour");
            ui.label("• fly bars, curtains and drops – fly in / out");
            ui.label("• lights – aim, colour, beam and level");
            ui.label("• revolve, pit and stage lifts");
            ui.add_space(6.0);
            ui.label(RichText::new("Add furniture from the 🛋 Pieces tab, or build your own with 🛠 Build Custom Piece.").color(Color32::from_gray(160)));
        }
    }

    fn piece_inspector(&mut self, ui: &mut Ui, idx: usize, now: f64) {
        let feet = self.feet;
        let v = self.project.venue.clone();
        let d = self.disp.clone();
        let battens = self.state().battens.clone();
        let warns: Vec<String> = self.warnings.iter().filter(|w| w.piece == idx).map(|w| w.text.clone()).collect();
        let (base, wrot) = piece_world(&self.state().pieces[idx], &v, &d, &battens);
        let mut action = 0u8;
        let mut revolve_toggle = None;
        let mut world_xz = None;
        {
            let p = &mut self.state_mut().pieces[idx];
            heading(ui, "Set piece");
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut p.name);
            });
            ui.label(RichText::new(&p.category).color(Color32::from_gray(150)));
            moving_parts_bar(ui, p);
            if p.template.is_some() {
                template_editor(ui, p, feet);
                egui::CollapsingHeader::new("Stretch overall").default_open(false).show(ui, |ui| {
                    ui.label(RichText::new("Squash or stretch the whole piece (the settings above keep their proportions).").size(11.0).color(Color32::from_gray(150)));
                    len_edit(ui, "Width", &mut p.size[0], feet, 0.02, 30.0);
                    len_edit(ui, "Depth", &mut p.size[2], feet, 0.005, 30.0);
                    len_edit(ui, "Height", &mut p.size[1], feet, 0.005, 20.0);
                    if ui.small_button("Remove stretch").clicked() {
                        p.size = p.base_size;
                    }
                });
            } else {
                egui::CollapsingHeader::new("Size").default_open(true).show(ui, |ui| {
                    len_edit(ui, "Width", &mut p.size[0], feet, 0.02, 30.0);
                    len_edit(ui, "Depth", &mut p.size[2], feet, 0.005, 30.0);
                    len_edit(ui, "Height", &mut p.size[1], feet, 0.005, 20.0);
                    if ui.small_button("Reset to original size").clicked() {
                        p.size = p.base_size;
                    }
                });
                hinge_list(ui, p);
            }
            egui::CollapsingHeader::new("Position & rotation").default_open(true).show(ui, |ui| {
                let mut x = base.x;
                let mut z = -base.z;
                let cx = len_edit(ui, "Across (+ = stage left)", &mut x, feet, -40.0, 40.0);
                let cz = len_edit(ui, "Upstage of setting line", &mut z, feet, -40.0, 40.0);
                if cx || cz {
                    world_xz = Some((x, -z));
                }
                len_edit(ui, "Raised above floor", &mut p.pos[1], feet, -3.0, 15.0);
                ui.horizontal(|ui| {
                    ui.label("Rotation");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(egui::DragValue::new(&mut p.rot).speed(1.0).suffix("°").fixed_decimals(0)).changed() {
                            p.rot = deg_wrap(p.rot);
                        }
                    });
                });
                ui.horizontal(|ui| {
                    for dd in [-15.0f32, -1.0, 1.0, 15.0, 90.0] {
                        if ui.small_button(format!("{:+}°", dd)).clicked() {
                            p.rot = deg_wrap(p.rot + dd);
                        }
                    }
                });
                if p.on_revolve {
                    ui.label(RichText::new(format!("On the revolve: currently facing {:.0}°", deg_wrap(wrot))).size(11.0).color(Color32::from_gray(150)));
                }
            });
            egui::CollapsingHeader::new("Look").default_open(true).show(ui, |ui| {
                egui::ComboBox::from_label("Material").selected_text(p.material.label()).show_ui(ui, |ui| {
                    for m in Material::ALL {
                        if ui.selectable_value(&mut p.material, m, m.label()).clicked() {
                            p.color = m.default_color();
                        }
                    }
                });
                color_edit(ui, "Main colour", &mut p.color);
            });
            egui::CollapsingHeader::new("Staging").default_open(true).show(ui, |ui| {
                if v.has_revolve {
                    let mut on = p.on_revolve;
                    if ui.checkbox(&mut on, "Rides the revolve").changed() {
                        revolve_toggle = Some(on);
                    }
                }
                let cur = p.flown_on.and_then(|i| battens.get(i)).map(|b| b.name.clone()).unwrap_or_else(|| "Stands on the floor".into());
                egui::ComboBox::from_label("Hang from").selected_text(cur).show_ui(ui, |ui| {
                    ui.selectable_value(&mut p.flown_on, None, "Stands on the floor");
                    for (i, b) in battens.iter().enumerate() {
                        if matches!(b.kind, BattenKind::Scenery | BattenKind::Drop | BattenKind::Border) {
                            ui.selectable_value(&mut p.flown_on, Some(i), format!("{} (flies)", b.name));
                        }
                    }
                });
                ui.checkbox(&mut p.locked, "Lock (can't be dragged)");
                ui.checkbox(&mut p.visible, "Visible in this scene");
            });
            ui.horizontal_wrapped(|ui| {
                if ui.button("⧉ Duplicate").clicked() {
                    action = 1;
                }
                if ui.button("🗑 Delete").clicked() {
                    action = 2;
                }
                if ui.button("🛠 Edit parts").clicked() {
                    action = 3;
                }
                if ui.button("⭐ Save to my library").clicked() {
                    action = 4;
                }
            });
        }
        if let Some((x, z)) = world_xz {
            self.set_piece_world_xz(idx, x, z);
        }
        if let Some(on) = revolve_toggle {
            self.set_on_revolve(idx, on);
        }
        ui.add_space(4.0);
        if !warns.is_empty() {
            for w in &warns {
                ui.label(RichText::new(format!("⚠ {w}")).color(Color32::from_rgb(255, 130, 110)));
            }
        } else {
            ui.label(RichText::new("✔ Fits: no clashes with curtains, sightlines or the stage edge").color(Color32::from_rgb(120, 200, 120)));
        }
        match action {
            1 => self.duplicate_piece(idx),
            2 => self.delete_selected(),
            3 => {
                let p = self.state().pieces[idx].clone();
                let item = LibItem { name: p.name.clone(), category: "Custom".into(), material: p.material, color: p.color, parts: scaled_parts(&p), template: None };
                self.builder = Some(Builder { item, editing_piece: Some(p.id), editing_lib: None, preview_on_stage: false });
            }
            4 => {
                let p = self.state().pieces[idx].clone();
                // Keep the settings (so the saved piece stays adjustable) unless it was stretched.
                let unstretched = (0..3).all(|i| (p.size[i] - p.base_size[i]).abs() < 1e-3);
                let template = if unstretched { p.template.clone() } else { None };
                self.project.custom_library.push(LibItem { name: p.name.clone(), category: "Custom".into(), material: p.material, color: p.color, parts: scaled_parts(&p), template });
                self.notify(format!("Saved '{}' to My custom pieces", p.name), now);
            }
            _ => {}
        }
    }

    fn batten_inspector(&mut self, ui: &mut Ui, idx: usize) {
        let feet = self.feet;
        let v = self.project.venue.clone();
        let max = v.max_trim();
        let mut remove = false;
        {
            let b = &mut self.state_mut().battens[idx];
            heading(ui, "Fly line / soft goods");
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut b.name);
            });
            egui::ComboBox::from_label("Type").selected_text(b.kind.label()).show_ui(ui, |ui| {
                for k in BattenKind::ALL {
                    ui.selectable_value(&mut b.kind, k, k.label());
                }
            });
            ui.add_space(4.0);
            ui.label(RichText::new(format!("Height (trim): {}", fmt_len(b.height, feet))).size(16.0).strong());
            ui.add(egui::Slider::new(&mut b.height, 0.3..=max).show_value(false));
            ui.horizontal_wrapped(|ui| {
                if ui.button("⬇ Fly in").on_hover_text("Lower until the goods reach the deck").clicked() {
                    b.height = b.in_trim().min(max);
                }
                if ui.button("⬆ Fly out").on_hover_text("Raise to the top").clicked() {
                    b.height = max;
                }
                if ui.button("Masking trim").on_hover_text("Bottom of the goods just above the opening").clicked() {
                    b.height = (v.pros_height + b.goods_h.min(2.0) - 0.2).min(max);
                }
            });
            if !v.has_fly_tower() {
                ui.label(RichText::new(format!("No fly tower here: pipes are dead-hung and top out at {}. Drops stay in view.", fmt_len(max, feet))).color(Color32::from_rgb(255, 190, 110)));
            }
            ui.separator();
            let mut up = b.dist;
            if len_edit(ui, "Upstage of setting line", &mut up, feet, 0.1, v.stage_depth - 0.1) {
                b.dist = up;
            }
            if !matches!(b.kind, BattenKind::Lighting | BattenKind::Scenery) {
                len_edit(ui, "Goods height (drop/legs)", &mut b.goods_h, feet, 0.2, 20.0);
                color_edit(ui, "Fabric / paint colour", &mut b.color);
            }
            if matches!(b.kind, BattenKind::HouseCurtain | BattenKind::Tabs) {
                ui.add(egui::Slider::new(&mut b.open, 0.0..=1.0).text("Open"));
                ui.horizontal(|ui| {
                    if ui.button("Open").clicked() {
                        b.open = 1.0;
                    }
                    if ui.button("Close").clicked() {
                        b.open = 0.0;
                    }
                });
            }
            ui.checkbox(&mut b.visible, "Hung in this scene");
            if ui.button("🗑 Remove this line").clicked() {
                remove = true;
            }
        }
        if remove {
            self.remove_batten(idx);
            self.sel = 0;
            return;
        }
        ui.add_space(4.0);
        if ui.button("+ Add a new fly line").clicked() {
            let s = self.state_mut();
            let d = s.battens.get(idx).map(|b| b.dist + 0.3).unwrap_or(1.0);
            let n = s.battens.len() + 1;
            s.battens.push(Batten { name: format!("Line {n}"), kind: BattenKind::Scenery, dist: d, height: max, goods_h: 0.0, color: [80, 80, 80], open: 1.0, visible: true, opening: 0.0, border_h: 0.0 });
            self.sel = OBJ_BATTEN + (self.state().battens.len() - 1) as u32;
        }
    }

    fn light_inspector(&mut self, ui: &mut Ui, idx: usize) {
        let feet = self.feet;
        let pieces: Vec<(String, [f32; 3])> = {
            let v = self.project.venue.clone();
            let d = self.disp.clone();
            let b = self.state().battens.clone();
            self.state()
                .pieces
                .iter()
                .map(|p| {
                    let (base, _) = piece_world(p, &v, &d, &b);
                    (p.name.clone(), [base.x, base.y + p.size[1] * 0.5, base.z])
                })
                .collect()
        };
        let battens = self.state().battens.clone();
        let mut remove = false;
        let mut dup = false;
        let mut aim = false;
        {
            let l = &mut self.state_mut().lights[idx];
            heading(ui, "Light");
            ui.horizontal(|ui| {
                ui.label("Name");
                ui.text_edit_singleline(&mut l.name);
            });
            egui::ComboBox::from_label("Fixture").selected_text(l.kind.label()).show_ui(ui, |ui| {
                for k in LightKind::ALL {
                    if ui.selectable_value(&mut l.kind, k, k.label()).clicked() {
                        l.beam_deg = k.default_beam();
                    }
                }
            });
            ui.add(egui::Slider::new(&mut l.intensity, 0.0..=1.0).text("Level"));
            ui.add(egui::Slider::new(&mut l.beam_deg, 5.0..=90.0).text("Beam").suffix("°"));
            ui.horizontal(|ui| {
                ui.label("Colour");
                let mut c = l.color;
                if ui.color_edit_button_rgb(&mut c).changed() {
                    l.color = c;
                }
            });
            ui.horizontal_wrapped(|ui| {
                for (name, c) in GELS {
                    let col = Color32::from_rgb((c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8);
                    if ui.add(egui::Button::new(RichText::new("●").color(col).size(18.0)).frame(false)).on_hover_text(name).clicked() {
                        l.color = c;
                    }
                }
            });
            ui.separator();
            match &mut l.mount {
                LightMount::Batten { index, x } => {
                    let n = battens.get(*index).map(|b| b.name.clone()).unwrap_or_default();
                    ui.label(format!("Hung on: {n}"));
                    len_edit(ui, "Position along bar", x, feet, -20.0, 20.0);
                }
                LightMount::Fixed { pos, position_name } => {
                    ui.label(format!("Position: {position_name}"));
                    len_edit(ui, "Across", &mut pos[0], feet, -30.0, 30.0);
                    len_edit(ui, "Height", &mut pos[1], feet, -2.0, 30.0);
                    let mut up = -pos[2];
                    if len_edit(ui, "Upstage", &mut up, feet, -40.0, 40.0) {
                        pos[2] = -up;
                    }
                }
            }
            ui.label(RichText::new("Aim point").strong());
            len_edit(ui, "Aim across", &mut l.target[0], feet, -30.0, 30.0);
            len_edit(ui, "Aim height", &mut l.target[1], feet, -3.0, 20.0);
            let mut up = -l.target[2];
            if len_edit(ui, "Aim upstage", &mut up, feet, -40.0, 40.0) {
                l.target[2] = -up;
            }
            ui.horizontal_wrapped(|ui| {
                if ui.button("🎯 Click stage to aim").clicked() {
                    aim = true;
                }
                if !pieces.is_empty() {
                    ui.menu_button("Aim at piece ▸", |ui| {
                        for (n, p) in &pieces {
                            if ui.button(n).clicked() {
                                l.target = *p;
                                ui.close();
                            }
                        }
                    });
                }
            });
            ui.label(RichText::new("Tip: drag the light in the 3D view to sweep its aim across the stage.").size(11.0).color(Color32::from_gray(150)));
            ui.horizontal(|ui| {
                if ui.button("⧉ Duplicate").clicked() {
                    dup = true;
                }
                if ui.button("🗑 Delete").clicked() {
                    remove = true;
                }
            });
        }
        if aim {
            self.aim_mode = true;
        }
        if dup {
            let mut l = self.state().lights[idx].clone();
            l.name = format!("{} copy", l.name);
            match &mut l.mount {
                LightMount::Batten { x, .. } => *x += 0.6,
                LightMount::Fixed { pos, .. } => pos[0] += 0.6,
            }
            self.state_mut().lights.push(l);
            self.sel = OBJ_LIGHT + (self.state().lights.len() - 1) as u32;
        }
        if remove {
            self.delete_selected();
        }
    }
}
