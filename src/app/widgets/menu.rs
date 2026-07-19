use crate::misc::IMAGES;

pub fn menu_item_no_image(text: &str) -> egui::Button<'_> {
    menu_item(IMAGES.blank, text)
}

pub fn menu_item<'a>(image: impl Into<egui::Image<'a>>, text: &str) -> egui::Button<'a> {
    egui::Button::image_and_text(image, text).wrap_mode(egui::TextWrapMode::Extend)
}

pub fn menu_item_no_image_with_submenu<R>(
    text: &str,
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R
) -> egui::InnerResponse<Option<R>> {
    menu_item_with_submenu(IMAGES.blank, text, ui, add_contents)
}

pub fn menu_item_with_submenu<'a, R>(
    image: impl Into<egui::Image<'a>>,
    text: &str,
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R
) -> egui::InnerResponse<Option<R>> {
    let (response, inner) = egui::menu::SubMenuButton::from_button(
        menu_item(image, text).right_text(egui::menu::SubMenuButton::RIGHT_ARROW),
    ).ui(ui, add_contents);
    egui::InnerResponse::new(inner.map(|i| i.inner), response)
}
