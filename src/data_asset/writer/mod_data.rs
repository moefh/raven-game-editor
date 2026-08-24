use std::io::{Result, Error};
use std::collections::HashMap;

use super::ProjectDataWriter;
use super::super::{
    DataAssetId,
    DataAssetType,
    ModData,
};

struct ModSampleRef {
    mod_id: DataAssetId,
    sample_index: usize,
}

// build the references for each
fn get_mod_sample_refs(writer: &ProjectDataWriter) -> (usize, HashMap<DataAssetId,Vec<ModSampleRef>>) {
    let mod_ids: Vec::<DataAssetId> = writer.store.asset_ids.mods.iter().copied().collect();

    let mut all_samples = HashMap::new();
    for mod_id in mod_ids.iter() {
        if let Some(mod_data) = writer.store.assets.mods.get(mod_id) {
            let mut mod_samples = Vec::new();
            for sample_index in 0..mod_data.samples.len() {
                mod_samples.push(ModSampleRef { mod_id: *mod_id, sample_index });
            }
            all_samples.insert(*mod_id, mod_samples);
        }
    }

    let mut merge_sample_saved_size = 0;
    for (mod1_index, mod1_id) in mod_ids.iter().enumerate() {
        if let Some(mod1_data) = writer.store.assets.mods.get(mod1_id) {
            for (mod2_index, mod2_id) in mod_ids.iter().enumerate().skip(mod1_index+1) {
                if let Some(mod2_data) = writer.store.assets.mods.get(mod2_id) {
                    for (sample1_index, sample1) in mod1_data.samples.iter().enumerate() {
                        if sample1.len == 0 || sample1.data.is_none() { continue; }
                        for (sample2_index, sample2) in mod2_data.samples.iter().enumerate() {
                            if sample2.len == 0 || sample2.data.is_none() { continue; }
                            if ModData::are_mod_samples_equal(sample1, sample2) {
                                writer.log(format!("-> merging mod samples: (mod{}:sample{}) to (mod{}:sample{})",
                                    mod2_index+1, sample2_index+1, mod1_index+1, sample1_index+1));
                                if let Some(mod2_samples) = all_samples.get_mut(mod2_id) {
                                    mod2_samples[sample2_index] = ModSampleRef {
                                        mod_id: *mod1_id,
                                        sample_index: sample1_index,
                                    };
                                    merge_sample_saved_size += (sample2.len * (sample2.bits_per_sample/8) as u32) as usize;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (merge_sample_saved_size, all_samples)
}

fn write_mod_samples_data(writer: &ProjectDataWriter, mod_data: &ModData, name_id: &str, sample_refs: &[ModSampleRef]) {
    for (index, sample) in mod_data.samples.iter().enumerate() {
        if let Some(sample_ref) = sample_refs.get(index) {
            if sample_ref.mod_id != mod_data.asset.id || sample_ref.sample_index != index {
                continue;  // skip this sample; it's been merged with another
            }
            if let Some(data) = &sample.data {
                writer.write(format!("static const int{}_t {}_mod_samples_{}_sample{:02}[] = {{",
                    sample.bits_per_sample, writer.ident.prefix_lower, name_id, index+1));
                for (i, spl) in data.iter().enumerate() {
                    if i.is_multiple_of(16) { writer.write("\n  "); }
                    if sample.bits_per_sample == 16 {
                        writer.write(format!("{},", spl));
                    } else {
                        writer.write(format!("{},", spl>>8));
                    }
                }
                writer.write("\n};\n");
                writer.write("\n");
            }
        }
    }
}

fn write_mod_pattern(writer: &ProjectDataWriter, mod_data: &ModData, name_id: &str) {
    writer.write(format!(
        "static const struct {}_MOD_CELL {}_mod_pattern_{}[] = {{\n",
        writer.ident.prefix_upper,
        writer.ident.prefix_lower,
        name_id
    ));
    let num_channels = mod_data.num_channels as usize;
    let num_patterns = mod_data.pattern.len().div_ceil(64 * num_channels);
    for pattern_num in 0..num_patterns {
        writer.write(format!("  // pattern {}\n", pattern_num));
        for row in 0..64 {
            writer.write("  ");
            for ch in 0..num_channels {
                let cell_index = (64 * pattern_num + row) * num_channels + ch;
                if cell_index >= mod_data.pattern.len() { break; }
                let cell = mod_data.pattern[cell_index];
                let note_index = if cell.period == 0 {
                    0xff
                } else {
                    let (note, octave) = ModData::get_period_note(cell.period);
                    if note < 0 || octave < 0 { 0xff } else { note + octave * 12 }
                };
                writer.write(format!("{{ {:>2}, {:#04x}, {:#05x}, }}, ", cell.sample, note_index, cell.effect));
            }
            writer.write("\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");
}

fn write_mod_samples(writer: &ProjectDataWriter, mod_data: &ModData, sample_refs: &[ModSampleRef]) -> Result<()> {
    for (index, sample) in mod_data.samples.iter().enumerate() {
        writer.write(format!("      {{{:>6},{:>6},{:>6}, {:#04x}, {:>2}, {:>2},",
            sample.len, sample.loop_start, sample.loop_len,
            if sample.finetune < 0 { sample.finetune + 16 } else { sample.finetune },
            sample.volume, sample.bits_per_sample));
        if sample.len == 0 || sample.data.is_none() {
            writer.write(" { .data = NULL }, },\n");
            continue;
        }
        if let Some(sample_ref) = sample_refs.get(index) {
            let sample_ref_mod_name_id = writer.ident.get_asset_name_id(DataAssetType::ModData, sample_ref.mod_id)?;
            writer.write(format!(" {{ .data{} = {}_mod_samples_{}_sample{:02} }}, }},\n",
                sample.bits_per_sample, writer.ident.prefix_lower,
                sample_ref_mod_name_id, sample_ref.sample_index+1));
        }
    }
    Ok(())
}

pub fn write_mods(writer: &ProjectDataWriter, mod_ids: &[DataAssetId]) -> Result<usize> {
    writer.write("// ================================================================\n");
    writer.write("// === MOD\n");
    writer.write("// ================================================================\n");
    writer.write("\n");

    // samples
    let (merge_sample_saved_size, mod_sample_refs) = get_mod_sample_refs(writer);
    for id in mod_ids.iter() {
        if let Some(mod_data) = writer.store.assets.mods.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::ModData, *id)?;
            let sample_refs = mod_sample_refs.get(id).ok_or_else(|| {
                Error::other(format!("can't find sample refs for mod {}", id))
            })?;
            write_mod_samples_data(writer, mod_data, name_id, sample_refs);
        }
    }

    // patterns
    for id in mod_ids.iter() {
        if let Some(mod_data) = writer.store.assets.mods.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::ModData, *id)?;
            write_mod_pattern(writer, mod_data, name_id);
        }
    }

    // mods
    writer.log(format!("-> writing {} mods", writer.store.asset_ids.mods.len()));
    writer.write(format!("const struct {}_MOD_DATA {}_mods[] = {{\n", writer.ident.prefix_upper, writer.ident.prefix_lower));
    for id in mod_ids.iter() {
        if let Some(mod_data) = writer.store.assets.mods.get(id) {
            let name_id = writer.ident.get_asset_name_id(DataAssetType::ModData, *id)?;
            writer.write("  {\n");
            writer.write("    // samples:\n");
            writer.write("    {\n");
            if let Some(sample_refs) = mod_sample_refs.get(id) {
                write_mod_samples(writer, mod_data, sample_refs)?;
            }
            writer.write("    },\n");
            writer.write("    // num channels:\n");
            writer.write(format!("    {},\n", mod_data.num_channels));
            writer.write("    // song positions:\n");
            writer.write(format!("    {}, {{", mod_data.song_positions.len()));
            for (index, song_pos) in mod_data.song_positions.iter().enumerate() {
                if index.is_multiple_of(16) { writer.write("\n      "); }
                writer.write(format!("{:>3},", song_pos));
            }
            writer.write("\n");
            writer.write("    },\n");
            writer.write("    // pattern:\n");
            writer.write(format!("    {}, {}_mod_pattern_{},\n",
                mod_data.pattern.len().div_ceil(64*mod_data.num_channels as usize),
                writer.ident.prefix_lower, name_id));
            writer.write("  },\n");
        }
    }
    writer.write("};\n");
    writer.write("\n");

    Ok(merge_sample_saved_size)
}
