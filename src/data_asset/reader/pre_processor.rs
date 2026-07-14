use std::io::Result;
use std::sync::LazyLock;
use regex::Regex;

use super::{
    StringLogger,
    DataAssetStore,
    ProjectData,
    Tokenizer,
    TokenPosition,
    error,
};

static RE_PRE_PROCESSOR_DEFINE: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^#\s*define\s+([A-Za-z0-9_]+)\s+(.*)$").unwrap());
static RE_PREFIXED_PRE_PROCESSOR_DEFINE: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^#\s*define\s+([A-Za-z0-9_]+?)_([A-Za-z0-9_]+)\s+(.*)").unwrap());
static RE_PRE_PROCESSOR_IF: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^#if").unwrap());
static RE_PRE_PROCESSOR_ELIF: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^#elif").unwrap());
static RE_PRE_PROCESSOR_ELSE: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^#else").unwrap());
static RE_PRE_PROCESSOR_ENDIF: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^#endif").unwrap());

fn parse_number(source: &str) -> Option<u64> {
    let mut num_tok = Tokenizer::new(source);
    match num_tok.read() {
        Ok(t) => t.get_number(),
        Err(_) => None,
    }
}

fn set_project_prefix(data: &mut ProjectData, prefix: &str, logger: &mut StringLogger) {
    logger.log(format!("-> got project prefix '{}'", prefix));
    data.prefix_upper.push_str(prefix);
    data.prefix_upper.push('_');
    data.prefix_upper.make_ascii_uppercase();
    data.prefix_lower.push_str(prefix);
    data.prefix_lower.push('_');
    data.prefix_lower.make_ascii_lowercase();
    data.prefix = prefix.to_owned();
    data.got_prefix = true;
}

pub fn handle_define(line: &str, data: &mut ProjectData, pos: TokenPosition, logger: &mut StringLogger) -> Result<()> {
    if let Some((_, [prefix, name, value])) = RE_PREFIXED_PRE_PROCESSOR_DEFINE.captures(line).map(|caps| caps.extract()) {
        if ! data.got_prefix {
            set_project_prefix(data, prefix, logger);
        } else if prefix != data.prefix {
            logger.log(format!("-> ignoring define named without project prefix: {}_{}", prefix, name));
            return Ok(());
        }

        if name == "DATA_FILE_VERSION" {
            match parse_number(value) {
                Some(file_version) => {
                    if file_version > DataAssetStore::VERSION as u64 {
                        return error(format!("refusing to parse unknown file version {} (max supported: {})",
                                             file_version, DataAssetStore::VERSION), pos);
                    }
                    logger.log(format!("-> got file version {}", file_version));
                    return Ok(());
                }

                None => {
                    return error(format!("bad file version number: {}", value), pos);
                }
            }
        }

        if name == "DATA_VGA_SYNC_BITS" {
            match parse_number(value) {
                Some(vga_sync_bits) => {
                    if vga_sync_bits > 0xff {
                        return error(format!("bad vga_sync_bits value: {:#x}", vga_sync_bits), pos);
                    }
                    logger.log(format!("-> got vga_sync_bits {:#04x}", vga_sync_bits));
                    data.vga_sync_bits = vga_sync_bits as u8;
                    return Ok(())
                }
                None => {
                    return error(format!("bad vga_sync_bits value: {}", value), pos);
                }
            }
        }

        if name == "DATA_VGA_BITS_PER_PIXEL" {
            match parse_number(value) {
                Some(vga_bits_per_pixel) => {
                    if vga_bits_per_pixel != 6 && vga_bits_per_pixel != 8 {
                        return error(format!("bad vga_bits_per_pixel value: {} (only 8 and 6 are supported)",
                                             vga_bits_per_pixel), pos);
                    }
                    logger.log(format!("-> got vga_bits_per_pixel {:#04x}", vga_bits_per_pixel));
                    data.vga_bits_per_pixel = vga_bits_per_pixel as u8;
                    return Ok(());
                }
                None => {
                    return error(format!("bad vga_bits_per_pixel value: {}", value), pos);
                }
            }
        }

        if name == "DATA_TILES_PER_WORLD_BLOCK" {
            match parse_number(value) {
                Some(tiles_per_world_block) => {
                    if ! (8..=32).contains(&tiles_per_world_block) {
                        return error(format!("bad tiles_per_world_block value: {} (must be between 8 and 32)",
                                             tiles_per_world_block), pos);
                    }
                    logger.log(format!("-> got tiles_per_world_block {}", tiles_per_world_block));
                    data.tiles_per_world_block = tiles_per_world_block as u32;
                    return Ok(());
                }
                None => {
                    return error(format!("bad tiles_per_world_block value: {}", value), pos);
                }
            }
        }

        if name.starts_with("SPRITE_WIDTH_") ||
            name.starts_with("SPRITE_HEIGHT_") ||
            name.starts_with("SPRITE_STRIDE_") ||
            name.starts_with("SPRITE_FRAMES_") {
                // ignore
                return Ok(());
            }

        if name.starts_with("PAL_SPRITE_WIDTH_") ||
            name.starts_with("PAL_SPRITE_HEIGHT_") ||
            name.starts_with("PAL_SPRITE_FRAMES_") ||
            name.starts_with("PAL_SPRITE_DEPTH_") {
                // ignore
                return Ok(());
            }

        if name == "DATA_SAVE_TIMESTAMP" {
            // ignore
            return Ok(());
        }
    }

    logger.log(format!("-> ignoring define line {}", line));
    Ok(())
}

fn handle_if(line: &str, data: &ProjectData, logger: &mut StringLogger) -> Result<()> {
    if line == format!("#if {}DATA_BYTES", data.prefix_upper) { return Ok(()); }
    if line == format!("#endif /* {}DATA_BYTES */", data.prefix_upper) { return Ok(()); }

    if line == format!("#if {}ADD_ROOM_SCRIPTS", data.prefix_upper) { return Ok(()); }
    if line == format!("#endif /* {}ADD_ROOM_SCRIPTS */", data.prefix_upper) { return Ok(()); }

    logger.log(format!("-> ignoring pre-processor if line: {}", line));
    Ok(())
}

pub fn handle_line(line: &str, data: &mut ProjectData, pos: TokenPosition, logger: &mut StringLogger) -> Result<()> {
    // #define NAME VALUE
    if RE_PRE_PROCESSOR_DEFINE.is_match(line) {
        return handle_define(line, data, pos, logger);
    }

    // #if ...
    // #elif ...
    // #else ...
    // #endif ...
    if RE_PRE_PROCESSOR_IF.is_match(line) ||
        RE_PRE_PROCESSOR_ELIF.is_match(line) ||
        RE_PRE_PROCESSOR_ELSE.is_match(line) ||
        RE_PRE_PROCESSOR_ENDIF.is_match(line) {
            return handle_if(line, data, logger);
        }

    logger.log(format!("-> ignoring unknown pre-processor line: {}", line));
    Ok(())
}
