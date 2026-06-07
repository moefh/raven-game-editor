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
