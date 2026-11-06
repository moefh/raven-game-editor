mod properties;

use crate::misc::IMAGES;
use crate::data_store::{
    self,
    DataAssetId,
    DataAssetList,
    AssetIdCollection,
    GenericAsset,
    DataStoreTables,
    DataStoreItemTable,
    PlayerChar,
    PlayerCharSuit,
    PlayerCharWeapon,
    SpriteAnimation,
    Sfx,
};

use super::{
    AssetEditorBase,
    WindowContext,
    DialogResult,
};
use super::dialogs::ConfirmationDialog;
use super::super::{
    menu_item,
    menu_item_no_image,
};

use properties::PropertiesDialog;

pub struct PlayerCharEditorAssetLists<'a> {
    pub animations: &'a DataAssetList<SpriteAnimation>,
    pub sfxs: &'a DataAssetList<Sfx>,
    pub asset_ids: &'a AssetIdCollection,
    pub tables: &'a DataStoreTables,
}

pub enum ItemAction {
    None,
    AddSuit,
    SelectSuit(usize),
    RemoveSuit(usize),
    AddWeapon,
    SelectWeapon(usize),
    RemoveWeapon(usize),
}

#[derive(Clone, Copy)]
pub enum ItemRef {
    None,
    Suit(usize),
    Weapon(usize),
}

impl ItemRef {
    pub fn is_suit(&self, suit_index: usize) -> bool {
        if let ItemRef::Suit(index) = self && *index == suit_index {
            true
        } else {
            false
        }
    }

    pub fn is_weapon(&self, weapon_index: usize) -> bool {
        if let ItemRef::Weapon(index) = self && *index == weapon_index {
            true
        } else {
            false
        }
    }
}

pub struct PlayerCharEditor {
    pub base: AssetEditorBase,
    editor: Editor,
    dialogs: Dialogs,
}

impl PlayerCharEditor {
    pub fn new(id: DataAssetId, open: bool) -> Self {
        PlayerCharEditor {
            base: AssetEditorBase::new(id, open),
            editor: Editor::new(id),
            dialogs: Dialogs::new(id),
        }
    }

    pub fn prepare_for_saving(&mut self, player_char: &mut PlayerChar) {
        self.editor.prepare_for_saving(player_char);
    }

    fn show_footer(ui: &mut egui::Ui, wc: &WindowContext, _editor: &Editor, base: &AssetEditorBase, player_char: &PlayerChar) {
        let margin = egui::Margin { left: 5, right: 5, top: 4, bottom: 0 };
        let bottom_frame = egui::Frame::NONE.inner_margin(margin).fill(base.footer_bg_color(wc, player_char.asset.id));
        egui::Panel::bottom(format!("editor_panel_{}_bottom", player_char.asset.id)).frame(bottom_frame).show(ui, |ui| {
            ui.horizontal(|ui| {
                let dirty = if base.is_dirty() { " (modified)" } else { "" };
                ui.add(egui::Label::new(format!(
                    "{} bytes{}",
                    player_char.data_size(),
                    dirty
                )).truncate());
            });
        });
    }

    pub fn show(&mut self, wc: &mut WindowContext, player_char: &mut PlayerChar, assets: &PlayerCharEditorAssetLists) {
        self.dialogs.show(wc, &mut self.editor, player_char);

        self.base.show_window(wc, player_char, [500.0, 350.0], [600.0, 450.0], |ui, wc, player_char, base| {
            Self::show_footer(ui, wc, &self.editor, base, player_char);
            self.editor.show(ui, wc, &mut self.dialogs, player_char, assets);
        });
    }
}

struct Dialogs {
    properties_dialog: PropertiesDialog,
    confirmation_dialog: ConfirmationDialog,
}

impl Dialogs {
    fn new(id: DataAssetId) -> Self {
        Dialogs {
            properties_dialog: PropertiesDialog::new(),
            confirmation_dialog: ConfirmationDialog::new(id),
       }
    }

    fn show(&mut self, wc: &mut WindowContext, _editor: &mut Editor, player_char: &mut PlayerChar) {
        if self.properties_dialog.open {
            self.properties_dialog.show(wc, player_char);
        }
    }
}

