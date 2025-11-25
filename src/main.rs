#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console on Windows

mod misc;
mod image;
mod sound;
mod data_asset;
mod editors;
mod app;

use crate::misc::image_table::IMAGES;
use crate::app::RavenEditorApp;

static FONT_BYTES: &[u8] = include_bytes!("../assets/fonts/ComicMono.ttf");

pub fn add_font(ctx: &egui::Context) {
    use eframe::epaint::text::{FontInsert, InsertFontFamily};

    ctx.add_font(FontInsert::new(
        "ComicMono",
        egui::FontData::from_static(FONT_BYTES),
        vec![
            InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Highest,
            },
            InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: egui::epaint::text::FontPriority::Lowest,
            },
        ],
    ));
}

fn load_icon() -> Option<egui::IconData> {
    let data = include_bytes!("../assets/PicoIcon.png");
    let image = match ::image::load_from_memory(data) {
        Ok(img) => img,
        Err(e) => {
            println!("Warning: failed to load icon: {}", e);
            return None;
        },
    };
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(egui::IconData {
        rgba: rgba.to_vec(),
        width,
        height,
    })
}

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default().with_inner_size([1800.0, 900.0]);
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    let argv: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let filename: Option<&std::path::Path> = match argv.get(1) {
        Some(s) => Some(s.as_ref()),
        None => None,
    };

    eframe::run_native(
        "Raven Game Editor",
        options,
        Box::new(|cc| {
            Ok(Box::new(
                match filename {
                    Some(filename) => RavenEditorApp::from_file(cc, filename),
                    None => RavenEditorApp::new(cc),
                }
            ))
        })
    )
}
