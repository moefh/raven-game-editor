use std::io::{Result, Error};
use std::path::Path;

use crate::data_asset::{ModData, ModSample, ModCell};

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
        } else if y != 0 {
            Some(format!("continue slide to note, sliding volume down by {}", y))
        } else {
            Some("continue slide to note, sliding volume by the last value".to_owned())
        },
        0x6 => if x != 0 {
            Some(format!("continue vibrato, sliding volume up by {}", x))
        } else if y != 0 {
            Some(format!("continue vibrato, sliding volume down by {}", y))
        } else {
            Some("continue vibrato, sliding volume by the last value".to_owned())
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
            Some("set sample offset (ignored because offset=0)".to_string())
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

#[allow(dead_code)]
pub struct ModFile {
    pub title: [u8; 20],
    pub sample_names: Vec<[u8; 22]>,
    pub id: [u8; 4],

    pub samples: Vec<ModSample>,
    pub pattern: Vec<ModCell>,
    pub song_positions: Vec<u8>,
    pub num_channels: u8,
}

struct WriteModFile<'a> {
    title: &'a [u8],
    sample_names: &'a [[u8; 22]],

    samples: &'a [ModSample],
    pattern: &'a [ModCell],
    song_positions: &'a [u8],
    num_channels: u8,
}

#[allow(dead_code)]
impl ModFile {

    fn display_mod_id(id: &[u8]) -> String {
        let mut s = String::new();
        for b in id {
            if (*b as char).is_ascii_alphanumeric() {
                s.push(*b as char);
            } else {
                s.push_str(&format!("\\x{:02x}", *b));
            }
        }
        s
    }

    pub fn read<P: AsRef<Path>>(filename: P) -> Result<ModFile> {
        let data = std::fs::read(filename)?;

        let mut r = super::reader::Reader::new(&data);

        // read ID for num channels and num samples
        r.seek(1080)?;
        let mut mod_id = [0; 4];
        r.read_bytes(&mut mod_id)?;
        let (num_channels, num_samples) = match &mod_id {
            b"M.K." | b"M!K!" | b"4CHN" | b"FLT4" => {
                (4, 31)
            }

            b"6CHN" => {
                (6, 31)
            }

            b"8CHN" => {
                (8, 31)
            }

            _ => {
                return Err(Error::other(format!("file doesn't look like a MOD file (id='{}')",
                                                Self::display_mod_id(&mod_id))));
            }
        };

        r.seek(0)?;

        // read title, sample names, and sample info
        let title = r.read_array::<20>()?;

        let mut samples = Vec::new();
        let mut sample_names = Vec::new();
        for _ in 0..num_samples {
            let sample_name = r.read_array::<22>()?;
            let len = r.read_u16_be()? as u32 * 2;
            let finetune = r.read_u8()?;
            let volume = r.read_u8()?;
            let loop_start = r.read_u16_be()? as u32 * 2;
            let loop_len = r.read_u16_be()? as u32 * 2;

            sample_names.push(sample_name);
            samples.push(ModSample {
                len,
                loop_start,
                loop_len,
                finetune: if finetune > 7 { finetune as i8 - 16 } else { finetune as i8 },
                volume,
                bits_per_sample: 8,
                data: None,
            });
        }

        // read song positions
        r.seek(950)?;
        let num_song_positions = r.read_u8()?;
        r.read_u8()?;  // ignore (restart song position?)
        let mut song_positions = vec![0; 128];
        r.read_bytes(&mut song_positions[0..128])?;
        song_positions.truncate(num_song_positions as usize);

        // read pattern
        r.seek(1084)?;
        let num_patterns = song_positions.iter().max().unwrap_or(&0) + 1;
        let mut pattern = Vec::new();
        for _pat in 0..num_patterns {
            for _row in 0..64 {
                for _ch in 0..num_channels {
                    let data0 = r.read_u8()? as u16;
                    let data1 = r.read_u8()? as u16;
                    let data2 = r.read_u8()? as u16;
                    let data3 = r.read_u8()? as u16;

                    pattern.push(ModCell {
                        sample: ((data0 & 0xf0) | (data2 >> 4)) as u8,
                        period: ((data0 & 0x0f) << 8) | data1,
                        effect: ((data2 & 0x0f) << 8) | data3,
                    });
                }
            }
        }

        // read sample data
        for sample in samples.iter_mut() {
            if sample.len > 0 {
                let mut data = vec![0; sample.len as usize];
                for spl in data.iter_mut() {
                    let val = r.read_u8()? as u16;
                    *spl = ((val << 8) | val) as i16;
                }
                sample.data = Some(data);
            }
        }

        Ok(ModFile {
            title,
            sample_names,
            id: mod_id,
            samples,
            song_positions,
            num_channels,
            pattern,
        })
    }

