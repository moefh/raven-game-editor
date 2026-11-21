use std::io::{Result, Error};
use std::collections::HashMap;

use super::{
    ProjectDataWriter,
    IdentStore,
};
use super::super::{
    DataAssetId,
    DataAssetType,
    FontDataAssetId,
    PlayerChar,
};

fn write_player_char_suits(writer: &ProjectDataWriter, player_char: &PlayerChar, name_id: &str) -> Result<()> {
    writer.write(format!(
        "static const struct {}_PLAYER_CHAR_SUIT {}_player_char_suits_{}[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower,
        name_id
    ));
    for suit in player_char.suits.iter() {
        writer.write("  {\n");
        let anim_ident = writer.ident.get_asset_ident(DataAssetType::SpriteAnimation, suit.anim_id)?;
        writer.write(format!("    {},\n", anim_ident));
        writer.write(format!("    {},\n", suit.req_upgrade_id));
        writer.write("  },\n");
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

fn write_player_char_weapons(writer: &ProjectDataWriter, player_char: &PlayerChar, name_id: &str) -> Result<()> {
    writer.write(format!(
        "static const struct {}_PLAYER_CHAR_WEAPON {}_player_char_weapons_{}[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower,
        name_id
    ));
    for weapon in player_char.weapons.iter() {
        writer.write("  {\n");
        let shoot_anim_ident = writer.ident.get_asset_ident(DataAssetType::SpriteAnimation, weapon.shoot.anim_id)?;
        writer.write(format!("    {},  // shoot animation\n", shoot_anim_ident));
        let shoot_sfx_ident = writer.ident.get_asset_ident_or_none(DataAssetType::Sfx, weapon.shoot.sfx_id)?;
        writer.write(format!("    {},  // shoot sfx\n", shoot_sfx_ident));

        let bullet_anim_ident = writer.ident.get_asset_ident(DataAssetType::SpriteAnimation, weapon.bullet.anim_id)?;
        writer.write(format!("    {},  // bullet animation\n", bullet_anim_ident));
        let hit_anim_ident = writer.ident.get_asset_ident_or_none(DataAssetType::SpriteAnimation, weapon.bullet.hit_anim_id)?;
        writer.write(format!("    {},  // bullet hit animation\n", hit_anim_ident));
        let hit_sfx_ident = writer.ident.get_asset_ident_or_none(DataAssetType::Sfx, weapon.bullet.hit_sfx_id)?;
        writer.write(format!("    {},  // bullet hit sfx\n", hit_sfx_ident));

        writer.write(format!("    {},\n", weapon.req_collectable_id));
        writer.write(format!("    {},\n", weapon.bullet.ammo_pickup_id));
        writer.write(format!("    {},\n", weapon.shoot.crouch_x));
        writer.write(format!("    {},\n", weapon.shoot.crouch_y));
        writer.write(format!("    {},\n", weapon.bullet.x));
        writer.write(format!("    {},\n", weapon.bullet.y));
        writer.write(format!("    {},\n", weapon.bullet.dx));
        writer.write(format!("    {},\n", weapon.bullet.dy));
        writer.write(format!("    {},\n", weapon.bullet.damage));
        writer.write(format!("    {},\n", weapon.bullet.initial_delay));
        writer.write(format!("    {},\n", weapon.bullet.interval));
        writer.write(format!("    {},\n", weapon.shoot.anim_len));

        writer.write("  },\n");
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

pub fn write_player_chars(writer: &ProjectDataWriter, player_char_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === PLAYER CHARACTERS\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in player_char_ids.iter() {
        if let Some(player_char) = writer.store.assets.player_chars.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::PlayerChar, *id)?;
            write_player_char_suits(writer, player_char, name_id)?;
            write_player_char_weapons(writer, player_char, name_id)?;
        }
    }

    writer.log(format!("-> writing {} player_chars", writer.store.asset_ids.player_chars.len()));
    writer.write(format!(
        "const struct {}_PLAYER_CHAR {}_player_chars[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower
    ));
    for id in player_char_ids.iter() {
        if let Some(player_char) = writer.store.assets.player_chars.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::PlayerChar, *id)?;
            let (font_type_upper, font_type_lower, font_ident) = match player_char.font_id {
                FontDataAssetId::Fixed(font_id) => {
                    ("FIXED", "fixed", writer.ident.get_asset_ident(DataAssetType::Font, font_id)?)
                }
                FontDataAssetId::Prop(font_id) => {
                    ("PROP", "prop", writer.ident.get_asset_ident(DataAssetType::PropFont, font_id)?)
                }
            };
            let gamepad_sprite_ident = writer.ident.get_asset_ident(DataAssetType::Sprite, player_char.gamepad_sprite_id)?;
            writer.write("  {\n");
            writer.write(format!(
                "    {{ {}_FONT_TYPE_{}, .{}_index = {} }},\n",
                writer.ident.prefix_upper, font_type_upper,
                font_type_lower, font_ident
            ));
            writer.write(format!("    {},\n", gamepad_sprite_ident));
            writer.write(format!(
                "    {}, {}, {}_player_char_suits_{}, {}_player_char_weapons_{}\n",
                player_char.suits.len(),
                player_char.weapons.len(),
                writer.ident.prefix_lower, name_id,
                writer.ident.prefix_lower, name_id,
            ));
            writer.write("  },\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

fn write_player_char_suit_names(writer: &ProjectDataWriter, player_char: &PlayerChar, name_id_upper: &str) -> Result<()> {
    let mut index_to_name_id = HashMap::new();

    writer.write(format!("enum {}_PLAYER_CHAR_{}_SUIT_NAMES {{\n", writer.ident.prefix_upper, name_id_upper));
    for (index, suit) in player_char.suits.iter().enumerate() {
        let use_name_id = if suit.name == "names" { "names0" } else { &suit.name };
        IdentStore::add_unique_name_id(index, use_name_id, &mut index_to_name_id);
        let suit_name_id = index_to_name_id.get(&index).ok_or_else(|| {
            Error::other(format!("error reading name of suit {} player_char {}", index, player_char.asset.id))
        })?;
        writer.write(format!(
            "  {}_PLAYER_CHAR_{}_SUIT_{},\n",
            writer.ident.prefix_upper,
            name_id_upper,
            suit_name_id.to_ascii_uppercase()
        ));
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

fn write_player_char_weapon_names(writer: &ProjectDataWriter, player_char: &PlayerChar, name_id_upper: &str) -> Result<()> {
    let mut index_to_name_id = HashMap::new();

    writer.write(format!("enum {}_PLAYER_CHAR_{}_WEAPON_NAMES {{\n", writer.ident.prefix_upper, name_id_upper));
    for (index, weapon) in player_char.weapons.iter().enumerate() {
        let use_name_id = if weapon.name == "names" { "names0" } else { &weapon.name };
        IdentStore::add_unique_name_id(index, use_name_id, &mut index_to_name_id);
        let weapon_name_id = index_to_name_id.get(&index).ok_or_else(|| {
            Error::other(format!("error reading name of weapon {} player_char {}", index, player_char.asset.id))
        })?;
        writer.write(format!(
            "  {}_PLAYER_CHAR_{}_WEAPON_{},\n",
            writer.ident.prefix_upper,
            name_id_upper,
            weapon_name_id.to_ascii_uppercase()
        ));
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}

pub fn write_player_char_names(writer: &ProjectDataWriter, player_char_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === PLAYER CHARTER SUIT/WEAPON NAMES\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in player_char_ids.iter() {
        if let Some(player_char) = writer.store.assets.player_chars.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::PlayerChar, *id)?;
            let name_id_upper = name_id.to_ascii_uppercase();
            write_player_char_suit_names(writer, player_char, &name_id_upper)?;
            write_player_char_weapon_names(writer, player_char, &name_id_upper)?;
        }
    }

    Ok(())
}
