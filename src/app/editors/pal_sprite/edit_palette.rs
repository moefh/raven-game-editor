use crate::app::WindowContext;
use crate::image::colors::{
    color_to_rgb,
    color_to_rgb_contrast,
    color_to_6bit_rgb,
    color_6bit_rgb_to_color,
};
use crate::data_asset::{PalSprite, PalSpriteDepth};

use super::super::{
    AssetEditorBase,
    ColorPickerPopupWidget,
};

const CLOSE_PICKER_ON_CLICK: bool = true;
const MIN_PALETTE_WIDTH: f32 = 300.0;

#[derive(Copy, Clone, PartialEq, Eq)]
enum SetPaletteOptions {
    ChangePaletteColors,
    AdaptImageColors,
}

impl SetPaletteOptions {
    fn text(&self) -> &str {
        match self {
            SetPaletteOptions::ChangePaletteColors => "just change palette",
            SetPaletteOptions::AdaptImageColors => "adapt image",
        }
    }
}

enum PalettePreset {
    White2,
    Red2,
    Green2,
    Blue2,
    Red4,
    Green4,
    Blue4,
    Gray4,
    Ega16,
    Red16,
    Green16,
    Blue16,
    Yellow16,
}

impl PalettePreset {
    fn text(&self) -> &'static str {
        match self {
            PalettePreset::White2   => "Black & White",
            PalettePreset::Red2     => "Red",
            PalettePreset::Green2   => "Green",
            PalettePreset::Blue2    => "Blue",
            PalettePreset::Red4     => "Red",
            PalettePreset::Green4   => "Green",
            PalettePreset::Blue4    => "Blue",
            PalettePreset::Gray4    => "Gray",
            PalettePreset::Ega16    => "EGA default",
            PalettePreset::Red16    => "Red",
            PalettePreset::Green16  => "Green",
            PalettePreset::Blue16   => "Blue",
            PalettePreset::Yellow16 => "Yellow",
        }
    }

    fn colors(&self) -> &'static [u8] {
        match self {
            PalettePreset::White2  => &[0b00_000_000, 0b11_111_111],
            PalettePreset::Red2    => &[0b00_000_000, 0b00_000_111],
            PalettePreset::Green2  => &[0b00_000_000, 0b00_111_000],
            PalettePreset::Blue2   => &[0b00_000_000, 0b11_000_000],

            PalettePreset::Red4   => &[0b00_000_000, 0b00_000_001, 0b00_000_100, 0b00_000_111],
            PalettePreset::Green4 => &[0b00_000_000, 0b00_001_000, 0b00_100_000, 0b00_111_000],
            PalettePreset::Blue4  => &[0b00_000_000, 0b01_000_000, 0b10_000_000, 0b11_000_000],
            PalettePreset::Gray4  => &[0b00_000_000, 0b01_010_010, 0b10_100_100, 0b11_111_111],

            PalettePreset::Ega16 => &[
                0b00_000_000, 0b10_000_000, 0b00_101_000, 0b10_101_000, 0b00_000_101, 0b10_000_101, 0b00_101_101, 0b10_101_101,
                0b01_010_010, 0b11_010_010, 0b01_111_010, 0b11_111_010, 0b01_010_111, 0b11_010_111, 0b01_111_111, 0b11_111_111,
            ],
            PalettePreset::Red16 => &[
                0b00_000_000, 0b00_000_001, 0b00_000_010, 0b00_000_011, 0b00_000_100, 0b00_000_101, 0b00_000_110, 0b00_000_111,
                0b00_001_111, 0b01_010_111, 0b01_011_111, 0b10_100_111, 0b10_101_111, 0b10_110_111, 0b11_110_111, 0b11_111_111,
            ],
            PalettePreset::Green16 => &[
                0b00_000_000, 0b00_001_000, 0b00_010_000, 0b00_011_000, 0b00_100_000, 0b00_101_000, 0b00_110_000, 0b00_111_000,
                0b00_111_001, 0b01_111_010, 0b01_111_011, 0b10_111_100, 0b10_111_101, 0b10_111_110, 0b11_111_110, 0b11_111_111,
            ],
            PalettePreset::Blue16 => &[
                0b00_000_000, 0b01_000_000, 0b10_000_000, 0b11_000_000, 0b10_001_001, 0b10_010_010, 0b10_011_011, 0b10_100_100,
                0b10_101_101, 0b11_001_001, 0b11_010_010, 0b11_011_011, 0b11_100_100, 0b11_101_101, 0b11_110_110, 0b11_111_111,
            ],
            PalettePreset::Yellow16 => &[
                0b00_000_000, 0b00_001_001, 0b00_010_010, 0b00_011_011, 0b00_100_100, 0b00_101_101, 0b00_110_110, 0b00_111_111,
                0b01_100_111, 0b01_101_111, 0b01_110_111, 0b01_111_111, 0b01_111_110, 0b01_111_111, 0b10_111_111, 0b11_111_111,
            ],
        }
    }

    fn load(&self, palette: &mut [u8]) {
        let c = self.colors();
        palette[0..c.len()].copy_from_slice(c);
        if palette.len() > c.len() {
            for color in palette[c.len()..].iter_mut() {
                *color = c[0];
            }
        }
    }

    fn presets_for_depth(depth: PalSpriteDepth) -> &'static [PalettePreset] {
        match depth {
            PalSpriteDepth::Bpp1 => &[
                PalettePreset::White2,
                PalettePreset::Red2,
                PalettePreset::Green2,
                PalettePreset::Blue2,
            ],
            PalSpriteDepth::Bpp2 => &[
                PalettePreset::Red4,
                PalettePreset::Green4,
                PalettePreset::Blue4,
                PalettePreset::Gray4,
            ],
            PalSpriteDepth::Bpp4 => &[
                PalettePreset::Ega16,
                PalettePreset::Red16,
                PalettePreset::Green16,
                PalettePreset::Blue16,
                PalettePreset::Yellow16,
            ],
        }
    }
}

