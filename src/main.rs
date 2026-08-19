#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console on Windows

mod misc;
mod image;
mod sound;
mod data_asset;
mod app;
mod platform;

use crate::app::{
    RavenEditorApp,
    AppSettings,
};
use crate::data_asset::StringLogger;

const SEND_LOG_TO_STDOUT: bool = false;

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

#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    let mut logger = StringLogger::new(SEND_LOG_TO_STDOUT);
    let settings = AppSettings::load(&mut logger);
    let mut viewport = egui::ViewportBuilder::default().with_clamp_size_to_monitor_size(true);
    if settings.start_maximized {
        viewport = viewport.with_inner_size([10000.0, 10000.0]);
    } else {
        viewport = viewport.with_inner_size([1800.0, 900.0]);
    }
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        centered: true,
        ..Default::default()
    };

    let argv: Vec<std::ffi::OsString> = std::env::args_os().collect();
    let filename: Option<std::path::PathBuf> = argv.get(1).map(std::path::PathBuf::from);

    eframe::run_native(
        "Raven Game Editor",
        options,
        Box::new(|cc| {
            let mut app = RavenEditorApp::new(cc, false, logger, settings);
            if let Some(filename) = filename && let Some(file) = app::SysDialogOpenFile::create(&filename) {
                app.open(file);
            }
            Ok(Box::new(app))
        })
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("main_canvas")
            .expect("Failed to find main_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("main_canvas was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    let mut logger = StringLogger::new(SEND_LOG_TO_STDOUT);
                    let settings = AppSettings::load(&mut logger);
                    Ok(Box::new(RavenEditorApp::new(cc, true, logger, settings)))
                }),
            )
            .await;

        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(()) => {
                    loading_text.remove();
                }
                Err(err) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {err:?}");
                }
            }
        }
    });
}