    pub fn write_mod_data<P: AsRef<Path>>(filename: P, mod_data: &ModData) -> Result<()> {
        let title = filename.as_ref().file_prefix().map(|f| f.as_encoded_bytes()).unwrap_or_else(|| { &[] });
        Self::write_mod(filename.as_ref(), &WriteModFile {
            title,
            sample_names: &[],
            samples: &mod_data.samples,
            pattern: &mod_data.pattern,
            song_positions: &mod_data.song_positions,
            num_channels: mod_data.num_channels,
        })
    }

    pub fn write_mod_file<P: AsRef<Path>>(filename: P, mod_file: &ModFile) -> Result<()> {
        Self::write_mod(filename.as_ref(), &WriteModFile {
            title: &mod_file.title,
            sample_names: &mod_file.sample_names,
            samples: &mod_file.samples,
            pattern: &mod_file.pattern,
            song_positions: &mod_file.song_positions,
            num_channels: mod_file.num_channels,
        })
    }

    fn write_mod(filename: &Path, mod_file: &WriteModFile) -> Result<()> {
        let mut w = super::writer::Writer::new();

        // title and sample info
        w.write_n_bytes(mod_file.title, b' ', 20);
        for (sample_index, sample) in mod_file.samples.iter().enumerate().take(31) {
            let sample_name = mod_file.sample_names.get(sample_index).unwrap_or(&[b' '; 22]);
            let has_data = sample.data.is_some();

            w.write_n_bytes(sample_name, b' ', 22);
            if sample.len.div_ceil(2) > u16::MAX as u32 {
                return Err(Error::other(format!("sample {} is too long: max len is {}", sample_index+1, u16::MAX as u32 * 2)));
            }
            w.write_u16_be(if has_data { sample.len.div_ceil(2) as u16 } else { 0 });
            w.write_u8(if sample.finetune >= 0 { sample.finetune } else { sample.finetune + 16 } as u8);
            w.write_u8(if sample.volume > 0x3f { 0x3f } else { sample.volume });
            w.write_u16_be(if has_data { sample.loop_start.div_ceil(2) as u16 } else { 0 });
            w.write_u16_be(if has_data { sample.loop_len.div_ceil(2) as u16 } else { 0 });
        }
        for _ in mod_file.samples.len()..31 {
            w.write_n_bytes(&[], b' ', 22);
            w.write_u16_be(0);  // len
            w.write_u8(0);      // finetune
            w.write_u8(0);      // volume
            w.write_u16_be(0);  // loop_start
            w.write_u16_be(0);  // loop_len
        }

        // song positions
        w.write_u8(mod_file.song_positions.len() as u8);
        w.write_u8(0);
        w.write_n_bytes(mod_file.song_positions, 0, 128);

        // tag
        match mod_file.num_channels {
            4 => { w.write_bytes(b"M.K."); }
            6 => { w.write_bytes(b"6CHN"); }
            8 => { w.write_bytes(b"8CHN"); }
            _ => { return Err(Error::other("unsupported number of channels (supported: 4, 6, 8)".to_owned())); }
        };

        // patterns
        for cell in mod_file.pattern.iter() {
            let cell_data = [
                (((cell.period >> 8) & 0x0f) as u8 | (cell.sample        & 0xf0)),
                (cell.period & 0xff) as u8,
                (((cell.effect >> 8) & 0x0f) as u8 | ((cell.sample << 4) & 0xf0)),
                (cell.effect & 0xff) as u8,
            ];
            w.write_bytes(&cell_data);
        }

        // samples
        for sample in mod_file.samples.iter().take(31) {
            if let Some(data) = &sample.data {
                for spl in data.iter() {
                    w.write_u8((spl >> 8) as u8);
                }
                if ! data.len().is_multiple_of(2) {
                    w.write_u8(0);
                }
            }
        }

        std::fs::write(filename, &w.data)
    }
}