pub struct EditPaletteDialog {
    pub image_changed: bool,
    pub open: bool,
    dlg_id: egui::Id,
    depth: PalSpriteDepth,
    set_pal_options: SetPaletteOptions,
    palette: Vec<u8>,
    edit_color_index: Option<usize>,
    popup: ColorPickerPopupWidget,
}

impl EditPaletteDialog {
    pub fn new() -> Self {
        EditPaletteDialog {
            dlg_id: egui::Id::new("dlg_pal_sprite_pal_edit"),
            image_changed: false,
            open: false,
            depth: PalSpriteDepth::Bpp1,
            set_pal_options: SetPaletteOptions::ChangePaletteColors,
            palette: vec![0; 16],
            edit_color_index: None,
            popup: ColorPickerPopupWidget::new(egui::Id::new("dlg_pal_sprite_pal_edit_popup"), CLOSE_PICKER_ON_CLICK),
        }
    }

    pub fn set_open(&mut self, wc: &mut WindowContext, pal_sprite: &PalSprite) {
        self.depth = pal_sprite.depth;
        self.set_pal_options = SetPaletteOptions::ChangePaletteColors;
        self.palette[..].copy_from_slice(&pal_sprite.palette);
        self.edit_color_index = if wc.vga_bits_per_pixel == 8 { None } else { Some(0) };
        self.open = true;
        wc.set_dialog_open(self.dlg_id, self.open);
    }

    fn confirm(&mut self, pal_sprite: &mut PalSprite) {
        let new_color_to_index = PalSprite::gen_color_to_palette_index_map(&self.palette);
        let conversion_color_to_index = match self.set_pal_options {
            SetPaletteOptions::ChangePaletteColors => { &pal_sprite.color_to_palette_index_map }  // use color at old pal index
            SetPaletteOptions::AdaptImageColors    => { &new_color_to_index }                     // use color at index with nearest color
        };
        for pixel in pal_sprite.data.iter_mut() {
            let index = conversion_color_to_index[*pixel as usize] as usize;
            *pixel = self.palette[index];
        }
        pal_sprite.depth = self.depth;
        pal_sprite.palette[..].copy_from_slice(&self.palette);
        pal_sprite.color_to_palette_index_map[..].copy_from_slice(&new_color_to_index);
        self.image_changed = true;
    }

    fn draw_palette(&self, painter: &egui::Painter, rect: egui::Rect, dims: (i32, i32)) {
        let (pal_colors_x, pal_colors_y) = dims;
        let item_w = rect.width() / (pal_colors_x as f32);
        let item_h = rect.height() / (pal_colors_y as f32);
        for y in 0..pal_colors_y {
            for x in 0..pal_colors_x {
                let item_rect = egui::Rect {
                    min: egui::Pos2::new(rect.min.x + (x     as f32) * item_w, rect.min.y + (y     as f32) * item_h),
                    max: egui::Pos2::new(rect.min.x + ((x+1) as f32) * item_w, rect.min.y + ((y+1) as f32) * item_h),
                };
                let color_index = (y*pal_colors_x+x) as usize;
                painter.rect_filled(item_rect, egui::CornerRadius::ZERO, color_to_rgb(self.palette[color_index]));
                if Some(color_index) == self.edit_color_index {
                    let stroke = egui::Stroke::new(1.0, color_to_rgb_contrast(self.palette[color_index]));
                    painter.rect_stroke(item_rect, egui::CornerRadius::ZERO, stroke, egui::StrokeKind::Inside);
                }
            }
        }
    }

