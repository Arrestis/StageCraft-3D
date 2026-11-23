//! The "Build Custom Piece" window: assemble a piece from simple blocks.

use crate::app::*;
use crate::library;
use crate::model::*;
use crate::panels::*;
use crate::scene::fmt_len;
use egui::{Color32, RichText};

impl App {
    pub fn open_builder(&mut self, item: Option<LibItem>) {
        let item = item.unwrap_or_else(|| LibItem {
            name: "My custom piece".into(),
            category: "Custom".into(),
            material: Material::PaintedWood,
            color: [176, 150, 120],
            parts: vec![Part { shape: Shape::Box, offset: [0.0, 0.0, 0.0], size: [1.2, 0.9, 0.6], rot: 0.0, color: None, material: None, hinge: None }],
            template: None,
        });
        self.builder = Some(Builder { item, editing_piece: None, editing_lib: None, preview_on_stage: true });
    }

    pub fn builder_window(&mut self, ctx: &egui::Context, now: f64) {
        let Some(mut b) = self.builder.take() else { return };
        let mut open = true;
        let mut close = false;
        let feet = self.feet;
        let mut result: Option<(LibItem, u8)> = None;
        egui::Window::new("🛠 Build Custom Piece")
            .open(&mut open)
            .default_width(380.0)
            .default_pos(egui::pos2(330.0, 120.0))
            .resizable(true)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name");
                    ui.text_edit_singleline(&mut b.item.name);
                });
                egui::ComboBox::from_label("Main material").selected_text(b.item.material.label()).show_ui(ui, |ui| {
                    for m in Material::ALL {
                        if ui.selectable_value(&mut b.item.material, m, m.label()).clicked() {
                            b.item.color = m.default_color();
                        }
                    }
                });
                color_edit(ui, "Main colour", &mut b.item.color);
                ui.checkbox(&mut b.preview_on_stage, "Preview live at centre stage");
                ui.separator();
                ui.label(RichText::new("Add a building block:").strong());
                let mut add_part: Option<(Shape, [f32; 3], f32)> = None;
                let mut add_hinged: Option<(Shape, [f32; 3], i8, &str)> = None;
                ui.horizontal_wrapped(|ui| {
                    if ui.button("▭ Box / flat").clicked() {
                        add_part = Some((Shape::Box, [1.2, 2.4, 0.08], 0.0));
                    }
                    if ui.button("▬ Platform").clicked() {
                        add_part = Some((Shape::Box, [2.4, 0.4, 1.2], 0.0));
                    }
                    if ui.button("◯ Column").clicked() {
                        add_part = Some((Shape::Cylinder, [0.4, 2.6, 0.4], 0.0));
                    }
                    if ui.button("◢ Ramp").clicked() {
                        add_part = Some((Shape::Ramp, [1.2, 0.5, 2.0], 0.0));
                    }
                    if ui.button("≡ Stairs").clicked() {
                        add_part = Some((Shape::Stairs(4), [1.0, 0.8, 1.2], 0.0));
                    }
                    if ui.button("◸ Angled wall").clicked() {
                        add_part = Some((Shape::Box, [2.0, 2.7, 0.08], 30.0));
                    }
                    if ui.button("⌒ Arch top").clicked() {
                        add_part = Some((Shape::ArchTop, [1.5, 0.75, 0.3], 0.0));
                    }
                    if ui.button("● Sphere / rock").clicked() {
                        add_part = Some((Shape::Sphere, [1.0, 0.8, 1.0], 0.0));
                    }
                    if ui.button("▲ Cone").clicked() {
                        add_part = Some((Shape::Cone, [0.6, 0.8, 0.6], 0.0));
                    }
                    if ui.button("🚪 Door leaf").on_hover_text("A hinged panel that swings open and shut").clicked() {
                        add_hinged = Some((Shape::Box, [0.9, 2.1, 0.04], -1, "Door"));
                    }
                    if ui.button("🗃 Lid / trapdoor").on_hover_text("A panel that lifts up about its back edge").clicked() {
                        add_hinged = Some((Shape::Box, [0.8, 0.04, 0.6], 0, "Lid"));
                    }
                });
                if let Some((shape, size, side, label)) = add_hinged {
                    let n = b.item.parts.len() as f32;
                    b.item.parts.push(Part {
                        shape,
                        offset: [n * 0.3, 0.0, 0.0],
                        size,
                        rot: 0.0,
                        color: None,
                        material: None,
                        hinge: Some(Hinge { side, open_deg: 0.0, outwards: true, max_deg: 110.0, label: label.into(), pivot: None }),
                    });
                }
                if let Some((shape, size, rot)) = add_part {
                    let n = b.item.parts.len() as f32;
                    b.item.parts.push(Part { shape, offset: [n * 0.3, 0.0, 0.0], size, rot, color: None, material: None, hinge: None });
                }
                ui.separator();
                let mut remove = None;
                let mut dup = None;
                let last = b.item.parts.len().saturating_sub(1);
                egui::ScrollArea::vertical().max_height(330.0).show(ui, |ui| {
                    for (i, p) in b.item.parts.iter_mut().enumerate() {
                        egui::CollapsingHeader::new(format!("{}. {}", i + 1, p.shape.label())).id_salt(("part", i)).default_open(i == last).show(ui, |ui| {
                            egui::ComboBox::from_id_salt(("shape", i)).selected_text(p.shape.label()).show_ui(ui, |ui| {
                                for s in [Shape::Box, Shape::Cylinder, Shape::Cone, Shape::Sphere, Shape::Ramp, Shape::Stairs(4), Shape::ArchTop] {
                                    let is = std::mem::discriminant(&p.shape) == std::mem::discriminant(&s);
                                    if ui.selectable_label(is, s.label()).clicked() {
                                        p.shape = s;
                                    }
                                }
                            });
                            if let Shape::Stairs(n) = &mut p.shape {
                                ui.add(egui::Slider::new(n, 1..=20).text("Treads"));
                            }
                            len_edit(ui, "Width", &mut p.size[0], feet, 0.01, 20.0);
                            len_edit(ui, "Depth", &mut p.size[2], feet, 0.005, 20.0);
                            len_edit(ui, "Height", &mut p.size[1], feet, 0.005, 20.0);
                            len_edit(ui, "Offset across", &mut p.offset[0], feet, -20.0, 20.0);
                            len_edit(ui, "Offset up/downstage", &mut p.offset[2], feet, -20.0, 20.0);
                            len_edit(ui, "Raised by", &mut p.offset[1], feet, -5.0, 20.0);
                            deg_edit(ui, "Rotation", &mut p.rot, -180.0, 180.0);
                            hinge_edit(ui, p, i);
                            let mut own = p.color.is_some();
                            if ui.checkbox(&mut own, "Own colour").changed() {
                                p.color = if own { Some([200, 200, 200]) } else { None };
                            }
                            if let Some(c) = &mut p.color {
                                color_edit(ui, "Colour", c);
                            }
                            ui.horizontal(|ui| {
                                if ui.small_button("⧉ Duplicate block").clicked() {
                                    dup = Some(i);
                                }
                                if ui.small_button("🗑 Remove block").clicked() {
                                    remove = Some(i);
                                }
                            });
                        });
                    }
                });
                if let Some(i) = dup {
                    let mut p = b.item.parts[i].clone();
                    p.offset[0] += p.size[0];
                    b.item.parts.push(p);
                }
                if let Some(i) = remove {
                    b.item.parts.remove(i);
                }
                let (lo, hi) = b.item.bounds();
                ui.label(
                    RichText::new(format!("Overall: {} wide × {} deep × {} high", fmt_len(hi[0] - lo[0], feet), fmt_len(hi[2] - lo[2], feet), fmt_len(hi[1] - lo[1], feet)))
                        .color(Color32::from_rgb(150, 200, 240)),
                );
                ui.separator();
                ui.horizontal_wrapped(|ui| {
                    let ok = !b.item.parts.is_empty();
                    if b.editing_piece.is_some() && ui.add_enabled(ok, egui::Button::new("✔ Update piece on stage")).clicked() {
                        result = Some((b.item.clone(), 3));
                    }
                    if ui.add_enabled(ok, egui::Button::new("+ Add to stage")).clicked() {
                        result = Some((b.item.clone(), 1));
                    }
                    if ui.add_enabled(ok, egui::Button::new("⭐ Save to my library")).clicked() {
                        result = Some((b.item.clone(), 2));
                    }
                    if ui.button("Close").clicked() {
                        close = true;
                    }
                });
            });
        if let Some((item, what)) = result {
            match what {
                1 => {
                    self.add_piece_from(&item);
                    self.notify(format!("Added '{}' to the stage", item.name), now);
                }
                2 => {
                    if let Some(i) = b.editing_lib.filter(|i| *i < self.project.custom_library.len()) {
                        self.project.custom_library[i] = item.clone();
                    } else {
                        self.project.custom_library.push(item.clone());
                        b.editing_lib = Some(self.project.custom_library.len() - 1);
                    }
                    self.notify(format!("Saved '{}' to My custom pieces", item.name), now);
                }
                _ => {
                    if let Some(id) = b.editing_piece {
                        let s = self.state_mut();
                        if let Some(p) = s.pieces.iter_mut().find(|p| p.id == id) {
                            let np = library::make_piece(&item, id, p.pos);
                            p.parts = np.parts;
                            p.base_size = np.base_size;
                            p.size = np.size;
                            p.name = item.name.clone();
                            p.material = item.material;
                            p.color = item.color;
                            p.template = item.template.clone();
                        }
                        self.notify("Piece updated", now);
                    }
                }
            }
        }
        if open && !close {
            self.builder = Some(b);
        }
    }
}

