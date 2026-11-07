use std::io::Result;

use super::{
    ProjectDataWriter,
};
use super::super::{
    DataAssetId,
    DataAssetType,
};

pub fn write_enemies(writer: &ProjectDataWriter, enemy_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === ENEMIES\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    writer.log(format!("-> writing {} enemies", writer.store.asset_ids.enemies.len()));
    writer.write(format!(
        "const struct {}_ENEMY {}_enemies[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower
    ));
    for id in enemy_ids.iter() {
        if let Some(enemy) = writer.store.assets.enemies.get(id) {
            let anim_index = writer.ident.get_asset_index(DataAssetType::SpriteAnimation, enemy.anim_id)?;
            writer.write(format!(
                "  {{ {}, &{}_sprite_animations[{}], {} }},\n",
                enemy.enemy_type.value(),
                writer.ident.prefix_lower,
                anim_index,
                enemy.health,
            ));
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}