struct Editor {
    asset_id: DataAssetId,
    id_trees_panel: egui::Id,
    id_suits_tree: egui::Id,
    id_weapons_tree: egui::Id,
    id_grid_suit: String,
    id_grid_weapon: String,
    selected_item: ItemRef,
    remove_item: ItemRef,
    sorted_animation_ids: Vec<DataAssetId>,
    sorted_sfx_ids: Vec<DataAssetId>,
}

impl Editor {
    fn new(asset_id: DataAssetId) -> Self {
        Editor {
            asset_id,
            id_trees_panel: egui::Id::unique(format!("editor_{}_trees_panel", asset_id)),
            id_suits_tree: egui::Id::unique(format!("editor_{}_suits_tree", asset_id)),
            id_weapons_tree: egui::Id::unique(format!("editor_{}_weapons_tree", asset_id)),
            id_grid_suit: format!("editor_panel_{}_suit_grid", asset_id),
            id_grid_weapon: format!("editor_panel_{}_weapon_grid", asset_id),
            selected_item: ItemRef::None,
            remove_item: ItemRef::None,
            sorted_animation_ids: Vec::new(),
            sorted_sfx_ids: Vec::new(),
        }
    }

    fn prepare_for_saving(&mut self, _player_char: &mut PlayerChar) {
    }

    fn select_some_item(&mut self, player_char: &PlayerChar, suit_first: bool) -> bool {
        if suit_first && ! player_char.suits.is_empty() {
            self.selected_item = ItemRef::Suit(0);
            return true;
        }
        if ! player_char.weapons.is_empty() {
            self.selected_item = ItemRef::Weapon(0);
            return true;
        }
        if ! player_char.suits.is_empty() {
            self.selected_item = ItemRef::Suit(0);
            return true;
        }
        self.selected_item = ItemRef::None;
        false
    }

    fn ensure_valid_selection(&mut self, player_char: &PlayerChar) {
        match self.selected_item {
            ItemRef::None => {
                self.select_some_item(player_char, true);
            }
            ItemRef::Suit(index) => {
                if player_char.suits.get(index).is_none() {
                    self.select_some_item(player_char, true);
                }
            }
            ItemRef::Weapon(index) => {
                if player_char.weapons.get(index).is_none() {
                    self.select_some_item(player_char, false);
                }
            }
        }
    }

    fn remove_item(&mut self, player_char: &mut PlayerChar, item: ItemRef) {
        match item {
            ItemRef::Suit(index) => {
                if index < player_char.suits.len() {
                    player_char.suits.remove(index);
                    self.ensure_valid_selection(player_char);
                }
            }
            ItemRef::Weapon(index) => {
                if index < player_char.weapons.len() {
                    player_char.weapons.remove(index);
                    self.ensure_valid_selection(player_char);
                }
            }
            ItemRef::None => {}
        }
    }

    fn request_remove_item(&mut self, wc: &mut WindowContext, dialogs: &mut Dialogs, player_char: &PlayerChar, remove_item: ItemRef) {
        self.remove_item = remove_item;
        match remove_item {
            ItemRef::Suit(index) => {
                if let Some(suit) = player_char.suits.get(index) {
                    dialogs.confirmation_dialog.set_open(wc, "Remove Suit", format!("Remove suit '{}'?", suit.name), "Yes", "No");
                }
            }
            ItemRef::Weapon(index) => {
                if let Some(weapon) = player_char.weapons.get(index) {
                    dialogs.confirmation_dialog.set_open(wc, "Remove Weapon", format!("Remove weapon '{}'?", weapon.name), "Yes", "No");
                }
            }
            ItemRef::None => {}
        }
    }

