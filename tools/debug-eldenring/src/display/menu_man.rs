use std::ptr::NonNull;

use debug::UiExt;
use eldenring::cs::*;
use hudhook::imgui::{TableColumnSetup, Ui};

use super::{DebugDisplay, DisplayUiExt};

fn nested_ptr<T: DebugDisplay>(ui: &Ui, label: impl AsRef<str>, ptr: Option<NonNull<T>>) {
    ui.nested_opt(label, ptr.map(|value| unsafe { value.as_ref() }));
}

fn optional_ptr<T>(ui: &Ui, label: impl AsRef<str>, ptr: Option<NonNull<T>>) {
    let label = label.as_ref();
    match ptr {
        Some(value) => ui.display_copiable(label, format!("{:p}", value.as_ptr())),
        None => ui.text(format!("{label}: None")),
    }
}

impl DebugDisplay for DLReferenceCountObjectHeader {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Reference Count", self.reference_count);
    }
}

impl DebugDisplay for PlayerStatusCalculator {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Data", &self.data);
    }
}

impl DebugDisplay for PlayerStatusCalculatorData {
    fn render_debug(&self, ui: &Ui) {
        ui.header("Vitals", || {
            ui.text(format!(
                "HP: {}/{}    FP: {}/{}    Stamina: {}",
                self.current_hp,
                self.current_max_hp,
                self.current_fp,
                self.current_max_fp,
                self.current_max_stamina
            ));
            ui.text(format!(
                "Equip Load: {:.2}/{:.2} (type {})",
                self.equipment_weight, self.max_equip_load, self.weight_type
            ));
        });

        ui.header("Effective Stats", || {
            ui.display("Vigor", self.effective_vigor);
            ui.display("Mind", self.effective_mind);
            ui.display("Endurance", self.effective_endurance);
            ui.display("Vitality", self.effective_vitality);
            ui.display("Strength", self.effective_strength);
            ui.display("Dexterity", self.effective_dexterity);
            ui.display("Intelligence", self.effective_intelligence);
            ui.display("Faith", self.effective_faith);
            ui.display("Arcane", self.effective_arcane);
        });

        ui.display(
            "All Item Weight Change Rate",
            self.all_item_weight_change_rate,
        );
        ui.display("Toughness Damage Cut Rate", self.toughness_damage_cut_rate);
        ui.display("Item Drop Rate", self.item_drop_rate);
        ui.display("Magic Slots Count", self.magic_slots_count);
        ui.display("Rune Count", self.rune_count);

        ui.nested("Attack", &self.attack);
        ui.nested("Defense", &self.defense);
        ui.nested("Damage Negation", &self.damage_negation);
        ui.nested("Total Resistance", &self.total_resistance);
        ui.nested("Resistance", &self.resistance);
    }
}

impl DebugDisplay for PlayerStatusCalculatorAttack {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Left Primary", self.left_armament_primary);
        ui.display("Left Secondary", self.left_armament_secondary);
        ui.display("Left Tertiary", self.left_armament_tertiary);
        ui.display("Right Primary", self.right_armament_primary);
        ui.display("Right Secondary", self.right_armament_secondary);
        ui.display("Right Tertiary", self.right_armament_tertiary);
    }
}

impl DebugDisplay for PlayerStatusDefense {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Physical", self.physical);
        ui.display("Strike", self.strike);
        ui.display("Slash", self.slash);
        ui.display("Pierce", self.pierce);
        ui.display("Magic", self.magic);
        ui.display("Fire", self.fire);
        ui.display("Lightning", self.lightning);
        ui.display("Holy", self.holy);
    }
}

impl DebugDisplay for PlayerStatusCalculatorDamageNegation {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Physical", self.physical);
        ui.display("Slash", self.slash);
        ui.display("Strike", self.strike);
        ui.display("Pierce", self.pierce);
        ui.display("Magic", self.magic);
        ui.display("Fire", self.fire);
        ui.display("Lightning", self.lightning);
        ui.display("Holy", self.holy);
    }
}

impl DebugDisplay for StatusEffectFloats {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Poison", self.poison);
        ui.display("Disease", self.disease);
        ui.display("Blood", self.blood);
        ui.display("Curse", self.curse);
        ui.display("Freeze", self.freeze);
        ui.display("Sleep", self.sleep);
        ui.display("Madness", self.madness);
    }
}

