//! Toolbar, status bar and the left-hand tabs (venue, pieces, lights, scenes).

use crate::app::*;
use crate::library;
use crate::model::*;
use crate::scene::*;
use crate::venues;
use egui::{Color32, RichText, Ui};

pub const ACCENT: Color32 = Color32::from_rgb(232, 170, 70);

pub fn heading(ui: &mut Ui, t: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(t).strong().size(15.0).color(ACCENT));
    ui.add_space(2.0);
}

/// Edit a length stored in metres, shown in metres or feet.
pub fn len_edit(ui: &mut Ui, label: &str, m: &mut f32, feet: bool, lo: f32, hi: f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if feet {
                let mut ft = *m / 0.3048;
                let r = ui.add(egui::DragValue::new(&mut ft).speed(0.05).range(lo / 0.3048..=hi / 0.3048).suffix(" ft").fixed_decimals(2));
                if r.changed() {
                    *m = ft * 0.3048;
                    changed = true;
                }
            } else if ui.add(egui::DragValue::new(m).speed(0.01).range(lo..=hi).suffix(" m").fixed_decimals(2)).changed() {
                changed = true;
            }
        });
    });
    changed
}

pub fn deg_edit(ui: &mut Ui, label: &str, d: &mut f32, lo: f32, hi: f32) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            changed = ui.add(egui::DragValue::new(d).speed(0.5).range(lo..=hi).suffix("°").fixed_decimals(1)).changed();
        });
    });
    changed
}

pub fn color_edit(ui: &mut Ui, label: &str, c: &mut [u8; 3]) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(label);
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let mut col = Color32::from_rgb(c[0], c[1], c[2]);
            if ui.color_edit_button_srgba(&mut col).changed() {
                *c = [col.r(), col.g(), col.b()];
                changed = true;
            }
        });
    });
    changed
}

