use egui::{
    Pos2,
    Vec2,
};

use crate::data_store::{
    DataStore,
    Tileset,
    RoomTriggerType,
};
use crate::image::{
    ImageCollection,
    TextureSlot,
};

use super::{
    WindowContext,
    Player,
    GameAssets,
    GameRunnerState,
    CollisionRect,
    DrawInfo,
};
use super::super::super::editors::{
    FontPainter,
};

struct TriggerPromptInfo {
    trigger_rect: CollisionRect,
    interact_prompt_id: u16,
    screen_pos: Pos2,
    zoom: f32,
}

fn extract_trigger_prompt_button(prompt: &str) -> (Option<u32>, &str) {
    let mut bytes = prompt.bytes();
    if let Some(prefix) = bytes.next() && prefix == b'!' && let Some(button) = bytes.next() && button < 128 {
        let btn = match button {
            b'B' | b'b' => { 0 }
            b'A' | b'a' => { 1 }
            b'Y' | b'y' => { 2 }
            b'X' | b'x' => { 3 }
            _ => { 0 }
        };
        if let Some(space) = bytes.next() && space == b' ' {
            (Some(btn), &prompt[3..])
        } else {
            (Some(btn), &prompt[2..])
        }
    } else {
        (None, prompt)
    }
}

fn draw_trigger_prompt(
    ui: &mut egui::Ui,
    wc: &mut WindowContext,
    prompt_info: TriggerPromptInfo,
    player: &Player,
    game_assets: &GameAssets,
    store: &DataStore,
) {
    let player_anim = game_assets.char_anim;
    let player_rect = CollisionRect::new(player.x, player.y, player_anim.clip_rect.w, player_anim.clip_rect.h);
    if player_rect.intersects_rect(&prompt_info.trigger_rect) &&
        let Some(prompt) = store.tables.interact_prompt.items.get(prompt_info.interact_prompt_id as usize) {
            let font = game_assets.font;
            let zoom = prompt_info.zoom;
            let screen_pos = prompt_info.screen_pos;
            let font_height = font.height as f32 * zoom;
            let font_width = font.measure(font_height, &prompt.value);
            let pos = screen_pos + Vec2::new(
                zoom * (prompt_info.trigger_rect.x as f32 + 0.5 * prompt_info.trigger_rect.w as f32) - 0.5 * font_width,
                zoom * (prompt_info.trigger_rect.y as f32 - 0.5 * Tileset::TILE_SIZE as f32),
            );

            let (btn_index, prompt_string) = extract_trigger_prompt_button(&prompt.value);
            let text_pos = if let Some(btn) = btn_index {
                let sprite = game_assets.gamepad_sprite;
                let sprite_offset = zoom * 0.5 * Vec2::new(
                    sprite.width as f32 + 6.0,
                    sprite.height as f32 - font.height as f32,
                );

                let sprite_size = Vec2::new(sprite.width as f32, sprite.height as f32);
                let sprite_uv = sprite.get_item_uv(btn);
                let draw_rect = egui::Rect::from_min_size(
                    pos - sprite_offset,
                    zoom * sprite_size,
                );

                let texture = game_assets.gamepad_sprite.texture(wc.tex_man, wc.egui.ctx, TextureSlot::Transparent);
                egui::Image::from_texture((texture.id(), sprite_size)).uv(sprite_uv).paint_at(ui, draw_rect);

                pos + sprite_offset
            } else {
                pos
            };

            for y in [-1, 0, 1] {
                for x in [-1, 0, 1] {
                    if x != 0 || y != 0 {
                        let pos = text_pos + Vec2::new(x as f32 * zoom, y as f32 * zoom);
                        font.paint_string(ui, wc, prompt_string, pos, font_height, egui::Color32::WHITE);
                    }
                }
            }
            font.paint_string(ui, wc, prompt_string, text_pos, font_height, egui::Color32::BLUE);
        }
}

pub fn draw_room_trigger_prompts(
    ui: &mut egui::Ui,
    wc: &mut WindowContext,
    game_state: &GameRunnerState,
    game_assets: &GameAssets,
    player: &Player,
    store: &DataStore,
    draw_info: &DrawInfo,
) {
    for trigger in &game_assets.room.triggers {
        match trigger.trigger_type {
            RoomTriggerType::WallButton { width, height, interact_prompt_id, set_effect_id, .. } => {
                if ! game_state.is_effect_activated(set_effect_id) {
                    let prompt_info = TriggerPromptInfo {
                        trigger_rect: CollisionRect {
                            x: trigger.x as i32,
                            y: trigger.y as i32,
                            w: width as i32,
                            h: height as i32,
                        },
                        interact_prompt_id,
                        screen_pos: draw_info.pos,
                        zoom: draw_info.zoom,
                    };
                    draw_trigger_prompt(ui, wc, prompt_info, player, game_assets, store);
                }
            }

            RoomTriggerType::GetCollectable { interact_prompt_id, get_collectable_id, req_flag_id, .. } => {
                let has_flag_requirements = req_flag_id == u16::MAX || game_state.is_flag_set(req_flag_id);
                if has_flag_requirements &&
                    ! game_state.acquired_collectable_ids.contains(&get_collectable_id) &&
                    let Some(item) = store.tables.collectable.items.get(get_collectable_id as usize) &&
                    let Some(sprite) = store.assets.sprites.get(&item.sprite_id) {
                        let prompt_info = TriggerPromptInfo {
                            trigger_rect: CollisionRect {
                                x: trigger.x as i32,
                                y: trigger.y as i32,
                                w: sprite.width as i32,
                                h: sprite.height as i32,
                            },
                            interact_prompt_id,
                            screen_pos: draw_info.pos,
                            zoom: draw_info.zoom,
                        };
                        draw_trigger_prompt(ui, wc, prompt_info, player, game_assets, store);
                    }
            }

            _ => {}
        }
    }
}
