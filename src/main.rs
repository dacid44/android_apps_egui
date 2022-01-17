#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

use image::io::Reader as ImageReader;
use std::io::Cursor;
use image::EncodableLayout;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let app = android_apps_egui::AppOrganizerApp::default();
    let mut native_options = eframe::NativeOptions::default();
    let icon_file = include_bytes!("icon/android_apps_egui.png");
    let icon_reader = ImageReader::with_format(Cursor::new(icon_file), image::ImageFormat::Png).decode().unwrap();
    let icon = icon_reader.as_rgba8().unwrap();
    native_options.icon_data = Some(eframe::epi::IconData { rgba: icon.as_bytes().to_vec(), width: 256, height: 256 });
    eframe::run_native(Box::new(app), native_options);
}
