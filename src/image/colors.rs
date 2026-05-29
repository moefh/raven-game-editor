
pub const RED   : u8 = 0b00_000_111;
pub const GREEN : u8 = 0b00_111_000;
pub const BLUE  : u8 = 0b11_000_000;

pub const TRANSPARENT: u8 = GREEN;

pub fn color_to_rgb(color: u8) -> egui::Color32 {
    let r = color & 0x7;
    let g = (color >> 3) & 0x7;
    let b = (color >> 6) & 0x3;
    let cr = (r << 5) | (r << 2) | (r >> 2);
    let cg = (g << 5) | (g << 2) | (g >> 2);
    let cb = (b << 6) | (b << 4) | (b << 2) | b;
    egui::Color32::from_rgb(cr, cg, cb)
}

pub fn color_to_rgb_contrast(color: u8) -> egui::Color32 {
    let r = (color & 0x7) as f32;
    let g = ((color >> 3) & 0x7) as f32;
    let b = (((color >> 5) & 0x6) | (color >> 7)) as f32;
    let brightness = r*0.3 + g*0.8 + b*0.1;
    if brightness < 4.5 {
        egui::Color32::WHITE
    } else {
        egui::Color32::BLACK
    }
}
