#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console on Windows

mod misc;
mod data_asset;
mod editors;
mod app;

use crate::misc::image_table::IMAGES;
use crate::app::RavenEditorApp;

fn add_font(ctx: &egui::Context) {
    use eframe::epaint::text::{FontInsert, InsertFontFamily};

    ctx.add_font(FontInsert::new(
        "ComicMono",
        egui::FontData::from_static(include_bytes!("../assets/fonts/ComicMono.ttf")),
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

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1800.0, 900.0]),
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
            egui_extras::install_image_loaders(&cc.egui_ctx);
            add_font(&cc.egui_ctx);
            cc.egui_ctx.set_zoom_factor(1.5);
            cc.egui_ctx.options_mut(|opt: &mut egui::Options| {
                opt.zoom_with_keyboard = false;
            });
            cc.egui_ctx.set_theme(egui::ThemePreference::Light);
            Ok(Box::new(
                match filename {
                    Some(filename) => RavenEditorApp::from_file(cc, filename),
                    None => RavenEditorApp::new(cc),
                }
            ))
        })
    )
}