/// Hinge settings of one block: none, or which edge it swings on and how far it is open.
fn hinge_edit(ui: &mut egui::Ui, p: &mut Part, i: usize) {
    let mut on = p.hinge.is_some();
    if ui.checkbox(&mut on, "Hinged (opens and shuts)").changed() {
        p.hinge = if on { Some(Hinge { side: -1, open_deg: 0.0, outwards: true, max_deg: 110.0, label: format!("Door {}", i + 1), pivot: None }) } else { None };
    }
    if let Some(h) = p.hinge.as_mut() {
        ui.horizontal(|ui| {
            ui.label("Name");
            ui.text_edit_singleline(&mut h.label);
        });
        ui.horizontal(|ui| {
            ui.label("Hinge on");
            ui.selectable_value(&mut h.side, -1, "Left edge");
            ui.selectable_value(&mut h.side, 1, "Right edge");
            ui.selectable_value(&mut h.side, 0, "Back edge (lid)");
        });
        if h.side != 0 {
            ui.checkbox(&mut h.outwards, "Opens towards the front");
        }
        ui.add(egui::Slider::new(&mut h.max_deg, 10.0..=180.0).suffix("°").text("Opens up to"));
        ui.add(egui::Slider::new(&mut h.open_deg, 0.0..=h.max_deg).suffix("°").text("Open now"));
    }
}