impl DebugDisplay for CSMenuManImp {
    fn render_debug(&self, ui: &Ui) {
        nested_ptr(ui, "Menu Data", self.menu_data);
        ui.display("Disable Mouse Cursor", self.disable_mouse_cursor);
        nested_ptr(ui, "Popup Menu", self.popup_menu);
        optional_ptr(ui, "Window Job", self.window_job);

        ui.separator();
        ui.display("Stream Flag F0", self.stream_flag_f0);
        ui.display("Stream Flag F1", self.stream_flag_f1);
        ui.nested("Dword Lookup Vector F8", &self.dword_lookup_vector_f8);
        ui.display("Menu Text Dirty", self.menu_text_dirty_138);
        ui.display("Menu Text Shadow Active", self.menu_text_shadow_active_139);
        ui.display("Disable Save Menu", self.disable_save_menu);
        ui.display(
            "Popup Selection Result Index",
            self.popup_selection_result_index,
        );
        ui.display("Reset Preserved Word 330", self.reset_preserved_word_330);
        ui.display("Reset Preserved Word 338", self.reset_preserved_word_338);

        ui.separator();
        ui.nested("Player Menu Ctrl", &self.player_menu_ctrl);
        ui.nested("Null Player Menu Ctrl", &self.null_player_menu_ctrl);
        ui.nested("Back Screen Data", &self.back_screen_data);
        ui.nested("Loading Screen Data", &self.loading_screen_data);
        nested_ptr(
            ui,
            "System Announce View Model",
            self.system_announce_view_model,
        );
        ui.nested("Update Task", &self.update_task);
        ui.nested("Tail Flags", &self.tail_flags_898);
    }
}

impl DebugDisplay for CSMenuManDwordLookupVectorF8 {
    fn render_debug(&self, ui: &Ui) {
        ui.table(
            "cs-menu-man-dword-lookup-vector-f8",
            [
                TableColumnSetup::new("Index"),
                TableColumnSetup::new("Value"),
            ],
            self.iter(),
            |ui, i, value| {
                ui.table_next_column();
                ui.text(format!("{i}"));
                ui.table_next_column();
                ui.text(format!("{value}"));
            },
        );
    }
}

impl DebugDisplay for CSMenuInputFlags {
    fn render_debug(&self, ui: &Ui) {
        ui.debug("Raw Flags", self);
        ui.display("Is Down", self.is_down());
        ui.display("Triggered", self.triggered());
    }
}

impl DebugDisplay for CSMenuManStateBlock90 {
    fn render_debug(&self, ui: &Ui) {
        ui.table(
            "cs-menu-input-flags",
            [
                TableColumnSetup::new("Index"),
                TableColumnSetup::new("Down"),
                TableColumnSetup::new("Triggered"),
                TableColumnSetup::new("Raw"),
            ],
            self.input_flags.iter(),
            |ui, i, flags| {
                ui.table_next_column();
                ui.text(format!("{i}"));
                ui.table_next_column();
                ui.text(format!("{}", flags.is_down()));
                ui.table_next_column();
                ui.text(format!("{}", flags.triggered()));
                ui.table_next_column();
                ui.text(format!("{flags:?}"));
            },
        );
    }
}

impl DebugDisplay for CSMenuManStateBlock7a0 {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Active", self.active);
        ui.display_copiable("Primary Text", &self.primary_text);
        ui.display_copiable("Secondary Text", &self.secondary_text);
        ui.display_copiable("Status Text", &self.status_text);
    }
}

impl DebugDisplay for CSMenuManTailFlags898 {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Top Menu Debug Enabled", self.top_menu_debug_enabled());
        ui.display(
            "Help Menu Workaround Enabled",
            self.help_menu_workaround_enabled(),
        );
    }
}

impl DebugDisplay for CSMenuDataDisplayGhostRequest {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Request Word", self.request_word);
        ui.display("Sentinel Byte 02", self.sentinel_byte_02);
    }
}

impl DebugDisplay for CSMenuData {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Text Entry 8", &self.text_entry_8);
        ui.display("Yes/No Sign Menu Result", self.yes_no_sign_menu_result);
        ui.display("Mode 54", self.mode_54);
        ui.display("Show Steam Names", self.show_steam_names);
        ui.display("Name Display Flags Tail", self.name_display_flags_tail);
        ui.nested("Gaitem Use State", &self.menu_gaitem_use_state);
        ui.nested("Text Entry A8", &self.text_entry_a8);
    }
}

impl DebugDisplay for CSMenuGaitemUseState {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Source Word 10", self.source_word_10);
        ui.display("Request Word 14", self.request_word_14);
    }
}

impl DebugDisplay for CSMenuDataTextEntry {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Entry ID", self.entry_id);
        ui.display_copiable("Text", &self.text);
        ui.display("Active", self.active);
    }
}

impl DebugDisplay for MenuViewModel {
    fn render_debug(&self, ui: &Ui) {
        ui.text("Known layout: view-model vtable prefix only.");
    }
}

impl DebugDisplay for WorldMapViewModel {
    fn render_debug(&self, ui: &Ui) {
        ui.text("Known layout: world-map view-model vtable prefix only.");
    }
}

impl DebugDisplay for MultiPlayViewModel {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Base", &self.base);
        ui.display_copiable("Password", &self.password);
        ui.display_copiable("Vow Type", &self.vow_type);
        ui.display_copiable("Humanity Count", &self.humanity_count);
    }
}

