//! GachaHub – native launcher for gacha games, written entirely in Rust.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod backend;
mod i18n;
mod pages;
mod theme;

use eframe::egui;

fn main() -> eframe::Result {
    let icon =
        eframe::icon_data::from_png_bytes(include_bytes!("../assets/gachahub-icon.png")).ok();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("GachaHub")
        .with_app_id("gachahub")
        .with_inner_size([1360.0, 840.0])
        .with_min_inner_size([1080.0, 700.0]);
    if let Some(icon) = icon {
        viewport = viewport.with_icon(icon);
    }

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "GachaHub",
        options,
        Box::new(|cc| Ok(Box::new(app::GachaHubApp::new(cc)))),
    )
}
