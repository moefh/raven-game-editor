mod about;
mod message_box;
mod confirmation;
mod colorset;
mod text_input;

use about::{*};
use message_box::{*};
use confirmation::{*};
use colorset::{*};
use text_input::{*};

use crate::misc::IMAGES;

use super::{
    SysDialogs,
    AppSettings,
    AppWindowTracker,
};

#[derive(PartialEq, Debug)]
pub enum DialogResult {
    None,
    Yes,
    No,
    Cancel,
}

pub struct AppDialogs {
    about: AboutDialog,
    message_box: MessageBoxDialog,
    confirmation: ConfirmationDialog,
    colorset: ColorsetEditorDialog,
    text_input: TextInputDialog,
}

impl AppDialogs {
    pub fn new() -> Self {
        AppDialogs {
            about: AboutDialog::new(),
            message_box: MessageBoxDialog::new(),
            confirmation: ConfirmationDialog::new(),
            colorset: ColorsetEditorDialog::new(),
            text_input: TextInputDialog::new(),
        }
    }

    pub fn open_message_box(&mut self, wt: &mut AppWindowTracker, title: impl AsRef<str>, text: impl AsRef<str>) {
        self.message_box.set_open(wt, title.as_ref(), text.as_ref());
    }

    pub fn open_confirmation_dialog(
        &mut self,
        wt: &mut AppWindowTracker,
        title: impl AsRef<str>,
        text: impl AsRef<str>,
        yes: impl AsRef<str>,
        no: impl AsRef<str>
    ) {
        self.confirmation.set_open(wt, title.as_ref(), text.as_ref(), yes.as_ref(), no.as_ref());
    }

    pub fn open_text_input_dialog(
        &mut self,
        wt: &mut AppWindowTracker,
        title: impl AsRef<str>,
        prompt: impl AsRef<str>,
        input: impl AsRef<str>,
        yes: impl AsRef<str>,
        no: impl AsRef<str>
    ) {
        self.text_input.set_open(wt, title.as_ref(), prompt.as_ref(), input.as_ref(), yes.as_ref(), no.as_ref());
    }

    pub fn open_about(&mut self, wt: &mut AppWindowTracker) {
        self.about.set_open(wt);
    }

    pub fn open_colorset_dialog(&mut self, wt: &mut AppWindowTracker, colorset: usize) {
        self.colorset.set_open(wt, colorset);
    }

    pub fn show_non_response_dialogs(&mut self, ui: &mut egui::Ui, wt: &mut AppWindowTracker,
                                     sys_dialogs: &SysDialogs, settings: &mut AppSettings) {
        self.about.show(ui, wt, sys_dialogs);
        self.message_box.show(ui, wt, sys_dialogs);
        self.colorset.show(ui, wt, sys_dialogs, settings);
    }

    pub fn show_confirmation_dialog(&mut self, ui: &mut egui::Ui, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) -> DialogResult {
        self.confirmation.show(ui, wt, sys_dialogs)
    }

    pub fn show_text_input_dialog(&mut self, ui: &mut egui::Ui, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs) -> DialogResult {
        self.text_input.show(ui, wt, sys_dialogs)
    }

    pub fn get_text_input_dialog_input(&mut self) -> String {
        self.text_input.get_input()
    }
}

pub fn create_dialog_window<T>(
    sys_dialogs: &SysDialogs,
    ui: &mut egui::Ui,
    id: egui::Id,
    width: f32,
    title: &str,
    add_content: impl FnOnce(&mut egui::Ui) -> T
) -> egui::ModalResponse<T> {
    let frame = egui::Frame::popup(ui.style())
        .inner_margin(egui::Margin::ZERO)
        .fill(ui.style().visuals.widgets.open.weak_bg_fill);
    egui::Modal::new(id).frame(frame).show(ui.ctx(), |ui| {
        sys_dialogs.block_ui(ui);
        ui.set_width(width);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
            // title bar
                let title_frame = egui::Frame::new().inner_margin(egui::Margin { left: 5, right: 5, top: 3, bottom: 0 });
                title_frame.show(ui, |ui| {
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.add_space(3.0);
                        ui.add(egui::Label::new(title).selectable(false));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.add(egui::Button::image(IMAGES.close).frame_when_inactive(false)).clicked() {
                                ui.close();
                            }
                        });
                    });
                });
                let size = egui::Vec2::new(ui.available_size_before_wrap().x, 1.0);
                let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());
                ui.painter().hline(
                    rect.left()..=rect.right(),
                    rect.bottom() + 2.0,
                    ui.style().visuals.window_stroke
                );

            // content
            egui::Frame::NONE.inner_margin(ui.style().spacing.menu_margin).fill(ui.style().visuals.window_fill).show(ui, |ui| {
                add_content(ui)
            }).inner
        }).inner
    })
}