impl App {
    // ------------------------------------------------------------------ top bar
    pub fn top_bar(&mut self, ui: &mut Ui, now: f64) {
        let ctx = ui.ctx().clone();
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if let Some(logo) = &self.logo {
                ui.add(egui::Image::new(logo).fit_to_exact_size(egui::vec2(26.0, 26.0)));
            }
            ui.label(RichText::new("Stagecraft 3D").strong().size(18.0).color(ACCENT));
            ui.separator();
            ui.menu_button("📁 File", |ui| {
                if ui.button("✨ New project (keep venue)").clicked() {
                    let v = self.project.venue.clone();
                    self.project = crate::sample::empty_project(v);
                    self.sel = 0;
                    self.snap_display();
                    ui.close();
                }
                if ui.button("🎭 Load sample production").clicked() {
                    let v = self.project.venue.clone();
                    self.project = crate::sample::sample_project(v);
                    self.sel = 0;
                    self.snap_display();
                    ui.close();
                }
                ui.separator();
                if ui.button("📂 Open project (.json)…").clicked() {
                    ui.close();
                    self.open_json(now);
                }
                if ui.button("💾 Save project (.json)…").clicked() {
                    ui.close();
                    self.save_json(now);
                }
                if ui.button("📷 Export PNG snapshot of this view…").clicked() {
                    ui.close();
                    self.export_png(&ctx);
                }
            });
            ui.add(egui::TextEdit::singleline(&mut self.project.name).desired_width(170.0).hint_text("Production name"));
            ui.separator();
            ui.label("View:");
            let aspect = self.view_aspect().max(0.3);
            for p in CamPreset::ALL {
                if ui.selectable_label(self.cam_preset == p, p.label()).clicked() {
                    self.apply_cam_preset(p, true, aspect);
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            if ui.selectable_label(self.work_mode, "💡 Work Light").on_hover_text("Even, bright light for building the set").clicked() {
                self.work_mode = true;
            }
            if ui.selectable_label(!self.work_mode, "🌙 Cue Lighting").on_hover_text("Preview the lighting rig and mood").clicked() {
                self.work_mode = false;
            }
            ui.separator();
            ui.toggle_value(&mut self.show_beams, "🔦 Beams");
            ui.toggle_value(&mut self.show_rigging, "🪢 Rigging");
            ui.toggle_value(&mut self.show_labels, "🏷 Labels");
            ui.toggle_value(&mut self.show_grid, "▦ Grid");
            ui.toggle_value(&mut self.show_sightlines, "👁 Sightlines");
            ui.toggle_value(&mut self.show_warnings, "⚠ Warnings");
            ui.separator();
            ui.toggle_value(&mut self.snap, "🧲 Snap");
            let snap_txt = if self.feet { format!("{:.0} in", self.snap_step / 0.0254) } else { format!("{:.2} m", self.snap_step) };
            egui::ComboBox::from_id_salt("snapstep").width(70.0).selected_text(snap_txt).show_ui(ui, |ui| {
                let opts: Vec<(f32, &str)> = if self.feet {
                    vec![(0.0762, "3 in"), (0.1524, "6 in"), (0.3048, "1 ft"), (0.6096, "2 ft")]
                } else {
                    vec![(0.05, "0.05 m"), (0.1, "0.10 m"), (0.25, "0.25 m"), (0.5, "0.50 m"), (1.0, "1.00 m")]
                };
                for (v, l) in opts {
                    ui.selectable_value(&mut self.snap_step, v, l);
                }
            });
            egui::ComboBox::from_id_salt("rotstep").width(60.0).selected_text(format!("{:.0}° turn", self.rot_step)).show_ui(ui, |ui| {
                for v in [1.0f32, 5.0, 15.0, 45.0, 90.0] {
                    ui.selectable_value(&mut self.rot_step, v, format!("{v:.0}°"));
                }
            });
            ui.separator();
            if ui.selectable_label(!self.feet, "m").clicked() {
                self.feet = false;
                self.snap_step = 0.25;
            }
            if ui.selectable_label(self.feet, "ft").clicked() {
                self.feet = true;
                self.snap_step = 0.3048;
            }
            ui.separator();
            let q = if self.render_scale < 0.6 {
                "Draft"
            } else if self.render_scale < 0.9 {
                "Normal"
            } else {
                "Sharp"
            };
            egui::ComboBox::from_id_salt("quality").width(80.0).selected_text(q).show_ui(ui, |ui| {
                ui.selectable_value(&mut self.render_scale, 0.5, "Draft (fastest)");
                ui.selectable_value(&mut self.render_scale, 0.75, "Normal");
                ui.selectable_value(&mut self.render_scale, 1.0, "Sharp");
            });
        });
        ui.add_space(2.0);
    }

    fn view_aspect(&self) -> f32 {
        let r = self.view_rect();
        r.width() / r.height().max(1.0)
    }

