use std::io::{Result, Error};
use std::path::Path;

const WAV_FORMAT_PCM: u16 = 1;

pub struct WavFile {
    pub channels: Vec<Vec<i16>>,
    pub bits_per_sample: u16,
    #[allow(dead_code)]
    pub sample_rate: u32,
}

impl WavFile {

    fn read_24bit_sample(r: &mut super::reader::Reader) -> Result<i16> {
        r.read_u8()?; // ignore low 8 bits
        Ok(r.read_u16_le()? as i16)
    }

    fn read_32bit_sample(r: &mut super::reader::Reader) -> Result<i16> {
        let val = r.read_f32_le()?;
        Ok((val * 32767.0).clamp(i16::MIN as f32, i16::MAX as f32) as i16)
    }

    pub fn read(filename: &Path) -> Result<Self> {
        let data = std::fs::read(filename)?;

        let mut r = super::reader::Reader::new(&data);

        if r.read_byte_vec(4)? != b"RIFF" { return Err(Error::other("invalid file format")); }
        r.read_u32_le()?;  // file size - 8
        if r.read_byte_vec(4)? != b"WAVE" { return Err(Error::other("invalid wave header")); }

        let mut num_channels = 0;
        let mut sample_rate = 0;
        let mut bits_per_sample = 0;
        loop {
            let mut tag = [0; 4];
            r.read_bytes(&mut tag)?;

            match &tag {
                b"fmt " => {
                    let chunk_size = r.read_u32_le()?;
                    if chunk_size < 0x10 { return Err(Error::other("invalid file format: fmt chunk size too small")); }

                    let format = r.read_u16_le()?;
                    num_channels = r.read_u16_le()?;
                    sample_rate = r.read_u32_le()?;
                    r.read_u32_le()?;  // bytes/second
                    r.read_u16_le()?;  // bytes/block
                    bits_per_sample = r.read_u16_le()?;

                    if format != WAV_FORMAT_PCM {
                        return Err(Error::other(format!("unsupported WAV format: {}", format)));
                    }

                    if chunk_size > 0x10 {
                        r.skip((chunk_size - 0x10) as usize)?;
                    }
                }

                b"data" => {
                    if num_channels == 0 || bits_per_sample == 0 { return Err(Error::other("invalid WAV format: no fmt chunk")); }
                    let chunk_size = r.read_u32_le()?;

                    let num_samples = (chunk_size / (num_channels * (bits_per_sample / 8)) as u32) as usize;
                    let mut channels = Vec::new();
                    for _ in 0..num_channels {
                        channels.push(Vec::new());
                    }
                    for _ in 0..num_samples {
                        for ch in channels.iter_mut() {
                            ch.push(match bits_per_sample {
                                8 => (r.read_u8()? as i16 - 128) << 8,
                                16 => r.read_u16_le()? as i16,
                                24 => Self::read_24bit_sample(&mut r)?,
                                32 => Self::read_32bit_sample(&mut r)?,
                                _ => { return Err(Error::other(format!("unsupported WAV: {} bits per sample", bits_per_sample))); }
                            });
                        }
                    }

                    return Ok(WavFile {
                        channels,
                        bits_per_sample,
                        sample_rate,
                    });
                }

                _ => {
                    let chunk_size = r.read_u32_le()?;
                    r.skip(chunk_size as usize)?;
                }
            };
        }
    }

    pub fn write(filename: &Path, sample_rate: u32, bits_per_sample: u16, samples: &[i16]) -> Result<()> {
        if bits_per_sample != 8 && bits_per_sample != 16 {
            return Err(Error::other(format!("unsupported bits per sample value: {}", bits_per_sample)));
        }

        let num_channels = 1u16;
        let bytes_per_sample = bits_per_sample / 8;
        let file_size = 44 + bytes_per_sample as u32 * samples.len() as u32;

        let mut w = super::writer::Writer::new();
        w.write_bytes(b"RIFF");
        w.write_u32_le(file_size - 8);
        w.write_bytes(b"WAVE");

        w.write_bytes(b"fmt ");
        w.write_u32_le(0x10);
        w.write_u16_le(WAV_FORMAT_PCM);
        w.write_u16_le(num_channels);
        w.write_u32_le(sample_rate);
        w.write_u32_le(sample_rate * (num_channels * bytes_per_sample) as u32);
        w.write_u16_le(num_channels * bytes_per_sample);
        w.write_u16_le(bits_per_sample);

        w.write_bytes(b"data");
        w.write_u32_le(bytes_per_sample as u32 * samples.len() as u32);

        for &sample in samples {
            if bits_per_sample == 16 {
                w.write_u16_le(sample as u16);
            } else {
                w.write_u8(((sample >> 8) + 128) as u8);
            }
        }

        std::fs::write(filename, &w.data)
    }
}
