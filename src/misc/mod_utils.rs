
pub const NOTE_NAMES: &[&str] = &[ "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B" ];
pub const WAVEFORM_NAMES: &[&str] = &[ "sine", "ramp down", "square", "random" ];

const PAL_MAGIC: f64 = 7093789.2;
//const NTSC_MAGIC: f64 = 7159090.5;

pub fn get_period_sample_rate(period: u16) -> Option<f32> {
    if period == 0 { return None; }
    Some((PAL_MAGIC / (2.0*period as f64)) as f32)
}

pub fn get_note_name(note: i32) -> &'static str {
    NOTE_NAMES[(note as usize) % NOTE_NAMES.len()]
}

pub fn get_effect_description(effect: u16, note: i32, song_positions: &[u8]) -> Option<String> {
    let x = ((effect & 0xf0) >> 4) as i32;
    let y = ( effect & 0x0f      ) as i32;
    let xy = (effect & 0x0ff) as i32;
    match effect >> 8 {
        0x0 => {
            Some(format!("arpeggio of ({} {} {})", get_note_name(note),
                         get_note_name((note+x)%12), get_note_name((note+y)%12)))
        },
        0x1 => Some(format!("slide up by {} periods per tick", xy)),
        0x2 => Some(format!("slide down by {} periods per tick", xy)),
        0x3 => Some(format!("slide to note, moving {} periods per tick", xy)),
        0x4 => {
            if x == 0 && y == 0 {
                Some("vibrato with last strength/speed".to_string())
            } else if x == 0 && y != 0 {
                Some(format!("vibrato with strength {} and last speed", y))
            } else if x != 0 && y == 0 {
                Some(format!("vibrato with speed {} and last strength", x))
            } else {
                Some(format!("vibrato with strength {} and speed {}", y, x))
            }
        },
        0x5 => if x != 0 {
            Some(format!("continue slide to note, sliding volume up by {}", x))
        } else {
            Some(format!("continue slide to note, sliding volume down by {}", y))
        },
        0x6 => if x != 0 {
            Some(format!("continue vibrato, sliding volume up by {}", x))
        } else {
            Some(format!("continue vibrato, sliding volume down by {}", y))
        },
        0x7 => {
            if x == 0 && y == 0 {
                Some("tremolo with last strength/speed".to_string())
            } else if x == 0 && y != 0 {
                Some(format!("tremolo with strength {} and last speed", y))
            } else if x != 0 && y == 0 {
                Some(format!("tremolo with speed {} and last strength", x))
            } else {
                Some(format!("tremolo with strength {} and speed {}", y, x))
            }
        },
        0x8 => Some(format!("raven fx8 callback with id {}\n\n(some MODs use this as pan {} {})", xy,
                            if xy < 128 { (128-xy).to_string() } else { (xy-127).to_string() },
                            if xy < 128 { "left" } else { "right" }
        )),
        0x9 => if xy == 0 {
            Some("set sample offset (ignored)".to_string())
        } else {
            Some(format!("set sample offset (start at offset {})", xy*0x100))
        },
        0xA => if x != 0 {
            Some(format!("volume slide up by {}", x))
        } else {
            Some(format!("volume slide down by {}", y))
        },
        0xB => Some(format!("jump to song position index {} (pattern {})", xy,
                            if xy >= 0 && (xy as usize) < song_positions.len() { song_positions[xy as usize] } else { 0 })),
        0xC => Some(format!("set volume to {:#04x} ({}%)", xy, ((xy as f32 * 100.0) / 64.0).round())),
        0xD => Some(format!("skip to next pattern at row {}", x*10 + y)),
        0xE => {
            match (effect >> 4) & 0xf {
                0x0 => Some(format!("set filter {}", if xy == 0 { "on" } else { "off" })),  // yes, 0 means "on" here
                0x1 => Some(format!("fineslide up by {} periods per tick", xy)),
                0x2 => Some(format!("fineslide down by {} periods per tick", xy)),
                0x3 => Some(format!("set glissando {}", if xy == 0 { "off" } else { "on" })),
                0x4 => Some(format!("set vibrato waveform to {} {} retrigger",
                                    WAVEFORM_NAMES[(y&0x3) as usize],
                                    if (y&0x4) == 0 { "with" } else { "with no" }       // yes, 0 means "on" here too
                )),
                0x5 => Some(format!("set finetune to {}", if y < 8 { y } else { y - 0x10 })),
                0x6 => if y == 0 {
                    Some("set loop start".to_string())
                } else {
                    Some(format!("loop from start marker {} times", y))
                },
                0x7 => Some(format!("set tremolo waveform to {} {} retrigger",
                                    WAVEFORM_NAMES[(y&0x3) as usize],
                                    if (y&0x4) == 0 { "with" } else { "with no" }       // yes, 0 means "on" here too
                )),
                0x8 => Some(format!("raven fxF8 callback with id {}\n\n(some MODs use this as pan {} {})", y,
                                    if y < 8 { (8-y).to_string() } else { (y-7).to_string() },
                                    if y < 8 { "left" } else { "right" }
                )),
                0x9 => Some(format!("retrigger sample every {} ticks", y)),
                0xA => Some(format!("fine volume up by {}", y)),
                0xB => Some(format!("fine volume down by {}", y)),
                0xC => Some(format!("cut sample after {} ticks", y)),
                0xD => Some(format!("delay sample start by {} ticks", y)),
                0xE => Some(format!("delay pattern by {} line-lengths", y)),
                0xF => Some("invert loop".to_string()),
                _ => None
            }
        },
        0xF => if xy <= 31 {
            Some(format!("set speed to {} ticks per row", xy))
        } else {
            Some(format!("set speed to {} BPM", xy))
        },
        _ => None,
    }
}