    fn run_action(
        &mut self,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        player_char: &mut PlayerChar,
        action: ItemAction,
        assets: &PlayerCharEditorAssetLists,
    ) {
        match action {
            ItemAction::SelectSuit(index) => {
                self.selected_item = ItemRef::Suit(index);
            }
            ItemAction::SelectWeapon(index) => {
                self.selected_item = ItemRef::Weapon(index);
            }
            ItemAction::AddSuit => {
                if let Some(anim_id) = assets.asset_ids.animations.get_first() {
                    let index = player_char.suits.len();
                    player_char.suits.push(PlayerCharSuit::new("new_suit", anim_id));
                    self.selected_item = ItemRef::Suit(index);
                } else {
                    wc.open_message_box("No Sprite Animation Available", "You must create a sprite animation first!");
                }
            }
            ItemAction::AddWeapon => {
                if let Some(anim_id) = assets.asset_ids.animations.get_first() {
                    let index = player_char.weapons.len();
                    player_char.weapons.push(PlayerCharWeapon::new("new_weapon", anim_id, anim_id));
                    self.selected_item = ItemRef::Weapon(index);
                } else {
                    wc.open_message_box("No Sprite Animation Available", "You must create a sprite animation first!");
                }
            }
            ItemAction::RemoveSuit(index) => {
                self.request_remove_item(wc, dialogs, player_char, ItemRef::Suit(index));
            }
            ItemAction::RemoveWeapon(index) => {
                self.request_remove_item(wc, dialogs, player_char, ItemRef::Weapon(index));
            }
            ItemAction::None => {}
        }
    }

