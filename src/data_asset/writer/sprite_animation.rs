use std::io::{Result, Error};
use std::collections::HashMap;
use std::sync::LazyLock;
use regex::Regex;

use super::{
    ProjectDataWriter,
    IdentStore,
};
use super::super::{
    DataAssetId,
    DataAssetType,
    SpriteAnimation,
};

static RE_UNNAMED_LOOP: LazyLock<Regex> = LazyLock::new(
    || Regex::new(r"^loop_[0-9]+$").unwrap());

pub struct AnimationInfo {
    pub add_foot: bool,
    pub loop_offsets: Vec<usize>,
    pub loop_index_to_name_id: HashMap<usize,String>,
}

fn write_animation_frames(writer: &ProjectDataWriter, anim: &SpriteAnimation, name_id: &str) -> AnimationInfo {
    let add_foot = anim.loops.iter().any(|l| l.frame_indices.iter().any(|f| f.foot_index.is_some()));
    let mut loop_offsets = Vec::new();
    let mut loop_index_to_name_id = HashMap::new();

    writer.write(format!(
        "static const uint8_t {}_sprite_animation_frames_{}[] = {{\n",
        writer.ident.prefix_lower,
        name_id
    ));
    let mut offset = 0;
    for (loop_index, aloop) in anim.loops.iter().enumerate() {
        let use_loop_name_id = if aloop.name_id == "names" { "names0" } else { &aloop.name_id };
        IdentStore::add_unique_name_id(loop_index, use_loop_name_id, &mut loop_index_to_name_id);
        if aloop.frame_indices.is_empty() { continue; }
        writer.write(format!("  // {}\n  ", aloop.name_id));
        for frame in aloop.frame_indices.iter() {
            writer.write(format!("{:#04x},", frame.head_index.unwrap_or(0xff)));
            if add_foot {
                writer.write(format!("{:#04x},", frame.foot_index.unwrap_or(0xff)));
            }
        }
        writer.write("\n");
        loop_offsets.push(offset);
        offset += aloop.frame_indices.len() * if add_foot { 2 } else { 1 };
    }
    writer.write("};\n");
    writer.write("\n");

    AnimationInfo {
        add_foot,
        loop_offsets,
        loop_index_to_name_id,
    }
}

pub fn write_sprite_animations(writer: &ProjectDataWriter, anim_ids: &[DataAssetId]) -> Result<HashMap<DataAssetId, AnimationInfo>> {
    writer.write("// ================================================================\n");
    writer.write("// === SPRITE ANIMATIONS\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    let mut animation_info = HashMap::new();

    for id in anim_ids.iter() {
        if let Some(anim) = writer.store.assets.animations.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::SpriteAnimation, *id)?;
            animation_info.insert(*id, write_animation_frames(writer, anim, name_id));
        }
    }

    writer.log(format!("-> writing {} animations", writer.store.asset_ids.animations.len()));
    writer.write(format!(
        "const struct {}_SPRITE_ANIMATION {}_sprite_animations[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower
    ));
    for id in anim_ids.iter() {
        if let Some(anim) = writer.store.assets.animations.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::SpriteAnimation, *id)?;
            let sprite_index = writer.ident.get_asset_index(DataAssetType::Sprite, anim.sprite_id)?;
            let info = animation_info.get(id).ok_or_else(|| {
                Error::other(format!("can't find info for animation {}", id))
            })?;
            writer.write("  {\n");
            writer.write(format!("    {}_sprite_animation_frames_{},\n", writer.ident.prefix_lower, name_id));
            writer.write(format!("    &{}_sprites[{}],\n", writer.ident.prefix_lower, sprite_index));
            writer.write(format!("    {{ {}, {}, {}, {} }},\n", anim.clip_rect.x, anim.clip_rect.y, anim.clip_rect.w, anim.clip_rect.h));
            writer.write(format!("    {},\n", if info.add_foot { 1 } else { 0 }));
            writer.write(format!("    {},\n", anim.foot_overlap));
            writer.write("    {\n");
            let mut last_offset = 0;
            for (loop_index, aloop) in anim.loops.iter().enumerate() {
                let loop_name_id = info.loop_index_to_name_id.get(&loop_index).ok_or_else(|| {
                    Error::other(format!("can't find loop name {} for animation {}", loop_index, id))
                })?;
                let loop_offset = info.loop_offsets.get(loop_index).copied().unwrap_or(last_offset);
                writer.write(format!(
                    "      {{ {:>5}, {:>5}, {}, {:>3} }},",
                    loop_offset,
                    aloop.frame_indices.len(),
                    if aloop.dont_loop { 1 } else { 0 },
                    aloop.frame_speed.saturating_sub(1) & 0xff,
                ));
                if ! aloop.frame_indices.is_empty() {
                    writer.write(format!(" // {}\n", loop_name_id));
                } else {
                    writer.write("\n");
                }
                last_offset = loop_offset + aloop.frame_indices.len();
            }
            writer.write("    }\n");
            writer.write("  },\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(animation_info)
}

pub fn write_animation_names(
    writer: &ProjectDataWriter,
    anim_ids: &[DataAssetId],
    animation_info: HashMap<DataAssetId, AnimationInfo>
) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === SPRITE ANIMATION LOOP NAMES\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in anim_ids.iter() {
        if let Some(anim) = writer.store.assets.animations.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::SpriteAnimation, *id)?;
            let info = animation_info.get(id).ok_or_else(|| {
                Error::other(format!("error reading loop names for animation {}", id))
            })?;
            let name_id_upper = name_id.to_ascii_uppercase();
            let mut num_named_loops = 0;
            for loop_index in (0..anim.loops.len()).rev() {
                let loop_name = info.loop_index_to_name_id.get(&loop_index).ok_or_else(|| {
                    Error::other(format!("error reading name for loop {} of animation {}", loop_index, id))
                })?;
                if ! RE_UNNAMED_LOOP.is_match(loop_name) {
                    num_named_loops = loop_index + 1;
                    break;
                }
            }
            if num_named_loops == 0 { continue; }

            writer.write(format!("enum {}_SPRITE_ANIMATION_{}_LOOP_NAMES {{\n", writer.ident.prefix_upper, name_id_upper));
            for loop_index in 0..num_named_loops {
                let loop_name_id = info.loop_index_to_name_id.get(&loop_index).ok_or_else(|| {
                    Error::other(format!("error reading name for loop {} of animation {}", loop_index, id))
                })?;
                let loop_name_id_upper = loop_name_id.to_ascii_uppercase();
                writer.write(format!("  {}_SPRITE_ANIMATION_{}_LOOP_{},\n", writer.ident.prefix_upper, name_id_upper, loop_name_id_upper));
            }
            writer.write("};\n");
            writer.write("\n");
        }
    }

    Ok(())
}
