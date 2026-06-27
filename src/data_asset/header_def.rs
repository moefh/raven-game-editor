const HEADER_TEMPLATE: &str = r###"
#ifndef $<PREFIX>_DATA_H_FILE
#define $<PREFIX>_DATA_H_FILE

#include <stddef.h>
#include <stdint.h>

#ifndef $<PREFIX>_SKIP_STRUCTS_MOD

struct $<PREFIX>_MOD_SAMPLE {
    uint32_t len;
    uint32_t loop_start;
    uint32_t loop_len;
    uint8_t  finetune;
    uint8_t  volume;
    uint16_t bits_per_sample;
    union {
        const void *data;
        const int8_t *data8;
        const int16_t *data16;
    };
};

struct $<PREFIX>_MOD_CELL {
    uint8_t  sample;
    uint8_t  note_index;
    uint16_t effect;
};

struct $<PREFIX>_MOD_DATA {
    struct $<PREFIX>_MOD_SAMPLE samples[31];
    uint8_t num_channels;

    uint8_t num_song_positions;
    uint8_t song_positions[128];

    uint8_t num_patterns;
    const struct $<PREFIX>_MOD_CELL *pattern;
};

#endif /* $<PREFIX>_SKIP_STRUCTS_MOD */

#ifndef $<PREFIX>_SKIP_STRUCTS_SFX

struct $<PREFIX>_SFX {
    int32_t len;
    int32_t loop_start;
    int32_t loop_len;
    int32_t bits_per_sample;
    union {
        const void *samples;
        const int8_t *spl8;
        const int16_t *spl16;
    };
};

#endif /* $<PREFIX>_SKIP_STRUCTS_SFX */

#ifndef $<PREFIX>_SKIP_STRUCTS_IMAGE

struct $<PREFIX>_IMAGE {
    int32_t width;
    int32_t height;
    int32_t stride;
    int32_t num_frames;
    const uint32_t *data;
};

#endif /* $<PREFIX>_SKIP_STRUCTS_IMAGE */

#ifndef $<PREFIX>_SKIP_STRUCTS_PAL_SPRITE

struct $<PREFIX>_PAL_SPRITE {
    int16_t width;
    int16_t height;
    int16_t num_frames;
    uint16_t bpp;
    uint8_t palette[16];
    const uint8_t *data;
};

#endif /* $<PREFIX>_SKIP_STRUCTS_PAL_SPRITE */

#ifndef $<PREFIX>_SKIP_STRUCTS_MAP

struct $<PREFIX>_MAP {
    int16_t w;
    int16_t h;
    int16_t para_w;
    int16_t para_h;
    const struct $<PREFIX>_IMAGE *tileset;
    const uint8_t *tiles;
};

#endif /* $<PREFIX>_SKIP_STRUCTS_MAP */

#ifndef $<PREFIX>_SKIP_STRUCTS_SPRITE_ANIMATION

struct $<PREFIX>_SPRITE_ANIMATION_LOOP {
    uint16_t offset;   // offset into animation frame_indices
    uint16_t length;   // loop data length
};

struct $<PREFIX>_SPRITE_ANIMATION_COLLISION {
    uint16_t x;
    uint16_t y;
    uint16_t w;
    uint16_t h;
};

struct $<PREFIX>_SPRITE_ANIMATION {
    const uint8_t *frame_indices;
    const struct $<PREFIX>_IMAGE *sprite;
    struct $<PREFIX>_SPRITE_ANIMATION_COLLISION collision;
    int8_t use_foot_frames;
    int8_t foot_overlap;
    struct $<PREFIX>_SPRITE_ANIMATION_LOOP loops[20];
};

#endif /* $<PREFIX>_SKIP_STRUCTS_SPRITE_ANIMATION */

#ifndef $<PREFIX>_SKIP_STRUCTS_FONT

struct $<PREFIX>_FONT {
    uint8_t width;
    uint8_t height;
    const uint8_t *data;
};

#endif /* $<PREFIX>_SKIP_STRUCTS_FONT */

#ifndef $<PREFIX>_SKIP_STRUCTS_PROP_FONT

struct $<PREFIX>_PROP_FONT {
    uint8_t height;
    const uint8_t *data;
    uint8_t char_width[96];
    uint16_t char_offset[96];
};

#endif /* $<PREFIX>_SKIP_STRUCTS_PROP_FONT */

#ifndef $<PREFIX>_SKIP_STRUCTS_ROOM

enum $<PREFIX>_ROOM_TRIGGER_TYPE {
   $<PREFIX>_ROOM_TRIGGER_TYPE_UNKNOWN,
   $<PREFIX>_ROOM_TRIGGER_TYPE_DOOR,
   $<PREFIX>_ROOM_TRIGGER_TYPE_PLAYER_SPAWN,
   $<PREFIX>_ROOM_TRIGGER_TYPE_ENEMY_SPAWN,
   $<PREFIX>_ROOM_TRIGGER_TYPE_TRAP,
};

