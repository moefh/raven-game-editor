use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::platform::console_log;
use crate::data_asset::StringLogger;

enum SysDialogOpenFileData {
    Reader { data: Arc<Mutex<Vec<u8>>> },
    Writer { handle: Arc<Mutex<Option<rfd::FileHandle>>> },
}

pub struct SysDialogOpenFile {
    filename: String,
    data: SysDialogOpenFileData,
}

impl SysDialogOpenFile {
    pub fn create(_path: &Path) -> Option<Self> {
        None
    }

    fn reader(filename: String, data: Vec<u8>) -> Self {
        SysDialogOpenFile {
            filename,
            data: SysDialogOpenFileData::Reader {
                data: Arc::new(Mutex::new(data))
            },
        }
    }

    fn writer(handle: rfd::FileHandle) -> Self {
        let filename = handle.file_name();
        SysDialogOpenFile {
            filename,
            data: SysDialogOpenFileData::Writer {
                handle: Arc::new(Mutex::new(Some(handle)))
            },
        }
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn path(&self) -> Option<&Path> {
        None
    }

    pub fn read_data(&self) -> Result<Vec<u8>, std::io::Error> {
        match &self.data {
            SysDialogOpenFileData::Reader { data } => {
                let mut data = data.lock().unwrap();
                Ok(data.drain(..).collect())
            }
            SysDialogOpenFileData::Writer { .. } => {
                Err(std::io::Error::other("trying to read from a file open for writing"))
            }
        }
    }

    pub fn read_string(&self) -> Result<String, std::io::Error> {
        let data = self.read_data()?;
        let string = str::from_utf8(&data).map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(String::from(string))
    }

    pub fn write_data(&self, write_data: Vec<u8>) -> Result<(), std::io::Error> {
        match &self.data {
            SysDialogOpenFileData::Writer { handle } => {
                let mut handle = handle.lock().unwrap();
                if let Some(handle) = handle.take() {
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Err(e) = handle.write(&write_data).await {
                            console_log(format!("ERROR writing data: {}", e));
                        }
                    });
                }
                Ok(())
            }
            SysDialogOpenFileData::Reader { .. } => {
                Err(std::io::Error::other("trying to write to a file open for reading"))
            }
        }
    }

    pub fn write_string(&self, string: String) -> Result<(), std::io::Error> {
        self.write_data(string.into_bytes())
    }
}

pub enum SysDialogResponse {
    File(SysDialogOpenFile),
    Cancel,
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

    async fn open_file(future: impl Future<Output = Option<rfd::FileHandle>>, response_data: Arc<Mutex<SysDialogResponseData>>) {
        let resp = match future.await {
            Some(file_handle) => {
                let filename = file_handle.file_name();
                let data = file_handle.read().await;
                Some(SysDialogResponse::File(SysDialogOpenFile::reader(filename, data)))
            }
            None => {
                Some(SysDialogResponse::Cancel)
            }
        };

        let mut data = response_data.lock().unwrap();
        data.response = resp;
        data.egui_ctx.request_repaint();
    }

    async fn save_file(future: impl Future<Output = Option<rfd::FileHandle>>, response_data: Arc<Mutex<SysDialogResponseData>>) {
        let resp = match future.await {
            Some(file_handle) => {
                Some(SysDialogResponse::File(SysDialogOpenFile::writer(file_handle)))
            }
            None => {
                Some(SysDialogResponse::Cancel)
            }
        };

        let mut data = response_data.lock().unwrap();
        data.response = resp;
        data.egui_ctx.request_repaint();
    }
}

pub struct SysDialogs {
    egui_ctx: egui::Context,
    request: Option<SysDialogRequest>,
}

impl SysDialogs {
    pub fn new(egui_ctx: egui::Context) -> Self {
        SysDialogs {
            request: None,
            egui_ctx,
        }
    }

    pub fn load_paths(&mut self, _logger: &mut StringLogger) {}

    pub fn save_paths(&mut self, _logger: &mut StringLogger) {}

    pub fn set_path_for_id(&self, _path_id: &str, _path: impl AsRef<Path>) {}

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

    pub fn open_file(&mut self,
        _window: Option<&eframe::Frame>,
        request_id: String,
        _path_id: &str,
        title: &str,
        filters: &[(&str, &[&str])]
    ) -> bool {
        if self.request.is_some() { return false; }

        let mut file_dialog = rfd::AsyncFileDialog::new().set_title(title);
        for filter in filters.iter() {
            file_dialog = file_dialog.add_filter(filter.0, filter.1);
        }
        let future = file_dialog.pick_file();

        let request = SysDialogRequest::new(request_id, self.egui_ctx.clone());
        let response_data = request.response_data.clone();
        wasm_bindgen_futures::spawn_local(async move {
            SysDialogRequest::open_file(future, response_data).await;
        });

        self.request = Some(request);
        true
    }

    pub fn save_file(
        &mut self,
        _window: Option<&eframe::Frame>,
        request_id: String,
        path_id: &str,
        title: &str,
        filters: &[(&str, &[&str])]
    ) -> bool {
        if self.request.is_some() { return false; }

        let mut file_dialog = rfd::AsyncFileDialog::new().set_title(title);
        for filter in filters.iter() {
            file_dialog = file_dialog.add_filter(filter.0, filter.1);
        }
        if let Some(filter) = filters.first() && let Some(ext) = filter.1.first() {
            if *ext == "*" || *ext == "" {
                file_dialog = file_dialog.set_file_name(path_id);
            } else {
                file_dialog = file_dialog.set_file_name(format!("{}.{}", path_id, ext));
            }
        }
        let future = file_dialog.save_file();

        let request = SysDialogRequest::new(request_id, self.egui_ctx.clone());
        let response_data = request.response_data.clone();
        wasm_bindgen_futures::spawn_local(async move {
            SysDialogRequest::save_file(future, response_data).await;
        });

        self.request = Some(request);
        true
    }

    pub fn get_response_for(&mut self, request_id: impl AsRef<str>) -> Option<SysDialogResponse> {
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
}
