use crate::image::{
    colors,
    ImageCollection,
    TextureSlot,
    DynamicImage,
};

use super::{
    GameRunnerWidget,
    WindowContext,
    Direction,
    DrawInfo,
    GameAssets,
    Player,
};

pub struct DarknessState {
    pub enabled: bool,
    pub fall_off: u8,

    pub radius: i32,           // 24.8 fixpoint
    pub offset_x: i16,         // 8.8 fixpoint
    pub offset_y: i16,         // 8.8 fixpoint

    pub control_enabled: bool,
    pub delta_radius: i32,     // 24.8 fixpoint
    pub target_radius: i32,    // 24.8 fixpoint
    pub delta_offset_x: i16,   // 8.8 fixpoint
    pub delta_offset_y: i16,   // 8.8 fixpoint
    pub target_offset_x: i16,  // 8.8 fixpoint
    pub target_offset_y: i16,  // 8.8 fixpoint

    pub shape_changed: bool,
    pub shape: DynamicImage,
}

impl DarknessState {
    const MAX_RADIUS: u32 = 320;

    pub fn new() -> Self {
        DarknessState {
            enabled: false,
            control_enabled: false,
            fall_off: 0,

            radius: 0,
            offset_x: 0,
            offset_y: 0,

            delta_radius: 0,
            target_radius: 0,
            delta_offset_x: 0,
            delta_offset_y: 0,
            target_offset_x: 0,
            target_offset_y: 0,

            shape_changed: false,
            shape: DynamicImage::new(2*Self::MAX_RADIUS+1, 2*Self::MAX_RADIUS+1, 1),
        }
    }

    pub fn tick_engine(&mut self) {
        if ! self.control_enabled {
            return;
        }
        if ! self.enabled || (self.delta_radius == 0 && self.delta_offset_x == 0 && self.delta_offset_y == 0) {
            self.control_enabled = false;
            return;
        }

        if self.delta_radius != 0 {
            self.radius += self.delta_radius;
            if (self.delta_radius > 0 && self.radius >= self.target_radius) ||
                (self.delta_radius < 0 && self.radius <= self.target_radius) {
                    self.radius = self.target_radius;
                    self.delta_radius = 0;
                }
            if self.radius>>8 >= GameRunnerWidget::WIDTH {
                self.enabled = false;
            } else {
                self.load_circle(self.radius >> 8);
            }
        }
        if self.delta_offset_x != 0 {
            self.offset_x += self.delta_offset_x;
            if (self.delta_offset_x > 0 && self.offset_x >= self.target_offset_x) ||
                (self.delta_offset_x < 0 && self.offset_x <= self.target_offset_x) {
                    self.offset_x = self.target_offset_x;
                    self.delta_offset_x = 0;
                }
        }
        if self.delta_offset_y != 0 {
            self.offset_y += self.delta_offset_y;
            if (self.delta_offset_y > 0 && self.offset_y >= self.target_offset_y) ||
                (self.delta_offset_y < 0 && self.offset_y <= self.target_offset_y) {
                    self.offset_y = self.target_offset_y;
                    self.delta_offset_y = 0;
                }
        }
    }

    fn clear_row(&mut self, y: i32, xs: i32, xe: i32) {
        if y < 0 || y >= self.shape.pixels.height as i32 || xe < 0 || xs > self.shape.pixels.width as i32 { return; }
        let width = self.shape.pixels.width as usize;
        let start = (y as usize * width) + xs.max(0) as usize;
        let end = (y as usize * width) + xe.min(self.shape.pixels.width as i32) as usize;
        self.shape.pixels.data[start..=end].fill(colors::TRANSPARENT);
    }

    fn load_circle(&mut self, radius: i32) {
        if radius <= 0 { return; }
        let radius = radius.clamp(0, Self::MAX_RADIUS as i32);
        self.radius = radius << 8;

        self.shape_changed = true;
        self.shape.pixels.width = (2*radius + 1) as u32;
        self.shape.pixels.height = (2*radius + 1) as u32;
        self.shape.pixels.data[0..(self.shape.pixels.width*self.shape.pixels.height) as usize].fill(0);

        let center = radius;
        let mut x = radius;
        let mut y = 0;
        let mut t1 = radius / 16;

        while x >= y {
            if y <= center {
                self.clear_row(center - y, center - x, center + x);
                self.clear_row(center + y, center - x, center + x);
            }
            if x <= center {
                self.clear_row(center - x, center - y, center + y);
                self.clear_row(center + x, center - y, center + y);
            }

            y += 1;
            t1 += y;
            let t2 = t1 - x;
            if t2 >= 0 {
                t1 = t2;
                x -= 1;
            }
        }
    }

