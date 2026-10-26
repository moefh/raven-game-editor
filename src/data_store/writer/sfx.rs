use std::io::Result;

use super::ProjectDataWriter;
use super::super::{
    DataAssetId,
    DataAssetType,
    Sfx,
};

fn write_sfx_data(writer: &ProjectDataWriter, sfx: &Sfx, name_id: &str) {
    writer.write(format!(
        "static const int{}_t {}_sfx_samples_{}[] = {{",
        sfx.bits_per_sample,
        writer.ident.prefix_lower,
        name_id
    ));
    for (i, spl) in sfx.samples.iter().enumerate() {
        if i.is_multiple_of(16) { writer.write("\n  "); }
        if sfx.bits_per_sample == 16 {
            writer.write(format!("{},", spl));
        } else {
            writer.write(format!("{},", spl>>8));
        }
    }
    writer.write("\n};\n");
    writer.write("\n");
}

pub fn write_sfxs(writer: &ProjectDataWriter, sfx_ids: &[DataAssetId]) -> Result<()> {
    writer.write("// ================================================================\n");
    writer.write("// === SFX\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    for id in sfx_ids.iter() {
        if let Some(sfx) = writer.store.assets.sfxs.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Sfx, *id)?;
            write_sfx_data(writer, sfx, name_id);
        }
    }

    writer.log(format!("-> writing {} sfxs", writer.store.asset_ids.sfxs.len()));
    writer.write(format!("const struct {}_SFX {}_sfxs[] = {{\n", writer.ident.prefix_upper, writer.ident.prefix_lower));
    for id in sfx_ids.iter() {
        if let Some(sfx) = writer.store.assets.sfxs.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::Sfx, *id)?;
            writer.write(format!(
                "  {{{:>6},{:>5},{:>5}, {}, {{ .spl{} = {}_sfx_samples_{} }} }},\n",
                sfx.len,
                sfx.loop_start,
                sfx.loop_len,
                sfx.bits_per_sample,
                sfx.bits_per_sample,
                writer.ident.prefix_lower,
                name_id
            ));
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(())
}
