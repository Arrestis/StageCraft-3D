//! A calm dark theme with a warm stage-light accent.

use egui::{Color32, Stroke};

pub fn apply(ctx: &egui::Context) {
    ctx.set_visuals(egui::Visuals::dark());
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = egui::vec2(6.0, 5.0);
    style.spacing.button_padding = egui::vec2(7.0, 3.0);
    let v = &mut style.visuals;
    v.panel_fill = Color32::from_rgb(24, 26, 32);
    v.window_fill = Color32::from_rgb(28, 30, 38);
    v.extreme_bg_color = Color32::from_rgb(16, 17, 22);
    v.faint_bg_color = Color32::from_rgb(30, 32, 40);
    v.selection.bg_fill = Color32::from_rgb(150, 96, 30);
    v.selection.stroke = Stroke::new(1.0, Color32::from_rgb(255, 210, 140));
    v.widgets.inactive.weak_bg_fill = Color32::from_rgb(40, 43, 52);
    v.widgets.hovered.weak_bg_fill = Color32::from_rgb(58, 60, 72);
    v.widgets.active.weak_bg_fill = Color32::from_rgb(90, 70, 40);
    v.hyperlink_color = Color32::from_rgb(255, 160, 130);
    ctx.set_global_style(style);
    add_symbol_fonts(ctx);
}

/// Builds without the "modern" features (Windows 7/8) have no automatic system-font lookup, so the
/// icons and arrows in the menus would show as boxes. Add Windows' own symbol fonts as fallbacks.
#[cfg(not(feature = "modern"))]
fn add_symbol_fonts(ctx: &egui::Context) {
    use egui::{FontData, FontFamily, FontInsert, FontPriority, InsertFontFamily};
    let dir = std::env::var_os("WINDIR").map(std::path::PathBuf::from).unwrap_or_else(|| "C:\\Windows".into());
    for file in ["seguiemj.ttf", "seguisym.ttf"] {
        if let Ok(bytes) = std::fs::read(dir.join("Fonts").join(file)) {
            let families = [FontFamily::Proportional, FontFamily::Monospace]
                .into_iter()
                .map(|family| InsertFontFamily { family, priority: FontPriority::Lowest })
                .collect();
            ctx.add_font(FontInsert::new(file, FontData::from_owned(bytes), families));
        }
    }
}

#[cfg(feature = "modern")]
fn add_symbol_fonts(_ctx: &egui::Context) {}
