// Copyright (C) 2026 Doug Rumble
// SPDX-License-Identifier: GPL-3.0-only

//! Stagecraft 3D – plan and preview theatre sets in real venues.
//!
//! Run normally to open the app. `--screenshot <file.png>` opens the window without
//! taking focus, draws about 120 frames, saves a picture of the window and exits.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod builder;
mod checks;
mod inspector;
mod joins;
mod library;
mod math;
mod mesh;
mod model;
mod panels;
mod raster;
mod sample;
mod scene;
mod startup;
mod style;
mod venues;

struct Host(app::App);

impl eframe::App for Host {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.frame(ui);
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let screenshot = args
        .iter()
        .position(|a| a == "--screenshot")
        .and_then(|i| args.get(i + 1).cloned());

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Stagecraft 3D – theatre set design")
        .with_inner_size([1480.0, 900.0])
        .with_min_inner_size([1000.0, 640.0]);
    if let Ok(img) = image::load_from_memory(include_bytes!("../assets/icon-256.png")) {
        let img = img.to_rgba8();
        let (w, h) = img.dimensions();
        viewport = viewport.with_icon(egui::IconData { rgba: img.into_raw(), width: w, height: h });
    }

    let mut options = eframe::NativeOptions::default();
    // Windows 7 / 8 builds (the "win7" targets) have no DirectX 12: draw with OpenGL there.
    #[cfg(target_vendor = "win7")]
    {
        options.renderer = eframe::Renderer::Glow;
    }
    // Anyone can force OpenGL (for old graphics drivers) with STAGECRAFT_OPENGL=1.
    if std::env::var_os("STAGECRAFT_OPENGL").is_some() {
        options.renderer = eframe::Renderer::Glow;
    }
    if screenshot.is_some() {
        viewport = viewport.with_active(false);
        #[cfg(target_os = "windows")]
        {
            viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnBottom);
        }
        #[cfg(target_os = "macos")]
        {
            options.event_loop_builder = Some(Box::new(|b| {
                use winit::platform::macos::EventLoopBuilderExtMacOS;
                b.with_activate_ignoring_other_apps(false);
            }));
        }
        // Without vsync a hidden/covered window still draws at full speed.
        #[cfg(feature = "modern")]
        {
            options.wgpu_options.surface.present_mode = eframe::wgpu::PresentMode::AutoNoVsync;
        }
    }
    options.viewport = viewport;

    eframe::run_native(
        "Stagecraft 3D",
        options,
        Box::new(move |cc| Ok(Box::new(Host(app::App::new(cc, screenshot))))),
    )
}
