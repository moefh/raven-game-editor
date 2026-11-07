mod properties;

use crate::misc::IMAGES;
use crate::data_store::{
    self,
    DataAssetId,
    DataAssetList,
    AssetIdCollection,
    GenericAsset,
    Enemy,
    SpriteAnimation,
};

use super::{
    AssetEditorBase,
    WindowContext,
    EnemyTypeSel,
};
use super::super::{
    menu_item,
};

use properties::PropertiesDialog;

pub struct EnemyEditorAssetLists<'a> {
    pub animations: &'a DataAssetList<SpriteAnimation>,
    pub asset_ids: &'a AssetIdCollection,
}

pub struct EnemyEditor {
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl EnemyEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        EnemyEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(id),
        }
    }

    pub fn prepare_for_saving(&mut self, enemy: &mut Enemy) {
        self.editor.prepare_for_saving(enemy);
    }

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, _editor: &Editor, base: &AssetEditorBase, enemy: &Enemy) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, enemy.asset.id));
        egui::Panel::bottom(format!("editor_panel_{}_bottom", enemy.asset.id)).frame(bottom_frame).show(ui, |ui| {
            ui.horizontal(|ui| {
                let dirty = if base.is_dirty() { " (modified)" } else { "" };
                ui.add(egui::Label::new(format!(
                    "{} bytes{}",
                    enemy.data_size(),
                    dirty
                )).truncate());
            });
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, enemy: &mut Enemy, assets: &EnemyEditorAssetLists) {
        self.dialogs.show(wc, &mut self.editor, enemy);

        self.base.show_window(wc, enemy, [400.0, 250.0], [400.0, 250.0], |ui, wc, enemy, base| {
            Self::show_footer(ui, wc, &self.editor, base, enemy);
            self.editor.show(ui, wc, &mut self.dialogs, enemy, assets);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
}

impl Dialogs {
    fn new(_id: DataAssetId) -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
       }
    }

    fn show(&mut self, wc: &mut WindowContext, _editor: &mut Editor, enemy: &mut Enemy) {
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, enemy);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    id_editor_grid: String,
    sorted_animation_ids: Vec<DataAssetId>,
}

impl Editor {
    fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            id_editor_grid: format!("editor_panel_{}_grid", asset_id),
            sorted_animation_ids: Vec::new(),
        }
    }

    fn prepare_for_saving(&mut self, _enemy: &mut Enemy) {
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, enemy: &mut Enemy) {
        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Enemy", |ui| {
                    if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                        dialogs.properties_dialog.set_open(wc, enemy);
                    }
                });
            });
        });
    }
    fn show_number_editor<N: egui::emath::Numeric>(
        ui: &mut egui::Ui,
        val: &mut N,
        range: core::ops::RangeInclusive<N>,
    ) {
        ui.add(egui::DragValue::new(val).range(range).min_size(egui::Vec2::new(60.0, 0.0)));
    }

    fn show_asset_id_editor<A: GenericAsset>(
        ui: &mut egui::Ui,
        combo_id: &str,
        assets: &DataAssetList<A>,
        sorted_ids: &[DataAssetId],
        asset_id: &mut DataAssetId
    ) {
        let cur_name = match assets.get(asset_id) {
            Some(asset) => { &asset.asset().name }
            None => { "??" }
        };
        egui::ComboBox::from_id_salt(combo_id)
            .selected_text(cur_name)
            .show_ui(ui, |ui| {
                for id in sorted_ids {
                    if let Some(asset) = assets.get(id) {
                        ui.selectable_value(asset_id, *id, &asset.asset().name);
                    }
                }
            });
    }

    fn show_editor(
        &mut self,
        ui: &mut egui::Ui,
        _wc: &mut WindowContext,
        _dialogs: &mut Dialogs,
        enemy: &mut Enemy,
        assets: &EnemyEditorAssetLists,
    ) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.add_space(5.0);
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                egui::Grid::new(&self.id_editor_grid)
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Behavior:");
                        let mut enemy_type_sel = EnemyTypeSel::from(enemy.enemy_type);
                        egui::ComboBox::from_id_salt(format!("editor_{}_enemy_type", self.asset_id))
                            .selected_text(enemy_type_sel.text())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut enemy_type_sel, EnemyTypeSel::Walker,  EnemyTypeSel::Walker.text());
                                ui.selectable_value(&mut enemy_type_sel, EnemyTypeSel::Chiller, EnemyTypeSel::Chiller.text());
                                ui.selectable_value(&mut enemy_type_sel, EnemyTypeSel::Hopper,  EnemyTypeSel::Hopper.text());
                                ui.selectable_value(&mut enemy_type_sel, EnemyTypeSel::Floater, EnemyTypeSel::Floater.text());
                                ui.selectable_value(&mut enemy_type_sel, EnemyTypeSel::Other,   EnemyTypeSel::Other.text());
                            });
                        ui.end_row();
                        if enemy_type_sel != EnemyTypeSel::from(enemy.enemy_type) {
                            enemy.enemy_type = enemy_type_sel.into();
                        }

                        if enemy_type_sel == EnemyTypeSel::Other {
                            let mut enemy_type_val = enemy.enemy_type.value();
                            ui.label("   other:");
                            ui.add(egui::DragValue::new(&mut enemy_type_val).speed(1.0).range(0..=u16::MAX));
                            ui.end_row();
                            if enemy_type_val != enemy.enemy_type.value() {
                                enemy.enemy_type = enemy_type_val.into();
                            }
                        }

                        ui.label("Animation:");
                        Self::show_asset_id_editor(
                            ui,
                            "enemy_animation_id",
                            assets.animations,
                            &self.sorted_animation_ids,
                            &mut enemy.anim_id
                        );
                        ui.end_row();

                        ui.label("Health:");
                        ui.horizontal(|ui| {
                            Self::show_number_editor(ui, &mut enemy.health, 0..=u16::MAX);
                        });
                        ui.end_row();
                    });
            });
        });
    }

    fn clear_sorted_ids(&mut self) {
        self.sorted_animation_ids.clear();
    }

    fn sort_ids(&mut self, assets: &EnemyEditorAssetLists) {
        if self.sorted_animation_ids.len() != assets.asset_ids.animations.len() {
            self.clear_sorted_ids();
        }

        if self.sorted_animation_ids.is_empty() {
            assets.asset_ids.animations.copy_to(&mut self.sorted_animation_ids);
            data_store::utils::sort_asset_ids_by_name(&mut self.sorted_animation_ids, assets.animations);
        }
    }

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        enemy: &mut Enemy,
        assets: &EnemyEditorAssetLists,
    ) {
        self.sort_ids(assets);

        self.show_menu_bar(ui, wc, dialogs, enemy);

        self.show_editor(ui, wc, dialogs, enemy, assets);
    }
}
