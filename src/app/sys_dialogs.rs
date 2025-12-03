use std::sync::{Arc, Mutex};
use std::thread;

pub enum SysDialogResponse {
    Cancel,
    File(std::path::PathBuf),
}

struct SysDialogResponseData {
    response: Option<SysDialogResponse>,
}

struct SysDialogRequest {
    request_id: String,
    response_data: Arc<Mutex<SysDialogResponseData>>,
}

impl SysDialogRequest {
    fn new(request_id: String) -> Self {
        SysDialogRequest {
            request_id,
            response_data: Arc::new(Mutex::new(SysDialogResponseData { response: None })),
        }
    }

    fn is_pending(&self) -> bool {
        self.response_data.lock().unwrap().response.is_none()
    }

    fn open_file(file_dialog: rfd::FileDialog, response_data: Arc<Mutex<SysDialogResponseData>>) {
        let file = file_dialog.pick_file();
        let mut data = response_data.lock().unwrap();
        data.response = match file {
            Some(file) => Some(SysDialogResponse::File(file)),
            None => Some(SysDialogResponse::Cancel),
        };
    }

    fn save_file(file_dialog: rfd::FileDialog, response_data: Arc<Mutex<SysDialogResponseData>>) {
        let file = file_dialog.save_file();
        let mut data = response_data.lock().unwrap();
        data.response = match file {
            Some(file) => Some(SysDialogResponse::File(file)),
            None => Some(SysDialogResponse::Cancel),
        };
    }
}

pub struct SysDialogs {
    request: Option<SysDialogRequest>,
}

impl SysDialogs {
    pub fn new() -> Self {
        SysDialogs {
            request: None,
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

    pub fn open_file(&mut self, window: Option<&eframe::Frame>, request_id: String, title: &str, filters: &[(&str, &[&str])]) -> bool {
        if self.request.is_some() { return false; }

        let mut file_dialog = rfd::FileDialog::new().set_title(title);
        if let Some(window) = window {
            file_dialog = file_dialog.set_parent(window);
        }
        for filter in filters.iter() {
            file_dialog = file_dialog.add_filter(filter.0, filter.1);
        }

        let request = SysDialogRequest::new(request_id);
        let response_data = request.response_data.clone();
        thread::spawn(move || SysDialogRequest::open_file(file_dialog, response_data));

        self.request = Some(request);
        true
    }

    pub fn save_file(&mut self, window: Option<&eframe::Frame>, request_id: String, title: &str, filters: &[(&str, &[&str])]) -> bool {
        if self.request.is_some() { return false; }

        let mut file_dialog = rfd::FileDialog::new().set_title(title);
        if let Some(window) = window {
            file_dialog = file_dialog.set_parent(window);
        }
        for filter in filters.iter() {
            file_dialog = file_dialog.add_filter(filter.0, filter.1);
        }

        let request = SysDialogRequest::new(request_id);
        let response_data = request.response_data.clone();
        thread::spawn(move || SysDialogRequest::save_file(file_dialog, response_data));

        self.request = Some(request);
        true
    }
}