    pub fn show(&mut self, wc: &mut WindowContext, pal_sprite: &mut PalSprite) -> bool {
        if ! self.open { return false; }

        if AssetEditorBase::show_dialog_window(wc, self.dlg_id, MIN_PALETTE_WIDTH + 100.0, "Edit Palette", |ui, wc| {
            egui::Frame::NONE.outer_margin(24.0).show(ui, |ui| {
                ui.horizontal(|ui| {
                    let button = ui.add(egui::Button::new("Load Preset..."));
                    egui::Popup::menu(&button).show(|ui| {
                        for preset in PalettePreset::presets_for_depth(self.depth) {
                            if ui.add(egui::Button::new(preset.text()).wrap_mode(egui::TextWrapMode::Extend)).clicked() {
                                preset.load(&mut self.palette);
                            }
                        }
                    });
                });

                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    let (n_colors_x, n_colors_y) = match self.depth {
                        PalSpriteDepth::Bpp1 => (2, 1),
                        PalSpriteDepth::Bpp2 => (4, 1),
                        PalSpriteDepth::Bpp4 => (8, 2),
                    };
                    let min_size = egui::Vec2::new(MIN_PALETTE_WIDTH, MIN_PALETTE_WIDTH / 4.0);
                    let (response, painter) = ui.allocate_painter(min_size, egui::Sense::click_and_drag());
                    self.draw_palette(&painter, response.rect, (n_colors_x, n_colors_y));

                    if let Some(pos) = response.interact_pointer_pos() && response.rect.contains(pos) {
                        let color_pos = pos - response.rect.min;
                        let x = (color_pos.x * n_colors_x as f32 / response.rect.width()).floor() as i32;
                        let y = (color_pos.y * n_colors_y as f32 / response.rect.height()).floor() as i32;
                        self.edit_color_index = Some((y * n_colors_x + x).clamp(0, self.depth.num_colors() as i32 - 1) as usize);
                    }
                    if wc.vga_bits_per_pixel == 8 &&
                        let Some(edit_color_index) = self.edit_color_index &&
                        let Some(color) = self.palette.get(edit_color_index) {
                            let mut edit_color = *color;
                            self.popup.show(&response, wc.settings, &mut edit_color);
                            if let Some(color) = self.palette.get_mut(edit_color_index) {
                                *color = edit_color;
                            }
                            if self.popup.close {
                                self.edit_color_index = None;
                            }
                        }
                });

                if wc.vga_bits_per_pixel == 6 &&
                    let Some(edit_color_index) = self.edit_color_index &&
                    let Some(color) = self.palette.get(edit_color_index) {
                        let mut rgb = color_to_6bit_rgb(*color);
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut rgb[0]).prefix("R ").speed(0.07).range(0..=3));
                            ui.add(egui::DragValue::new(&mut rgb[1]).prefix("G ").speed(0.07).range(0..=3));
                            ui.add(egui::DragValue::new(&mut rgb[2]).prefix("B ").speed(0.07).range(0..=3));
                        });
                        if let Some(color) = self.palette.get_mut(edit_color_index) {
                            *color = color_6bit_rgb_to_color(rgb[0], rgb[1], rgb[2]);
                        }
                    }

                ui.add_space(24.0);

                egui::Grid::new(format!("editor_panel_{}_pal_edit_grid", pal_sprite.asset.id))
                    .num_columns(2)
                    .spacing([8.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Color depth:");
                        egui::ComboBox::from_id_salt(format!("editor_{}_pal_edit_depth_combo", pal_sprite.asset.id))
                            .selected_text(format!("{} bpp ({} colors)", self.depth.bits_per_pixel(), self.depth.num_colors()))
                            .width(50.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.depth, PalSpriteDepth::Bpp1, "1 bpp (2 colors)");
                                ui.selectable_value(&mut self.depth, PalSpriteDepth::Bpp2, "2 bpp (4 colors)");
                                ui.selectable_value(&mut self.depth, PalSpriteDepth::Bpp4, "4 bpp (16 colors)");
                            });
                        ui.end_row();

                        ui.label("Adjust mode:");
                        egui::ComboBox::from_id_salt(format!("editor_{}_pal_edit_ajust_colors", pal_sprite.asset.id))
                            .selected_text(self.set_pal_options.text())
                            .width(50.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.set_pal_options,
                                    SetPaletteOptions::ChangePaletteColors,
                                    SetPaletteOptions::ChangePaletteColors.text()
                                );
                                ui.selectable_value(
                                    &mut self.set_pal_options,
                                    SetPaletteOptions::AdaptImageColors,
                                    SetPaletteOptions::AdaptImageColors.text()
                                );
                            });
                        ui.end_row();
                    });
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if ui.button("Cancel").clicked() {
                    ui.close();
                }
                if ui.button("Ok").clicked() {
                    self.confirm(pal_sprite);
                    ui.close();
                }
            });
        }).should_close() {
            self.open = false;
            wc.set_dialog_open(self.dlg_id, self.open);
        }
        if self.image_changed {
            self.image_changed = false;
            true
        } else {
            false
        }
    }
}