impl DebugDisplay for CSPopupMenu {
    fn render_debug(&self, ui: &Ui) {
        ui.pointer("Menu Man", self.menu_man.as_ptr());
        optional_ptr(ui, "Current Top Menu Job", self.current_top_menu_job);
        ui.display("Current Talk ID", self.current_talk_id);
        ui.nested("Input Buffer", &self.input_buffer_184);
        ui.nested("Popup Queue", &self.popup_queue);
        ui.display("Popup List Runtime Flag", self.popup_list_runtime_flag);

        ui.separator();
        optional_ptr(
            ui,
            "World Map Tile Back Reader",
            self.world_map_tile_back_reader,
        );
        nested_ptr(ui, "World Map View Model", self.world_map_view_model);
        nested_ptr(
            ui,
            "Gesture Equip View Model",
            self.gesture_equip_view_model,
        );
        nested_ptr(ui, "Multi Play View Model", self.multi_play_view_model);
        nested_ptr(ui, "Keyword View Model", self.keyword_view_model);
        nested_ptr(ui, "Network View Model", self.network_view_model);
        nested_ptr(ui, "Main Top View Model", self.main_top_view_model);
        nested_ptr(ui, "Tutorial View Model", self.tutorial_view_model);
        nested_ptr(ui, "Matching View Model", self.matching_view_model);

        ui.separator();
        ui.display("Show Failed To Save", self.show_failed_to_save);
        ui.nested("Fade State", &self.fade_state);
    }
}

impl DebugDisplay for PopupMenuFadeState {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Primary Value", self.primary_value);
        ui.display("Secondary Value", self.secondary_value);
    }
}

impl DebugDisplay for InputData {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Count", self.count);
    }
}

impl DebugDisplay for PopupMenuTextState {
    fn render_debug(&self, ui: &Ui) {
        ui.display_copiable("Text", &self.text);
    }
}

impl DebugDisplay for PopupMenuInlineSlot {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Index", self.index);
    }
}

impl DebugDisplay for PopupMenuInlineEntry {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Message ID", self.message_id);
        ui.display("Dedupe Key", self.dedupe_key);
        ui.display("Sort Key", self.sort_key);
        ui.display("Payload", self.payload);
    }
}

impl DebugDisplay for PopupMenuInputBuffer {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Prompt Message ID", self.prompt_message_id);
        ui.display("First Choice Message ID", self.first_choice_message_id);
        ui.display("Second Choice Message ID", self.second_choice_message_id);
        ui.debug("Prompt Text Source", self.prompt_text_source());
        ui.display("Raw Prompt Text Source", self.prompt_text_source);
        ui.debug("Choice Text Source", self.choice_text_source());
        ui.display("Raw Choice Text Source", self.choice_text_source);
        ui.display("First Choice Disabled", self.first_choice_disabled);
        ui.display("Second Choice Disabled", self.second_choice_disabled);
        ui.display("Selection Result Index", self.selection_result_index);
        ui.display("Cancel Result Value", self.cancel_result_value);
    }
}

impl DebugDisplay for MenuStringDeque {
    fn render_debug(&self, ui: &Ui) {
        ui.text("Known layout: native deque storage; no public iterator is exposed yet.");
    }
}

impl DebugDisplay for CSPlayerMenuCtrl {
    fn render_debug(&self, ui: &Ui) {
        ui.debug("Selected Goods Item", self.selected_goods_item);
        ui.debug("Selected Magic Item", self.selected_magic_item);
        ui.nested("Character Menu Flags", &self.chr_menu_flags);
        ui.display("Is Sign Puddle", self.is_sign_puddle);
        ui.display("Is Break-In Multi Region", self.is_break_in_multi_region);
    }
}

impl DebugDisplay for CSChrMenuFlags {
    fn render_debug(&self, ui: &Ui) {
        ui.debug("Flags", self.flags);
        ui.display("Pause Menu State", self.flags.pause_menu_state());
    }
}

impl DebugDisplay for NullPlayerMenuCtrl {
    fn render_debug(&self, ui: &Ui) {
        ui.text(
            "Known layout: fallback controller storage only; no public state fields are exposed.",
        );
    }
}

impl DebugDisplay for BackScreenData {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Active", self.active);
    }
}

impl DebugDisplay for LoadingScreenData {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Active", self.active);
        ui.display("Transition State", self.transition_state);
        ui.display("Transition Flags", self.transition_flags);
        ui.display("Loading Screen ID", self.loading_screen_id);
        ui.display("Fade Start", self.fade_start);
        ui.display("Fade End", self.fade_end);
        ui.display("Fade Duration", self.fade_duration);
    }
}

impl DebugDisplay for FeSystemAnnounceViewModel {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Base", &self.base);
        ui.nested("Message Queue", &self.message_queue);
    }
}

impl DebugDisplay for FeSystemAnnounceViewModelMessageQueue {
    fn render_debug(&self, ui: &Ui) {
        ui.text("Known layout: native deque/map storage; no public iterator is exposed yet.");
    }
}

impl DebugDisplay for AnnounceMessage {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Is Active", self.is_active);
        ui.display_copiable("Text", &self.text);
    }
}
