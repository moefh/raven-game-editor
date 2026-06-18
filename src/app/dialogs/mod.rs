mod about;
mod message_box;
mod confirmation;
mod colorset;

use about::{*};
use message_box::{*};
use confirmation::{*};
use colorset::{*};

pub use confirmation::ConfirmationDialogResult;

use super::{SysDialogs, AppSettings, AppWindowTracker};

pub struct AppDialogs {
    about: AboutDialog,
    message_box: MessageBoxDialog,
    confirmation: ConfirmationDialog,
    colorset: ColorsetEditorDialog,
}

impl AppDialogs {
    pub fn new() -> Self {
        AppDialogs {
            about: AboutDialog::new(),
            message_box: MessageBoxDialog::new(),
            confirmation: ConfirmationDialog::new(),
            colorset: ColorsetEditorDialog::new(),
        }
    }

    pub fn open_message_box(&mut self, wt: &mut AppWindowTracker, title: impl AsRef<str>, text: impl AsRef<str>) {
        self.message_box.set_open(wt, title.as_ref(), text.as_ref());
    }

    pub fn open_confirmation_dialog(&mut self, wt: &mut AppWindowTracker, title: impl AsRef<str>,
                                    text: impl AsRef<str>, yes: impl AsRef<str>, no: impl AsRef<str>) {
        self.confirmation.set_open(wt, title.as_ref(), text.as_ref(), yes.as_ref(), no.as_ref());
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

    pub fn show_confirmation_dialog(&mut self, ui: &mut egui::Ui, wt: &mut AppWindowTracker, sys_dialogs: &SysDialogs)
                                    -> ConfirmationDialogResult {
        self.confirmation.show(ui, wt, sys_dialogs)
    }

}

pub fn create_dialog_window<T>(sys_dialogs: &SysDialogs, ui: &mut egui::Ui, id: egui::Id, width: f32, title: &str,
                               add_content: impl FnOnce(&mut egui::Ui) -> T) -> egui::ModalResponse<T> {
    let title_bg = ui.style().visuals.widgets.open.weak_bg_fill;

    let frame = egui::Frame::popup(ui.style()).inner_margin(egui::Margin::ZERO);
    egui::Modal::new(id).frame(frame).show(ui.ctx(), |ui| {
        sys_dialogs.block_ui(ui);
        ui.set_width(width);
        ui.with_layout(egui::Layout::top_down_justified(egui::Align::Center), |ui| {
            // title bar
            egui::Frame::NONE.fill(title_bg).show(ui, |ui| {
                ui.add_space(8.0);
                ui.add(egui::Label::new(egui::RichText::new(title).heading()).selectable(false));

                let size = egui::Vec2::new(ui.available_size_before_wrap().x, 1.0);
                let (rect, _response) = ui.allocate_at_least(size, egui::Sense::hover());
                ui.painter().hline(
                    rect.left()..=rect.right(),
                    rect.bottom(),
                    ui.style().visuals.window_stroke
                );
            });

            // content
            egui::Frame::NONE.inner_margin(ui.style().spacing.menu_margin).show(ui, |ui| {
                add_content(ui)
            }).inner
        }).inner
    })
}