struct $<PREFIX>_ROOM_MAP_INFO {
    uint16_t x;
    uint16_t y;
    const struct $<PREFIX>_MAP *map;
};

struct $<PREFIX>_ROOM_TRIGGER_INFO {
    enum $<PREFIX>_ROOM_TRIGGER_TYPE type;
    int16_t x;
    int16_t y;
    union {
        struct {
            uint32_t data0;
            uint32_t data1;
            uint32_t data2;
            uint32_t data3;
        } any;
        struct {
            uint8_t direction;
        } player_spawn;
        struct {
             const struct $<PREFIX>_ROOM *room;
             uint16_t door;
        } door;
        struct {
             const struct $<PREFIX>_SPRITE_ANIMATION *animation;
        } enemy_spawn;
        struct {
             uint16_t width;
             uint16_t height;
             uint16_t type;
        } trap;
    };
};

struct $<PREFIX>_ROOM {
    uint16_t num_maps;
    uint16_t num_triggers;
    const struct $<PREFIX>_ROOM_MAP_INFO *maps;
    const struct $<PREFIX>_ROOM_TRIGGER_INFO *triggers;
};

#endif /* $<PREFIX>_SKIP_STRUCTS_ROOM */

#ifndef $<PREFIX>_SKIP_STRUCTS_WORLD

struct $<PREFIX>_WORLD_REGION {
    uint8_t x;
    uint8_t y;
    uint8_t width;
    uint8_t height;
    uint8_t *blocks;
    uint16_t *room_indices;
    uint32_t *block_bitmap;
};

struct $<PREFIX>_WORLD {
    uint16_t num_regions;
    const struct $<PREFIX>_WORLD_REGION *regions;
};

#endif /* $<PREFIX>_SKIP_STRUCTS_WORLD */

#ifndef $<PREFIX>_SKIP_ROOM_SCRIPT

struct $<PREFIX>_STATE;
typedef void (*$<prefix>_room_init_function)(uint32_t, struct $<PREFIX>_STATE *);

struct $<PREFIX>_ROOM_SCRIPT {
    $<prefix>_room_init_function init;
};

#endif /* $<PREFIX>_SKIP_ROOM_SCRIPT */

extern const struct $<PREFIX>_FONT $<prefix>_fonts[];
extern const struct $<PREFIX>_PROP_FONT $<prefix>_prop_fonts[];
extern const struct $<PREFIX>_MOD_DATA $<prefix>_mods[];
extern const struct $<PREFIX>_SFX $<prefix>_sfxs[];
extern const struct $<PREFIX>_IMAGE $<prefix>_tilesets[];
extern const struct $<PREFIX>_IMAGE $<prefix>_sprites[];
extern const struct $<PREFIX>_MAP $<prefix>_maps[];
extern const struct $<PREFIX>_SPRITE_ANIMATION $<prefix>_sprite_animations[];
extern const struct $<PREFIX>_ROOM $<prefix>_rooms[];

#if $<PREFIX>_ADD_ROOM_SCRIPTS
extern const struct $<PREFIX>_ROOM_SCRIPT *$<prefix>_room_script_table[];
#endif /* $<PREFIX>_ADD_ROOM_SCRIPTS */

#endif /* $<PREFIX>_DATA_H_FILE */
"###;

use std::sync::LazyLock;
use std::collections::HashMap;

#[derive(Default)]
pub struct VarReplacer {
    vars: HashMap<String, String>,
    default_value: String,
}

impl VarReplacer {
    pub fn new() -> Self {
        VarReplacer {
            vars: HashMap::new(),
            default_value: String::new(),
        }
    }

    pub fn add_var(&mut self, name: impl AsRef<str>, value: String) {
        self.vars.insert(name.as_ref().to_owned(), value);
    }

    pub fn replace(&self, template: &str) -> String {
        static RE_VAR: LazyLock<regex::Regex> = LazyLock::new(|| regex::Regex::new(r"\$<([A-Za-z0-9_]+)>").unwrap());
        RE_VAR.replace_all(template, self).into_owned()
    }
}

impl regex::Replacer for &VarReplacer {
    fn replace_append(&mut self, caps: &regex::Captures<'_>, dst: &mut String) {
        dst.push_str(self.vars.get(&caps[1]).unwrap_or(&self.default_value));
    }
}

pub fn write_header_def(filename: impl AsRef<std::path::Path>, prefix: &str) -> std::io::Result<()> {
    let mut repl = VarReplacer::new();

    let mut prefix_lower = String::from(prefix);
    prefix_lower.make_ascii_lowercase();
    repl.add_var("prefix", prefix_lower);

    let mut prefix_upper = String::from(prefix);
    prefix_upper.make_ascii_uppercase();
    repl.add_var("PREFIX", prefix_upper);

    let header = repl.replace(&HEADER_TEMPLATE[1..]);  // remove initial newline
    std::fs::write(filename, &header)
}