    pub fn dispel(&mut self, speed: u16) {
        if ! self.enabled { return; }
        self.delta_radius = speed as i32;
        self.target_radius = 320<<8;
        if ! self.control_enabled {
            self.delta_offset_x = 0;
            self.delta_offset_y = 0;
        }
        self.control_enabled = true;
    }

    pub fn set_at_player(&mut self, radius: u16, fall_off: u8, off_x: i8, off_y: i8) {
        self.fall_off = fall_off;
        self.radius = (radius as i32) << 8;
        self.offset_x = (off_x as i16) << 8;
        self.offset_y = (off_y as i16) << 8;

        self.load_circle(self.radius >> 8);
        self.control_enabled = false;
        self.enabled = true;
    }

    pub fn move_at_player(&mut self, off_x: i8, off_y: i8, speed: u16) {
        if ! self.enabled { return; }
        let off_x = (off_x as i16) << 8;
        let off_y = (off_y as i16) << 8;
        let speed = speed as i16;

        self.target_offset_x = off_x;
        self.target_offset_y = off_y;
        self.delta_offset_x = (off_x - self.offset_x).signum() * speed;
        self.delta_offset_y = (off_y - self.offset_y).signum() * speed;
        if ! self.control_enabled {
            self.delta_radius = 0;
        }
        self.control_enabled = true;
    }

    fn dark_rect(
        ui: &mut egui::Ui,
        draw_info: &DrawInfo,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
    ) {
        let rect = egui::Rect::from_min_max(
            draw_info.pos + draw_info.zoom * egui::Vec2::new(x1 as f32, y1 as f32),
            draw_info.pos + draw_info.zoom * egui::Vec2::new(x2 as f32, y2 as f32),
        );
        ui.painter().rect_filled(rect, egui::CornerRadius::ZERO, egui::Color32::from_rgb(0, 0, 0));
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        assets: &GameAssets,
        player: &Player,
        draw_info: &DrawInfo,
    ) {
        let image_size = egui::Vec2::new(self.shape.pixels.width as f32, self.shape.pixels.height as f32);
        let image_uv = self.shape.get_item_uv(0);

        let clip = &assets.char_anim.clip_rect;
        let player_x = match player.direction {
            Direction::Left  => { player.x - (assets.char_sprite.width as i32 - (clip.x + clip.w)) }
            Direction::Right => { player.x - clip.x }
        };
        let player_y = player.y - clip.y;
        let player_cx = player_x + assets.char_sprite.width as i32 / 2 + (self.offset_x>>8) as i32 * player.direction.dx();
        let player_cy = player_y + assets.char_sprite.height as i32 / 2 + (self.offset_y>>8) as i32;

        let draw_x = player_cx - self.shape.pixels.width as i32 / 2;
        let draw_y = player_cy - self.shape.pixels.height as i32 / 2;
        let draw_rect = egui::Rect::from_min_size(
            draw_info.pos + draw_info.zoom * egui::Vec2::new(draw_x as f32, draw_y as f32),
            draw_info.zoom * image_size
        );

        if self.shape_changed {
            self.shape_changed = false;
            self.shape.load_texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent, true);
        }
        let texture = self.shape.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
        egui::Image::from_texture((texture.id(), image_size)).uv(image_uv).paint_at(ui, draw_rect);

        const WIDTH: i32 = GameRunnerWidget::WIDTH;
        const HEIGHT: i32 = GameRunnerWidget::HEIGHT;
        let image_w = self.shape.pixels.width as i32;
        let image_h = self.shape.pixels.height as i32;
        let screen_x = draw_info.room_pos.0;
        let screen_y = draw_info.room_pos.1;
        Self::dark_rect(ui, draw_info, screen_x, screen_y, screen_x + WIDTH, draw_y);
        Self::dark_rect(ui, draw_info, screen_x, draw_y, draw_x, draw_y + image_h);
        Self::dark_rect(ui, draw_info, draw_x + image_w, draw_y, screen_x + WIDTH, draw_y + image_h);
        Self::dark_rect(ui, draw_info, screen_x, draw_y + image_h, screen_x + WIDTH, screen_y + HEIGHT);
    }
}
