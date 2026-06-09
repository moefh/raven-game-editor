use crate::app::WindowContext;
use crate::image::{
    ImageCollection,
    ColorSet,
    ColorSetSource,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SourceOptions {
    AllImages,                 // from all images, no priority
    SelectedImage,             // from selected image only
    PrioritizeSelectedImage,   // from all images, prioritizing selected
}

impl SourceOptions {
    pub fn text(&self) -> &'static str {
        match self {
            SourceOptions::AllImages => "all images",
            SourceOptions::SelectedImage => "only current image",
            SourceOptions::PrioritizeSelectedImage => "all images, prioritizing current",
        }
    }
}

pub struct CreateColorsetDialog {
    pub open: bool,
    pub selected_image: u32,
    pub created_colorset_index: usize,
    window_id: egui::Id,
    grid_id: egui::Id,
    source_combo_id: egui::Id,
    name: String,
    source: SourceOptions,
}

impl CreateColorsetDialog {
    pub fn new(id_prefix: impl AsRef<str>) -> Self {
        CreateColorsetDialog {
            window_id: egui::Id::new(format!("{}_{}", id_prefix.as_ref(), "create_colorset")),
            grid_id: egui::Id::new(format!("{}_{}", id_prefix.as_ref(), "create_colorset_grid")),
            source_combo_id: egui::Id::new(format!("{}_{}", id_prefix.as_ref(), "create_colorset_source_combo")),
            open: false,
            selected_image: 0,
            name: String::new(),
            source: SourceOptions::SelectedImage,
            created_colorset_index: 0,
        }
    }

    pub fn id() -> egui::Id {
        egui::Id::new("dlg_tileset_create_colorset")
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, selected_image: u32) {
        self.name.clear();
        self.name.push_str("new_colorset");
        self.source = SourceOptions::SelectedImage;
        self.selected_image = selected_image;
        self.created_colorset_index = 0;
        self.open = true;
        wc.set_window_open(Self::id(), self.open);
    }

    fn confirm(&mut self, wc: &mut WindowContext, image: &impl ImageCollection) -> bool {
        let source = match self.source {
            SourceOptions::AllImages => { ColorSetSource::AllImages }
            SourceOptions::SelectedImage => { ColorSetSource::SingleImage(self.selected_image) }
            SourceOptions::PrioritizeSelectedImage => { ColorSetSource::AllImagesPrioritizing(self.selected_image) }
        };
        let new_colorset_index = wc.settings.colorsets.add_custom_colorset(ColorSet::from_image(self.name.clone(), image, &source));
        self.created_colorset_index = new_colorset_index;
        true
    }

    pub fn show(&mut self, wc: &mut WindowContext, image: &impl ImageCollection) -> bool {
        if ! self.open { return false; }

        let mut confirmed = false;
        if egui::Modal::new(self.window_id).show(wc.egui.ctx, |ui| {
            wc.sys_dialogs.block_ui(ui);
            ui.set_width(450.0);
            ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
                ui.heading("Create Colorset");
                ui.separator();

                egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                    egui::Grid::new(self.grid_id)
                        .num_columns(2)
                        .spacing([8.0, 8.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut self.name);
                            ui.end_row();

                            ui.label("Colors from:");
                            egui::ComboBox::from_id_salt(self.source_combo_id)
                                .selected_text(self.source.text())
                                .width(50.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut self.source,
                                                        SourceOptions::SelectedImage,
                                                        SourceOptions::SelectedImage.text());
                                    ui.selectable_value(&mut self.source,
                                                        SourceOptions::PrioritizeSelectedImage,
                                                        SourceOptions::PrioritizeSelectedImage.text());
                                    ui.selectable_value(&mut self.source,
                                                        SourceOptions::AllImages,
                                                        SourceOptions::AllImages.text());
                                });
                            ui.end_row();
                        });
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    if ui.button("Cancel").clicked() {
                        ui.close();
                    }
                    if ui.button("Ok").clicked() && self.confirm(wc, image) {
                        confirmed = true;
                        ui.close();
                    }
                });
            });
        }).should_close() {
            self.open = false;
            wc.set_window_open(Self::id(), self.open);
        }
        confirmed
    }
}
