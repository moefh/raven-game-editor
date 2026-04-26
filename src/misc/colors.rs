
pub const RED   : u8 = 0b00000111;
pub const GREEN : u8 = 0b00111000;
pub const BLUE  : u8 = 0b11000000;

pub fn raven_color_to_rgb(raven_color: u8) -> egui::Color32 {
    let r = raven_color & 0x7;
    let g = (raven_color >> 3) & 0x7;
    let b = (raven_color >> 6) & 0x3;
    let cr = (r << 5) | (r << 2) | (r >> 2);
    let cg = (g << 5) | (g << 2) | (g >> 2);
    let cb = (b << 6) | (b << 4) | (b << 2) | b;
    egui::Color32::from_rgb(cr, cg, cb)
}
