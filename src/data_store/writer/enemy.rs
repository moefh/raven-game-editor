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
            let anim_ident = writer.ident.get_asset_ident(DataAssetType::SpriteAnimation, enemy.anim_id)?;
            writer.write(format!(
                "  {{ {}, {}, {} }},\n",
                enemy.enemy_type.value(),
                anim_ident,
                enemy.health,
            ));
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}
