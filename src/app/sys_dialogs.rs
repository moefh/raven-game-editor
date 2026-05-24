use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::data_asset::StringLogger;
use super::settings::AppPathLibrary;

pub struct PathLibraryEntry {
    path_id: String,
    lib: Arc<Mutex<AppPathLibrary>>,
}

impl PathLibraryEntry {
    fn new(path_id: &str, lib: Arc<Mutex<AppPathLibrary>>) -> Self {
        PathLibraryEntry {
            path_id: path_id.to_owned(),
            lib,
        }
    }

    pub fn set<P: AsRef<Path>>(&self, path: P) {
        self.lib.lock().unwrap().set(&self.path_id, path);
    }
}

pub enum SysDialogResponse {
    Cancel,
    File(std::path::PathBuf),
}

struct SysDialogResponseData {
    egui_ctx: egui::Context,
    response: Option<SysDialogResponse>,
}

struct SysDialogRequest {
    request_id: String,
    response_data: Arc<Mutex<SysDialogResponseData>>,
}

impl SysDialogRequest {
    fn new(request_id: String, egui_ctx: egui::Context) -> Self {
        SysDialogRequest {
            request_id,
            response_data: Arc::new(Mutex::new(SysDialogResponseData { egui_ctx, response: None })),
        }
    }

    fn is_pending(&self) -> bool {
        self.response_data.lock().unwrap().response.is_none()
    }

    fn open_file(file_dialog: rfd::FileDialog, path_entry: PathLibraryEntry, response_data: Arc<Mutex<SysDialogResponseData>>) {
        let file = file_dialog.pick_file();
        let mut data = response_data.lock().unwrap();
        data.response = match file {
            Some(file) => {
                if let Some(dir) = file.parent() {
                    path_entry.set(dir.to_path_buf());
                }
                Some(SysDialogResponse::File(file))
            }
            None => Some(SysDialogResponse::Cancel),
        };
        data.egui_ctx.request_repaint();
    }

    fn save_file(file_dialog: rfd::FileDialog, path_entry: PathLibraryEntry, response_data: Arc<Mutex<SysDialogResponseData>>) {
        let file = file_dialog.save_file();
        let mut data = response_data.lock().unwrap();
        data.response = match file {
            Some(file) => {
                if let Some(dir) = file.parent() {
                    path_entry.set(dir.to_path_buf());
                }
                Some(SysDialogResponse::File(file))
            }
            None => Some(SysDialogResponse::Cancel),
        };
        data.egui_ctx.request_repaint();
    }
}

pub struct SysDialogs {
    egui_ctx: egui::Context,
    request: Option<SysDialogRequest>,
    path_library: Arc<Mutex<AppPathLibrary>>,
}

impl SysDialogs {
    pub fn new(egui_ctx: egui::Context, path_library: AppPathLibrary) -> Self {
        SysDialogs {
            egui_ctx,
            request: None,
            path_library: Arc::new(Mutex::new(path_library)),
        }
    }

    pub fn has_open_dialog(&self) -> bool {
        match &self.request {
            None => false,
            Some(req) => req.is_pending(),
        }
    }

    pub fn block_ui(&self, ui: &mut egui::Ui) -> bool {
        if self.has_open_dialog() {
            ui.disable();
            true
        } else {
            false
        }
    }

    pub fn load_paths(&mut self, logger: &mut StringLogger) {
        self.path_library.lock().unwrap().load(logger);
    }

    pub fn save_paths(&mut self, logger: &mut StringLogger) {
        self.path_library.lock().unwrap().save(logger);
    }

    pub fn get_path_for_id(&self, path_id: &str) -> Option<PathBuf> {
        self.path_library.lock().unwrap().get(path_id)
    }

    pub fn set_path_for_id<P: AsRef<Path>>(&self, path_id: &str, path: P) {
        self.path_library.lock().unwrap().set(path_id, path);
    }

    pub fn get_response_for<S: AsRef<str>>(&mut self, request_id: S) -> Option<SysDialogResponse> {
        let response = match &self.request {
            None => None,
            Some(req) => {
                if req.request_id != request_id.as_ref() {
                    None
                } else {
                    let mut resp_data = req.response_data.lock().unwrap();
                    resp_data.response.take()
                }
            }
        };
        if response.is_some() {
            self.request = None;
        }
        response
    }

    pub fn open_file(&mut self, window: Option<&eframe::Frame>, request_id: String,
                     path_id: &str, title: &str, filters: &[(&str, &[&str])]) -> bool {
        if self.request.is_some() { return false; }

        let mut file_dialog = rfd::FileDialog::new().set_title(title);
        if let Some(dir) = self.get_path_for_id(path_id) {
            file_dialog = file_dialog.set_directory(dir);
        }
        if let Some(window) = window {
            file_dialog = file_dialog.set_parent(window);
        }
        for filter in filters.iter() {
            file_dialog = file_dialog.add_filter(filter.0, filter.1);
        }

        let path_entry = PathLibraryEntry::new(path_id, self.path_library.clone());
        let request = SysDialogRequest::new(request_id, self.egui_ctx.clone());
        let response_data = request.response_data.clone();
        thread::spawn(move || SysDialogRequest::open_file(file_dialog, path_entry, response_data));

        self.request = Some(request);
        true
    }

    pub fn save_file(&mut self, window: Option<&eframe::Frame>, request_id: String,
                     path_id: &str, title: &str, filters: &[(&str, &[&str])]) -> bool {
        if self.request.is_some() { return false; }

        let mut file_dialog = rfd::FileDialog::new().set_title(title);
        if let Some(dir) = self.get_path_for_id(path_id) {
            file_dialog = file_dialog.set_directory(dir);
        }
        if let Some(window) = window {
            file_dialog = file_dialog.set_parent(window);
        }
        for filter in filters.iter() {
            file_dialog = file_dialog.add_filter(filter.0, filter.1);
        }

        let path_entry = PathLibraryEntry::new(path_id, self.path_library.clone());
        let request = SysDialogRequest::new(request_id, self.egui_ctx.clone());
        let response_data = request.response_data.clone();
        thread::spawn(move || SysDialogRequest::save_file(file_dialog, path_entry, response_data));

        self.request = Some(request);
        true
    }
}
