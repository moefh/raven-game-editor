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

#ifndef $<PREFIX>_SKIP_STRUCTS_MAP

struct $<PREFIX>_MAP {
    int16_t w;
    int16_t h;
    int16_t bg_w;
    int16_t bg_h;
    const struct $<PREFIX>_IMAGE *tileset;
    const uint8_t *tiles;
};

#endif /* $<PREFIX>_SKIP_STRUCTS_MAP */

#ifndef $<PREFIX>_SKIP_STRUCT_SPRITE_ANIMATION

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

struct $<PREFIX>_ROOM_MAP_INFO {
    uint16_t x;
    uint16_t y;
    const struct $<PREFIX>_MAP *map;
};

struct $<PREFIX>_ROOM_ENTITY_INFO {
    int16_t x;
    int16_t y;
    const struct $<PREFIX>_SPRITE_ANIMATION *anim;
    uint16_t data0;
    uint16_t data1;
    uint16_t data2;
    uint16_t data3;
};

struct $<PREFIX>_ROOM_TRIGGER_INFO {
    int16_t x;
    int16_t y;
    uint16_t w;
    uint16_t h;
    uint16_t data0;
    uint16_t data1;
    uint16_t data2;
    uint16_t data3;
};

struct $<PREFIX>_ROOM {
    uint8_t num_maps;
    uint8_t num_entities;
    uint8_t num_triggers;
    const struct $<PREFIX>_ROOM_MAP_INFO *maps;
    const struct $<PREFIX>_ROOM_ENTITY_INFO *entities;
    const struct $<PREFIX>_ROOM_TRIGGER_INFO *triggers;
};

#endif /* $<PREFIX>_SKIP_STRUCTS_ROOM */

extern const struct $<PREFIX>_FONT $<prefix>_fonts[];
extern const struct $<PREFIX>_PROP_FONT $<prefix>_prop_fonts[];
extern const struct $<PREFIX>_MOD_DATA $<prefix>_mods[];
extern const struct $<PREFIX>_SFX $<prefix>_sfxs[];
extern const struct $<PREFIX>_IMAGE $<prefix>_tilesets[];
extern const struct $<PREFIX>_IMAGE $<prefix>_sprites[];
extern const struct $<PREFIX>_MAP $<prefix>_maps[];
extern const struct $<PREFIX>_SPRITE_ANIMATION $<prefix>_sprite_animations[];
extern const struct $<PREFIX>_ROOM $<prefix>_rooms[];

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
