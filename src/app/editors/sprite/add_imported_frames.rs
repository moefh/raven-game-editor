use crate::image::{
    ImageCollection,
    ImageCollectionIO,
    ImageSlicingMethod,
    ImageLoadOptions,
    ImagePixelsCollection,
};
use crate::data_asset::Sprite;

use super::super::{
    AssetEditorBase,
    WindowContext,
    EditorAction,
    SysDialogResponse,
    SysDialogOpenFile,
    AddImageLocation,
    ImageSlicingMethodOption,
};

pub struct AddImportedFramesDialog {
    pub open: bool,
    pub dlg_window_id: egui::Id,
    pub import_sprite_sys_dlg_id: String,
    pub load_options: ImageLoadOptions,
    pub add_frame_location: AddImageLocation,
    pub sel_frame: u32,
    pub clear_color: u8,
}

impl AddImportedFramesDialog {
    const DEFAULT_LOAD_OPTIONS: ImageLoadOptions = ImageLoadOptions {
        slicing_method: ImageSlicingMethod::by_number(1, 1),
        border: 0,
        space_between: 0,
        zoom_x: 1,
        zoom_y: 1,
    };

    pub fn new() -> Self {
        AddImportedFramesDialog {
            open: false,
            dlg_window_id: egui::Id::new("dlg_sprite_import"),
            load_options: Self::DEFAULT_LOAD_OPTIONS,
            add_frame_location: AddImageLocation::AtEnd,
            sel_frame: 0,
            clear_color: 0,
            import_sprite_sys_dlg_id: String::new(),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, sel_frame: u32, clear_color: u8, sprite: &Sprite) {
        self.add_frame_location = AddImageLocation::AtEnd;
        self.sel_frame = sel_frame;
        self.clear_color = clear_color;
        self.load_options = Self::DEFAULT_LOAD_OPTIONS;
        self.import_sprite_sys_dlg_id.replace_range(.., &format!("editor_{}_import_sprite", sprite.asset.id));
        self.open = true;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn close(&mut self, wc: &mut WindowContext) {
        self.open = false;
        wc.set_dialog_open(self.dlg_window_id, self.open);
    }

    fn create_empty_frames(&mut self, wc: &mut WindowContext, num_frames: u32, sprite: &mut Sprite) -> usize {
        let old_num_frames = sprite.num_frames;
        let insertion_point = match self.add_frame_location {
            AddImageLocation::BeforeSelected => { self.sel_frame.min(sprite.num_frames) }
            AddImageLocation::AfterSelected => { (self.sel_frame + 1).min(sprite.num_frames) }
            AddImageLocation::AtEnd => { sprite.num_frames }
        };
        sprite.resize(sprite.width, sprite.height, sprite.num_frames + num_frames, self.clear_color);
        if insertion_point < old_num_frames {
            let frame_size = (sprite.height * sprite.width) as usize;
            let src_start = insertion_point as usize * frame_size;
            let src_end = (sprite.num_frames - num_frames) as usize * frame_size;
            let dst_start = (insertion_point + num_frames) as usize * frame_size;
            sprite.data.copy_within(src_start..src_end, dst_start);
            sprite.data[src_start..dst_start].fill(self.clear_color);
            let num_frames_after_hole = old_num_frames - insertion_point;
            wc.add_editor_action(EditorAction::SpriteFramesAdded {
                sprite_id: sprite.asset.id,
                hole_start: insertion_point,
                hole_size: num_frames,
                num_frames_after_hole,
            });
        }
        insertion_point as usize * (sprite.height * sprite.width) as usize
    }

    fn confirm(&mut self, file: SysDialogOpenFile, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        let frames = file.read_data().and_then(|data| {
            let mut frames = ImagePixelsCollection::new(sprite.width, sprite.height, 1);
            frames.load_image_png(&data, &self.load_options).map(|_| frames)
        });
        match frames {
            Ok(frames) => {
                let num_frames = frames.num_items.min(255_u32.saturating_sub(sprite.num_frames));
                if num_frames == 0 {
                    wc.open_message_box(
                        "Error adding frames",
                        "This sprite has the maximum number of frames."
                    );
                } else if frames.width != sprite.width || frames.height != sprite.height {
                    wc.open_message_box(
                        "Error adding frames",
                        format!(
                            "The imported sprite must have the same\n\
                            size than from the current sprite:\n\
                            \n\
                              current: {}x{}\n\
                              imported: {}x{}.",
                            sprite.width,
                            sprite.height,
                            frames.width,
                            frames.height,
                        ),
                    );
                    return false;
                } else {
                    let frame_len = (sprite.height * sprite.width) as usize;
                    let dst_start = self.create_empty_frames(wc, num_frames, sprite);
                    let dst_len = num_frames as usize * frame_len;
                    sprite.data[dst_start..dst_start+dst_len].copy_from_slice(&frames.data[..dst_len]);
                }
                true
            }
            Err(e) => {
                wc.logger.log(format!("ERROR reading file from {}:", file.filename()));
                wc.logger.log(format!("{}", e));
                wc.open_message_box(
                    "Error importing sprite",
                    "Error importing sprite file.\n\nConsult the log window for more information."
                );
                false
            }
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, sprite: &mut Sprite) -> bool {
        if ! self.open { return false; }
        if let Some(SysDialogResponse::File(file)) = wc.sys_dialogs.get_response_for(&self.import_sprite_sys_dlg_id) &&
            self.confirm(file, wc, sprite) {
                self.close(wc);
                return true;
            }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_window_id, 350.0, "Add Imported Frames", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                egui::Grid::new(format!("editor_panel_{}_import_grid", sprite.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Zoom X:");
                        ui.add(egui::Slider::new(&mut self.load_options.zoom_x, 1..=256));
                        ui.end_row();

                        ui.label("Zoom Y:");
                        ui.add(egui::Slider::new(&mut self.load_options.zoom_y, 1..=256));
                        ui.end_row();

                        ui.label("Slice image:");
                        let mut slicing_option = ImageSlicingMethodOption::from_slicing_method(&self.load_options.slicing_method);
                        egui::ComboBox::from_id_salt(format!("editor_{}_import_combo_slicing", sprite.asset.id))
                            .selected_text(slicing_option.text())
                            .width(50.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut slicing_option,
                                                    ImageSlicingMethodOption::BySize,
                                                    ImageSlicingMethodOption::BySize.text());
                                ui.selectable_value(&mut slicing_option,
                                                    ImageSlicingMethodOption::ByNumber,
                                                    ImageSlicingMethodOption::ByNumber.text());
                            });
                        ui.end_row();

                        match slicing_option {
                            ImageSlicingMethodOption::BySize if ! matches!(
                                self.load_options.slicing_method,
                                ImageSlicingMethod::BySize{..}
                            ) => {
                                self.load_options.slicing_method = ImageSlicingMethod::by_size(sprite.width, sprite.height);
                            }
                            ImageSlicingMethodOption::ByNumber if ! matches!(
                                self.load_options.slicing_method,
                                ImageSlicingMethod::ByNumber{..}
                            ) => {
                                self.load_options.slicing_method = ImageSlicingMethod::by_number(1, 1);
                            }
                            _ => {}
                        }

                        match self.load_options.slicing_method {
                            ImageSlicingMethod::BySize { width, height } => {
                                let (mut w, mut h) = (width, height);
                                ui.label("Width:");  ui.add(egui::Slider::new(&mut w, 1..=256)); ui.end_row();
                                ui.label("Height:"); ui.add(egui::Slider::new(&mut h, 1..=256)); ui.end_row();
                                if w != width || h != height {
                                    self.load_options.slicing_method = ImageSlicingMethod::by_size(w, h);
                                }
                            }
                            ImageSlicingMethod::ByNumber { nx, ny } => {
                                let (mut x, mut y) = (nx, ny);
                                ui.label("Num X:"); ui.add(egui::Slider::new(&mut x, 1..=64)); ui.end_row();
                                ui.label("Num Y:"); ui.add(egui::Slider::new(&mut y, 1..=64)); ui.end_row();
                                if x != nx || y != ny {
                                    self.load_options.slicing_method = ImageSlicingMethod::by_number(x, y);
                                }
                            }
                        }

                        ui.label("Border:");
                        ui.add(egui::Slider::new(&mut self.load_options.border, 0..=32));
                        ui.end_row();

                        ui.label("Space between:");
                        ui.add(egui::Slider::new(&mut self.load_options.space_between, 0..=32));
                        ui.end_row();

                        ui.label("Insert at:");
                        egui::ComboBox::from_id_salt(format!("editor_panel_{}_insert_frame_at_combo", sprite.asset.id))
                            .selected_text(self.add_frame_location.text())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.add_frame_location,
                                    AddImageLocation::BeforeSelected,
                                    AddImageLocation::BeforeSelected.text()
                                );
                                ui.selectable_value(
                                    &mut self.add_frame_location,
                                    AddImageLocation::AfterSelected,
                                    AddImageLocation::AfterSelected.text()
                                );
                                ui.selectable_value(
                                    &mut self.add_frame_location,
                                    AddImageLocation::AtEnd,
                                    AddImageLocation::AtEnd.text()
                                );
                            });
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Open File").clicked() {
                    wc.sys_dialogs.open_file(
                        Some(wc.egui.window),
                        self.import_sprite_sys_dlg_id.clone(),
                        "sprite",
                        "Import Sprite",
                        &[
                            ("PNG files (*.png)", &["png"]),
                            ("All files (*.*)", &["*"]),
                        ]
                    );
                }
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
            });
        }).should_close() {
            self.close(wc);
        }
        false
    }
}