    // ------------------------------------------------------------------ status bar
    pub fn status_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let v = &self.project.venue;
            ui.label(
                RichText::new(format!(
                    "{} · opening {} × {} · depth {} · {}",
                    v.name,
                    fmt_len(v.pros_width, self.feet),
                    fmt_len(v.pros_height, self.feet),
                    fmt_len(v.stage_depth, self.feet),
                    if v.has_fly_tower() { format!("grid {}", fmt_len(v.grid_height, self.feet)) } else { "no fly tower".into() }
                ))
                .size(12.0),
            );
            ui.separator();
            let n = self.warnings.len();
            let txt = if n == 0 {
                RichText::new("✔ No set warnings").color(Color32::from_rgb(120, 200, 120))
            } else {
                RichText::new(format!("⚠ {n} set warning{}", if n == 1 { "" } else { "s" })).color(Color32::from_rgb(255, 120, 100))
            };
            if ui.selectable_label(self.show_warn_list, txt).clicked() {
                self.show_warn_list = !self.show_warn_list;
            }
            ui.separator();
            ui.label(
                RichText::new("Drag: orbit · Right-drag: pan · Scroll: zoom · Drag a piece to move (Shift = height) · Arrows nudge · Q/E rotate · Del delete · Ctrl+D duplicate · F focus")
                    .size(11.5)
                    .color(Color32::from_gray(150)),
            );
        });
    }

    // ------------------------------------------------------------------ left panel
    pub fn left_panel(&mut self, ui: &mut Ui, now: f64) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            for (t, l) in [(LeftTab::Venue, "🏛 Venue"), (LeftTab::Library, "🛋 Pieces"), (LeftTab::Lighting, "💡 Lights"), (LeftTab::Scenes, "🎬 Scenes")] {
                if ui.selectable_label(self.tab == t, RichText::new(l).size(13.5)).clicked() {
                    self.tab = t;
                }
            }
        });
        ui.separator();
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match self.tab {
            LeftTab::Venue => self.venue_tab(ui, now),
            LeftTab::Library => self.library_tab(ui, now),
            LeftTab::Lighting => self.lighting_tab(ui, now),
            LeftTab::Scenes => self.scenes_tab(ui, now),
        });
    }

    fn venue_tab(&mut self, ui: &mut Ui, now: f64) {
        heading(ui, "Stage preset");
        let cur_key = self.project.venue.key.clone();
        let mut pick: Option<Venue> = None;
        for v in &self.venues {
            let sel = cur_key == v.key || cur_key == format!("{}_edited", v.key);
            if ui.selectable_label(sel, RichText::new(&v.name).strong()).clicked() {
                pick = Some(v.clone());
            }
        }
        if let Some(v) = pick {
            self.set_venue(v, now);
        }
        ui.separator();
        let feet = self.feet;
        let v = &mut self.project.venue;
        ui.label(RichText::new(&v.location).italics().color(Color32::from_gray(170)));
        ui.label(&v.notes);
        if !v.tech_notes.is_empty() {
            egui::CollapsingHeader::new("Lighting & sound equipment").default_open(false).show(ui, |ui| {
                ui.label(RichText::new(&v.tech_notes).size(12.0));
            });
        }
        ui.label(RichText::new(format!("Source: {}", v.confidence)).size(11.0).color(Color32::from_rgb(150, 170, 200)));
        ui.add_space(4.0);
        let mut changed = false;
        egui::CollapsingHeader::new("Stage & proscenium").default_open(true).show(ui, |ui| {
            changed |= len_edit(ui, "Opening width", &mut v.pros_width, feet, 2.0, 30.0);
            changed |= len_edit(ui, "Opening height", &mut v.pros_height, feet, 2.0, 15.0);
            changed |= len_edit(ui, "Stage depth (to back wall)", &mut v.stage_depth, feet, 2.0, 30.0);
            changed |= len_edit(ui, "Wing width (each side)", &mut v.wing_width, feet, 0.0, 15.0);
            changed |= len_edit(ui, "Apron / forestage", &mut v.apron_depth, feet, 0.0, 6.0);
            changed |= deg_edit(ui, "Stage rake", &mut v.rake_deg, 0.0, 6.0);
            changed |= len_edit(ui, "Stage height above house", &mut v.stage_height, feet, 0.0, 2.0);
            ui.horizontal(|ui| {
                ui.label("House curtain");
                changed |= ui.selectable_value(&mut v.curtain_style, CurtainStyle::Traveller, "Draws apart").changed();
                changed |= ui.selectable_value(&mut v.curtain_style, CurtainStyle::Guillotine, "Flies out").changed();
            });
            changed |= ui.checkbox(&mut v.has_crossover, "Backstage crossover").changed();
            changed |= ui.checkbox(&mut v.has_house_steps, "Stage-to-house steps").changed();
        });
        egui::CollapsingHeader::new("Flying / fly loft").default_open(true).show(ui, |ui| {
            let mut fly = v.has_fly_tower();
            if ui.checkbox(&mut fly, "Fly tower (scenery can fly out of sight)").changed() {
                v.grid_height = if fly { (v.pros_height * 2.2).max(8.0) } else { 0.0 };
                changed = true;
            }
            if v.has_fly_tower() {
                changed |= len_edit(ui, "Grid height", &mut v.grid_height, feet, 4.0, 40.0);
                changed |= len_edit(ui, "Fly gallery height", &mut v.fly_gallery_height, feet, 0.0, 30.0);
            } else {
                changed |= len_edit(ui, "Highest pipe (dead-hung)", &mut v.pipe_height, feet, 2.0, 12.0);
            }
            ui.horizontal(|ui| {
                ui.label("System");
                ui.text_edit_singleline(&mut v.fly_system);
            });
        });
        egui::CollapsingHeader::new("Machinery").default_open(true).show(ui, |ui| {
            changed |= ui.checkbox(&mut v.has_pit, "Orchestra pit").changed();
            if v.has_pit {
                changed |= ui.checkbox(&mut v.pit_is_lift, "Pit is a lift").changed();
                changed |= len_edit(ui, "Pit width", &mut v.pit_width, feet, 2.0, 25.0);
                changed |= len_edit(ui, "Pit depth (front to back)", &mut v.pit_depth, feet, 1.0, 8.0);
            }
            changed |= ui.checkbox(&mut v.has_revolve, "Revolve").changed();
            if v.has_revolve {
                changed |= len_edit(ui, "Revolve diameter", &mut v.revolve_diameter, feet, 2.0, 20.0);
                let mut up = -v.revolve_z;
                if len_edit(ui, "Revolve centre upstage", &mut up, feet, 0.0, 30.0) {
                    v.revolve_z = -up;
                    changed = true;
                }
            }
            changed |= ui.checkbox(&mut v.has_stage_lift, "Stage lift / trap").changed();
            if v.has_stage_lift {
                changed |= len_edit(ui, "Lift width", &mut v.lift_width, feet, 0.5, 12.0);
                changed |= len_edit(ui, "Lift depth", &mut v.lift_depth, feet, 0.5, 8.0);
                changed |= len_edit(ui, "Lift centre (across)", &mut v.lift_x, feet, -10.0, 10.0);
                let mut up = -v.lift_z;
                if len_edit(ui, "Lift centre upstage", &mut up, feet, 0.0, 30.0) {
                    v.lift_z = -up;
                    changed = true;
                }
            }
        });
        egui::CollapsingHeader::new("Auditorium").default_open(false).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Seats");
                changed |= ui.add(egui::DragValue::new(&mut v.capacity).range(20..=3000)).changed();
            });
            changed |= len_edit(ui, "House width", &mut v.house_width, feet, 5.0, 50.0);
            changed |= len_edit(ui, "Rise per row (rake)", &mut v.house_rise_per_row, feet, 0.0, 0.5);
            ui.horizontal(|ui| {
                ui.label("Venue type");
                changed |= ui.selectable_value(&mut v.kind, VenueKind::Proscenium, "Proscenium").changed();
                changed |= ui.selectable_value(&mut v.kind, VenueKind::BlackBox, "Black box").changed();
            });
            ui.horizontal(|ui| {
                ui.label("Room look");
                let cur = if v.dark_room && v.look == RoomLook::Classic { RoomLook::Studio } else { v.look };
                egui::ComboBox::from_id_salt("room_look").selected_text(cur.label()).show_ui(ui, |ui| {
                    for l in RoomLook::ALL {
                        if ui.selectable_label(cur == l, l.label()).clicked() {
                            v.look = l;
                            v.dark_room = l == RoomLook::Studio;
                            changed = true;
                        }
                    }
                });
            });
            changed |= ui.checkbox(&mut v.side_galleries, "Side galleries").changed();
            changed |= ui.checkbox(&mut v.rear_gallery, "Rear gallery / balcony").changed();
            changed |= ui.checkbox(&mut v.aisles_both_sides, "Aisle stairs on both sides").changed();
            ui.horizontal(|ui| {
                ui.label("Cross aisle after row");
                changed |= ui.add(egui::DragValue::new(&mut v.cross_aisle_after).range(0..=40)).changed();
                ui.label(RichText::new("(0 = none)").size(11.0).color(Color32::from_gray(150)));
            });
            changed |= ui.checkbox(&mut v.side_screens, "Screens beside the stage").changed();
        });
        if changed {
            if !(v.key.ends_with("_edited") || v.key == "custom") {
                v.key = format!("{}_edited", v.key);
            }
            v.revolve_diameter = v.revolve_diameter.min(v.stage_width()).min(v.stage_depth);
            v.revolve_z = v.revolve_z.clamp(-v.stage_depth + v.revolve_diameter * 0.5, -v.revolve_diameter * 0.5);
            v.lift_z = v.lift_z.clamp(-v.stage_depth + v.lift_depth * 0.5, -v.lift_depth * 0.5);
            let max = v.max_trim();
            let depth = v.stage_depth;
            let sh = v.stage_height;
            for sc in self.project.scenes.iter_mut() {
                for b in sc.state.battens.iter_mut() {
                    b.height = b.height.min(max);
                    b.dist = b.dist.min(depth - 0.2);
                }
                sc.state.pit_lift = sc.state.pit_lift.max(-(sh + 2.5));
            }
        }
        ui.add_space(6.0);
        if ui.button("🔁 Rebuild rigging & lights for these dimensions").clicked() {
            let v = self.project.venue.clone();
            self.set_venue(v, now);
        }
    }

    fn library_tab(&mut self, ui: &mut Ui, now: f64) {
        if ui.add_sized([ui.available_width(), 34.0], egui::Button::new(RichText::new("🛠 Build Custom Piece…").size(15.0).strong())).clicked() {
            self.open_builder(None);
        }
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("🔍");
            ui.add(egui::TextEdit::singleline(&mut self.lib_filter).hint_text("Search pieces").desired_width(f32::INFINITY));
        });
        ui.horizontal_wrapped(|ui| {
            if ui.selectable_label(self.lib_cat == 0, "All").clicked() {
                self.lib_cat = 0;
            }
            for (i, c) in library::CATEGORIES.iter().enumerate() {
                if ui.selectable_label(self.lib_cat == i + 1, *c).clicked() {
                    self.lib_cat = i + 1;
                }
            }
        });
        ui.separator();
        let filter = self.lib_filter.to_lowercase();
        let cat = if self.lib_cat == 0 { None } else { Some(library::CATEGORIES[self.lib_cat - 1]) };
        let mut add: Option<LibItem> = None;
        let mut last_cat = String::new();
        for item in &self.library {
            if let Some(c) = cat {
                if item.category != c {
                    continue;
                }
            }
            if !filter.is_empty() && !item.name.to_lowercase().contains(&filter) {
                continue;
            }
            if item.category != last_cat {
                last_cat = item.category.clone();
                ui.label(RichText::new(&last_cat).strong().color(Color32::from_gray(190)));
            }
            let (lo, hi) = item.bounds();
            let dims = format!("{} × {} × {}", fmt_len(hi[0] - lo[0], self.feet), fmt_len(hi[2] - lo[2], self.feet), fmt_len(hi[1] - lo[1], self.feet));
            ui.horizontal(|ui| {
                let c = item.color;
                let (r, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 3.0, Color32::from_rgb(c[0], c[1], c[2]));
                if ui.button(format!("+ {}", item.name)).on_hover_text(format!("Add to stage · W×D×H {dims}")).clicked() {
                    add = Some(item.clone());
                }
            });
        }
        if let Some(it) = add {
            self.add_piece_from(&it);
            self.notify(format!("Added {} – drag it into place", it.name), now);
        }
        ui.separator();
        heading(ui, "My custom pieces");
        if self.project.custom_library.is_empty() {
            ui.label(RichText::new("Pieces you save from the builder appear here and are stored in the project file.").color(Color32::from_gray(150)));
        }
        let mut action: Option<(usize, u8)> = None;
        for (i, item) in self.project.custom_library.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut item.name).desired_width(120.0));
                if ui.small_button("+ Add").clicked() {
                    action = Some((i, 0));
                }
                if ui.small_button("✏").on_hover_text("Edit").clicked() {
                    action = Some((i, 1));
                }
                if ui.small_button("⧉").on_hover_text("Duplicate").clicked() {
                    action = Some((i, 2));
                }
                if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                    action = Some((i, 3));
                }
            });
        }
        if let Some((i, a)) = action {
            match a {
                0 => {
                    let it = self.project.custom_library[i].clone();
                    self.add_piece_from(&it);
                }
                1 => {
                    let it = self.project.custom_library[i].clone();
                    self.builder = Some(Builder { item: it, editing_piece: None, editing_lib: Some(i), preview_on_stage: true });
                }
                2 => {
                    let mut it = self.project.custom_library[i].clone();
                    it.name = format!("{} copy", it.name);
                    self.project.custom_library.push(it);
                }
                _ => {
                    self.project.custom_library.remove(i);
                }
            }
        }
    }

    fn lighting_tab(&mut self, ui: &mut Ui, now: f64) {
        heading(ui, "Lighting mode");
        ui.horizontal(|ui| {
            if ui.selectable_label(self.work_mode, "💡 Work / house light").clicked() {
                self.work_mode = true;
            }
            if ui.selectable_label(!self.work_mode, "🌙 Cue lighting").clicked() {
                self.work_mode = false;
            }
        });
        ui.checkbox(&mut self.show_beams, "Show beam cones");
        {
            let s = self.state_mut();
            ui.add(egui::Slider::new(&mut s.house_lights, 0.0..=1.0).text("House lights (cue mode)"));
        }
        heading(ui, "Quick looks for this scene");
        ui.horizontal_wrapped(|ui| {
            for look in ["Warm interior", "Bright day", "Sunset", "Moonlight", "Dramatic side", "Blackout"] {
                if ui.button(look).clicked() {
                    crate::sample::apply_look(self.state_mut(), look);
                    self.work_mode = false;
                    self.notify(format!("Look: {look} (Cue lighting on)"), now);
                }
            }
        });
        heading(ui, "Rig");
        let lx: Vec<(usize, String)> = self.state().battens.iter().enumerate().filter(|(_, b)| b.kind == BattenKind::Lighting).map(|(i, b)| (i, b.name.clone())).collect();
        let pw = self.project.venue.pros_width * 0.5;
        let depth = self.project.venue.stage_depth;
        let foh_z = self.project.venue.house_start_z() + 3.5;
        let ph = self.project.venue.pros_height;
        let mut add: Option<Light> = None;
        let dists: Vec<f32> = self.state().battens.iter().map(|b| b.dist).collect();
        ui.horizontal_wrapped(|ui| {
            ui.menu_button("+ Light on a bar", |ui| {
                for (i, n) in &lx {
                    if ui.button(n).clicked() {
                        let d = dists[*i];
                        add = Some(Light {
                            name: "Light".into(),
                            kind: LightKind::Fresnel,
                            mount: LightMount::Batten { index: *i, x: 0.0 },
                            target: [0.0, 0.0, -d + 0.5],
                            beam_deg: 40.0,
                            intensity: 0.8,
                            color: [1.0, 0.9, 0.75],
                        });
                        ui.close();
                    }
                }
            });
            let new_fixed = |name: &str, pos: [f32; 3], target: [f32; 3], posname: &str, kind: LightKind| Light {
                name: name.into(),
                kind,
                mount: LightMount::Fixed { pos, position_name: posname.into() },
                target,
                beam_deg: kind.default_beam(),
                intensity: 0.8,
                color: [1.0, 0.95, 0.85],
            };
            if ui.button("+ SL boom").clicked() {
                add = Some(new_fixed("SL boom", [pw + 0.9, 1.8, -depth * 0.4], [0.0, 1.0, -depth * 0.4], "SL boom", LightKind::Par));
            }
            if ui.button("+ SR boom").clicked() {
                add = Some(new_fixed("SR boom", [-pw - 0.9, 1.8, -depth * 0.4], [0.0, 1.0, -depth * 0.4], "SR boom", LightKind::Par));
            }
            if ui.button("+ FOH").clicked() {
                add = Some(new_fixed("FOH", [0.0, ph + 1.0, foh_z], [0.0, 1.2, -1.0], "FOH bridge", LightKind::Profile));
            }
        });
        if let Some(mut l) = add {
            l.name = format!("{} {}", l.name, self.state().lights.len() + 1);
            self.state_mut().lights.push(l);
            self.sel = OBJ_LIGHT + (self.state().lights.len() - 1) as u32;
        }
        ui.add_space(4.0);
        let mut select = None;
        let sel = self.sel;
        for (i, l) in self.state_mut().lights.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                let c = Color32::from_rgb((l.color[0] * 255.0) as u8, (l.color[1] * 255.0) as u8, (l.color[2] * 255.0) as u8);
                let (r, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
                ui.painter().circle_filled(r.center(), 6.0, c);
                if ui.selectable_label(sel == OBJ_LIGHT + i as u32, &l.name).clicked() {
                    select = Some(i);
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(egui::Slider::new(&mut l.intensity, 0.0..=1.0).show_value(false));
                });
            });
        }
        if let Some(i) = select {
            self.sel = OBJ_LIGHT + i as u32;
        }
    }

    fn scenes_tab(&mut self, ui: &mut Ui, now: f64) {
        heading(ui, "Scenes / act cues");
        ui.label(RichText::new("Each scene remembers its pieces, fly heights, revolve, lifts and lighting. Click a number to play the change.").color(Color32::from_gray(160)));
        ui.checkbox(&mut self.animate, "Animate scene changes (fly cues, revolve, lifts)");
        ui.add_space(4.0);
        let mut goto = None;
        let mut action: Option<(usize, u8)> = None;
        let cur = self.project.current;
        let n = self.project.scenes.len();
        for (i, sc) in self.project.scenes.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                if ui.selectable_label(i == cur, RichText::new(format!(" {} ", i + 1)).strong()).clicked() {
                    goto = Some(i);
                }
                let r = ui.add(egui::TextEdit::singleline(&mut sc.name).desired_width((ui.available_width() - 80.0).max(80.0)));
                if r.gained_focus() {
                    goto = Some(i);
                }
                if ui.small_button("▲").clicked() && i > 0 {
                    action = Some((i, 0));
                }
                if ui.small_button("▼").clicked() && i + 1 < n {
                    action = Some((i, 1));
                }
                if ui.small_button("🗑").clicked() && n > 1 {
                    action = Some((i, 2));
                }
            });
        }
        if let Some(i) = goto {
            if i != cur {
                self.go_to_scene(i);
            }
        }
        if let Some((i, a)) = action {
            let c = self.project.current;
            match a {
                0 => {
                    self.project.scenes.swap(i, i - 1);
                    self.project.current = if c == i { i - 1 } else if c == i - 1 { i } else { c };
                }
                1 => {
                    self.project.scenes.swap(i, i + 1);
                    self.project.current = if c == i { i + 1 } else if c == i + 1 { i } else { c };
                }
                _ => {
                    self.project.scenes.remove(i);
                    if c >= self.project.scenes.len() {
                        self.project.current = self.project.scenes.len() - 1;
                    } else if c > i {
                        self.project.current = c - 1;
                    }
                    self.snap_display();
                }
            }
        }
        if ui.button("+ New scene (copy of this one)").clicked() {
            let mut sc = self.project.scenes[self.project.current].clone();
            sc.name = format!("Act 1 Scene {}", self.project.scenes.len() + 1);
            self.project.scenes.insert(self.project.current + 1, sc);
            self.project.current += 1;
            self.notify("New scene created – move pieces and fly cues for it", now);
        }
        if ui.button("+ Empty scene").clicked() {
            let st = venues::fresh_state(&self.project.venue);
            let n = self.project.scenes.len() + 1;
            self.project.scenes.push(SceneState { name: format!("Scene {n}"), state: st });
            let i = self.project.scenes.len() - 1;
            self.go_to_scene(i);
        }
        ui.horizontal(|ui| {
            if ui.button("⏮ Previous").clicked() && self.project.current > 0 {
                let i = self.project.current - 1;
                self.go_to_scene(i);
            }
            if ui.button("Next ⏭").clicked() && self.project.current + 1 < self.project.scenes.len() {
                let i = self.project.current + 1;
                self.go_to_scene(i);
            }
        });
        heading(ui, "Scene notes");
        {
            let s = self.state_mut();
            ui.add(egui::TextEdit::multiline(&mut s.notes).desired_rows(4).desired_width(f32::INFINITY).hint_text("Blocking notes, scene-change crew, props…"));
        }
        heading(ui, "Save & export");
        let ctx = ui.ctx().clone();
        if ui.button("💾 Save project (.json)…").clicked() {
            self.save_json(now);
        }
        if ui.button("📂 Open project (.json)…").clicked() {
            self.open_json(now);
        }
        if ui.button("📷 Export PNG snapshot of this view…").clicked() {
            self.export_png(&ctx);
        }
    }
}