    fn show_menu_bar(&mut self, ui: &mut egui::Ui, wc: &mut WindowContext, dialogs: &mut Dialogs, player_char: &mut PlayerChar) {
        egui::Panel::top(format!("editor_panel_{}_top", self.asset_id)).show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("Character", |ui| {
                    if ui.add(menu_item(IMAGES.properties, " Properties...")).clicked() {
                        dialogs.properties_dialog.set_open(wc, player_char);
                    }
                });
            });
        });
    }

    fn show_suits_tree(
        &mut self,
        ui: &mut egui::Ui,
        _wc: &mut WindowContext,
        _dialogs: &mut Dialogs,
        player_char: &mut PlayerChar
    ) -> ItemAction {
        let mut action = ItemAction::None;
        let node = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), self.id_suits_tree, true);
        let mut toggle_node_open = false;
        let mut node_resp = node.show_header(ui, |ui| {
            let resp = ui.add(egui::Label::new("Suits").selectable(false).sense(egui::Sense::click()));
            egui::Popup::context_menu(&resp).show(|ui| {
                if ui.add(menu_item_no_image(" Add suit")).clicked() {
                    action = ItemAction::AddSuit;
                }
            });
            toggle_node_open = resp.clicked();
        });
        if toggle_node_open {
            node_resp.toggle();
        }
        node_resp.body(|ui| {
            for (suit_index, suit) in player_char.suits.iter().enumerate() {
                let mut selected = self.selected_item.is_suit(suit_index);
                let resp = ui.toggle_value(&mut selected, &suit.name);
                if resp.clicked() || resp.secondary_clicked() {
                    action = ItemAction::SelectSuit(suit_index);
                }
                egui::Popup::context_menu(&resp).show(|ui| {
                    if ui.add(menu_item_no_image(" Remove suit")).clicked() {
                        action = ItemAction::RemoveSuit(suit_index);
                    }
                });
            }
        });
        action
    }

    fn show_weapons_tree(
        &mut self,
        ui: &mut egui::Ui,
        _wc: &mut WindowContext,
        _dialogs: &mut Dialogs,
        player_char: &mut PlayerChar
    ) -> ItemAction {
        let mut action = ItemAction::None;
        let node = egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), self.id_weapons_tree, true);
        let mut toggle_node_open = false;
        let mut node_resp = node.show_header(ui, |ui| {
            let resp = ui.add(egui::Label::new("Weapons").selectable(false).sense(egui::Sense::click()));
            egui::Popup::context_menu(&resp).show(|ui| {
                if ui.add(menu_item_no_image(" Add weapon")).clicked() {
                    action = ItemAction::AddWeapon;
                }
            });
            toggle_node_open = resp.clicked();
        });
        if toggle_node_open {
            node_resp.toggle();
        }
        node_resp.body(|ui| {
            for (weapon_index, weapon) in player_char.weapons.iter().enumerate() {
                let mut selected = self.selected_item.is_weapon(weapon_index);
                let resp = ui.toggle_value(&mut selected, &weapon.name);
                if resp.clicked() || resp.secondary_clicked() {
                    action = ItemAction::SelectWeapon(weapon_index);
                }
                egui::Popup::context_menu(&resp).show(|ui| {
                    if ui.add(menu_item_no_image(" Remove weapon")).clicked() {
                        action = ItemAction::RemoveWeapon(weapon_index);
                    }
                });
            }
        });
        action
    }

    fn show_number_editor<N: egui::emath::Numeric>(
        ui: &mut egui::Ui,
        val: &mut N,
        range: core::ops::RangeInclusive<N>,
    ) {
        ui.add(egui::DragValue::new(val).range(range).min_size(egui::Vec2::new(60.0, 0.0)));
    }

    fn show_table_item_id_editor(
        ui: &mut egui::Ui,
        combo_id: &str,
        item_id: &mut u16,
        item_table: &DataStoreItemTable,
    ) {
        let mut show_other = false;
        let item_name = item_table.items.get(*item_id as usize).map(|item| item.name.as_str()).unwrap_or_else(|| {
            if *item_id == u16::MAX {
                "(none)"
            } else {
                show_other = true;
                "other..."
            }
        });
        egui::ComboBox::from_id_salt(combo_id)
            .selected_text(item_name)
            .show_ui(ui, |ui| {
                ui.selectable_value(item_id, u16::MAX, "(none)");
                for (index, item) in item_table.items.iter().enumerate() {
                    ui.selectable_value(item_id, index as u16, &item.name);
                }
                ui.selectable_value(item_id, item_table.items.len() as u16, "other...");
            });

        if show_other || (*item_id != u16::MAX && *item_id as usize >= item_table.items.len()) {
            ui.end_row();
            ui.label("   other:");
            ui.add(egui::DragValue::new(item_id).speed(1.0).range(0..=u16::MAX));
        }
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

    fn show_optional_asset_id_editor<A: GenericAsset>(
        ui: &mut egui::Ui,
        combo_id: &str,
        assets: &DataAssetList<A>,
        sorted_ids: &[DataAssetId],
        asset_id: &mut Option<DataAssetId>
    ) {
        let cur_name = match asset_id.and_then(|asset_id| assets.get(&asset_id)) {
            Some(asset) => { &asset.asset().name }
            None => { "<none>" }
        };
        egui::ComboBox::from_id_salt(combo_id)
            .selected_text(cur_name)
            .show_ui(ui, |ui| {
                ui.selectable_value(asset_id, None, "<none>");
                for id in sorted_ids {
                    if let Some(asset) = assets.get(id) {
                        ui.selectable_value(asset_id, Some(*id), &asset.asset().name);
                    }
                }
            });
    }

    fn show_suit_editor_tab(
        &mut self,
        ui: &mut egui::Ui,
        _wc: &mut WindowContext,
        _dialogs: &mut Dialogs,
        player_char: &mut PlayerChar,
        suit_index: usize,
        assets: &PlayerCharEditorAssetLists,
    ) {
        egui::Panel::top(format!("editor_{}_suit_top", self.asset_id)).show(ui, |ui| {
            ui.add_space(4.0);
            ui.heading("Suit");
            ui.add_space(1.0);
        });
        if let Some(suit) = player_char.suits.get_mut(suit_index) {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add_space(5.0);
                egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    egui::Grid::new(&self.id_grid_suit)
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut suit.name);
                            ui.end_row();

                            ui.label("Animation:");
                            Self::show_asset_id_editor(
                                ui,
                                "suit_animation_id",
                                assets.animations,
                                &self.sorted_animation_ids,
                                &mut suit.anim_id
                            );
                            ui.end_row();

                            ui.label("Req Upgrade:");
                            Self::show_table_item_id_editor(
                                ui,
                                "suit_req_upgrade_id",
                                &mut suit.req_upgrade_id,
                                &assets.tables.upgrade
                            );
                            ui.end_row();
                        });
                });
            });
        } else {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.label("Invalid suit selection!");
            });
        }
    }

    fn show_weapon_editor_tab(
        &mut self,
        ui: &mut egui::Ui,
        _wc: &mut WindowContext,
        _dialogs: &mut Dialogs,
        player_char: &mut PlayerChar,
        weapon_index: usize,
        assets: &PlayerCharEditorAssetLists,
    ) {
        egui::Panel::top(format!("editor_{}_weapon_top", self.asset_id)).show(ui, |ui| {
            ui.add_space(4.0);
            ui.heading("Weapon");
            ui.add_space(1.0);
        });
        if let Some(weapon) = player_char.weapons.get_mut(weapon_index) {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.add_space(5.0);
                egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                    egui::Grid::new(&self.id_grid_weapon)
                        .num_columns(2)
                        .spacing([8.0, 4.0])
                        .show(ui, |ui| {
                            ui.label("Name:");
                            ui.text_edit_singleline(&mut weapon.name);
                            ui.end_row();

                            ui.label("Req collectable:");
                            Self::show_table_item_id_editor(
                                ui,
                                "weapon_req_collectable_id",
                                &mut weapon.req_collectable_id,
                                &assets.tables.collectable
                            );
                            ui.end_row();

                            // === shooting =================================
                            ui.end_row();
                            ui.label("Shooting");
                            ui.separator();
                            ui.end_row();

                            ui.label("   Animation:");
                            Self::show_asset_id_editor(
                                ui,
                                "weapon_shoot_animation_id",
                                assets.animations,
                                &self.sorted_animation_ids,
                                &mut weapon.shoot.anim_id
                            );
                            ui.end_row();

                            ui.label("   Shoot SFX:");
                            Self::show_optional_asset_id_editor(
                                ui,
                                "weapon_shoot_sfx_id",
                                assets.sfxs,
                                &self.sorted_sfx_ids,
                                &mut weapon.shoot.sfx_id
                            );
                            ui.end_row();

                            ui.label("   Duration:");
                            ui.horizontal(|ui| {
                                Self::show_number_editor(ui, &mut weapon.shoot.anim_len, 0..=u8::MAX);
                                ui.label("frames");
                            });
                            ui.end_row();

                            ui.label("   Crouch X:");
                            ui.horizontal(|ui| {
                                Self::show_number_editor(ui, &mut weapon.shoot.crouch_x, i8::MIN..=i8::MAX);
                                ui.label("px");
                            });
                            ui.end_row();

                            ui.label("   Crouch Y:");
                            ui.horizontal(|ui| {
                                Self::show_number_editor(ui, &mut weapon.shoot.crouch_y, i8::MIN..=i8::MAX);
                                ui.label("px");
                            });
                            ui.end_row();

                            // === bullets ==================================
                            ui.end_row();
                            ui.label("Bullets");
                            ui.separator();
                            ui.end_row();

                            ui.label("   Animation:");
                            Self::show_asset_id_editor(
                                ui,
                                "weapon_bullet_animation_id",
                                assets.animations,
                                &self.sorted_animation_ids,
                                &mut weapon.bullet.anim_id
                            );
                            ui.end_row();

                            ui.label("   Hit Animation:");
                            Self::show_optional_asset_id_editor(
                                ui,
                                "weapon_bullet_hit_anim_id",
                                assets.animations,
                                &self.sorted_animation_ids,
                                &mut weapon.bullet.hit_anim_id
                            );
                            ui.end_row();

                            ui.label("   Hit SFX:");
                            Self::show_optional_asset_id_editor(
                                ui,
                                "weapon_bullet_hit_sfx_id",
                                assets.sfxs,
                                &self.sorted_sfx_ids,
                                &mut weapon.bullet.hit_sfx_id
                            );
                            ui.end_row();

                            ui.label("   Ammo pickup:");
                            Self::show_table_item_id_editor(
                                ui,
                                "weapon_ammo_pickup",
                                &mut weapon.bullet.ammo_pickup_id,
                                &assets.tables.pickup
                            );
                            ui.end_row();

                            ui.label("   Initial delay:");
                            ui.horizontal(|ui| {
                                Self::show_number_editor(
                                    ui,
                                    &mut weapon.bullet.initial_delay,
                                    0..=u8::MAX
                                );
                                ui.label("frames");
                            });
                            ui.end_row();

                            ui.label("   Repeat interval:");
                            ui.horizontal(|ui| {
                                Self::show_number_editor(ui, &mut weapon.bullet.interval, 0..=u8::MAX);
                                ui.label("frames");
                            });
                            ui.end_row();

                            ui.label("   Damage:");
                            ui.horizontal(|ui| {
                                Self::show_number_editor(ui, &mut weapon.bullet.damage, 0..=u16::MAX);
                            });
                            ui.end_row();

                            ui.label("   Spawn X:");
                            ui.horizontal(|ui| {
                                Self::show_number_editor(ui, &mut weapon.bullet.x, i8::MIN..=i8::MAX);
                                ui.label("px");
                            });
                            ui.end_row();

                            ui.label("   Spawn Y:");
                            ui.horizontal(|ui| {
                                Self::show_number_editor(ui, &mut weapon.bullet.y, i8::MIN..=i8::MAX);
                                ui.label("px");
                            });
                            ui.end_row();

                            ui.label("   Move Dx:");
                            ui.horizontal(|ui| {
                                Self::show_number_editor(ui, &mut weapon.bullet.dx, i16::MIN..=i16::MAX);
                                ui.label("px / (256 frames)");
                            });
                            ui.end_row();

                            ui.label("   Move Dy:");
                            ui.horizontal(|ui| {
                                Self::show_number_editor(ui, &mut weapon.bullet.dy, i16::MIN..=i16::MAX);
                                ui.label("px / (256 frames)");
                            });
                            ui.end_row();
                        });
                });
            });
        } else {
            egui::CentralPanel::default().show(ui, |ui| {
                ui.label("Invalid weapon selection!");
            });
        }
    }

    fn clear_sorted_ids(&mut self) {
        self.sorted_animation_ids.clear();
        self.sorted_sfx_ids.clear();
    }

    fn sort_ids(&mut self, assets: &PlayerCharEditorAssetLists) {
        if self.sorted_animation_ids.len() != assets.asset_ids.animations.len() ||
            self.sorted_sfx_ids.len() != assets.asset_ids.sfxs.len() {
            self.clear_sorted_ids();
        }

        if self.sorted_animation_ids.is_empty() {
            assets.asset_ids.animations.copy_to(&mut self.sorted_animation_ids);
            data_store::utils::sort_asset_ids_by_name(&mut self.sorted_animation_ids, assets.animations);
        }
        if self.sorted_sfx_ids.is_empty() {
            assets.asset_ids.sfxs.copy_to(&mut self.sorted_sfx_ids);
            data_store::utils::sort_asset_ids_by_name(&mut self.sorted_sfx_ids, assets.sfxs);
        }
    }

    fn show(
        &mut self,
        ui: &mut egui::Ui,
        wc: &mut WindowContext,
        dialogs: &mut Dialogs,
        player_char: &mut PlayerChar,
        assets: &PlayerCharEditorAssetLists,
    ) {
        self.sort_ids(assets);
        self.ensure_valid_selection(player_char);
        if dialogs.confirmation_dialog.open && dialogs.confirmation_dialog.show(wc) == DialogResult::Yes {
            self.remove_item(player_char, self.remove_item);
            self.remove_item = ItemRef::None;
        }

        self.show_menu_bar(ui, wc, dialogs, player_char);

        egui::Panel::left(self.id_trees_panel).resizable(false).exact_size(200.0).show(ui, |ui| {
            ui.add_space(5.0);
            egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
                let action = self.show_suits_tree(ui, wc, dialogs, player_char);
                self.run_action(wc, dialogs, player_char, action, assets);
                let action = self.show_weapons_tree(ui, wc, dialogs, player_char);
                self.run_action(wc, dialogs, player_char, action, assets);
            });
        });

        match self.selected_item {
            ItemRef::Suit(index) => { self.show_suit_editor_tab(ui, wc, dialogs, player_char, index, assets); }
            ItemRef::Weapon(index) => { self.show_weapon_editor_tab(ui, wc, dialogs, player_char, index, assets); }
            ItemRef::None => {
                egui::CentralPanel::default().show(ui, |ui| {
                    ui.label("Nothing selected!");
                });
            }
        }
    }
}
