use std::ptr::NonNull;

use hudhook::imgui::{TableColumnSetup, Ui};

use debug::UiExt;
use eldenring::cs::*;

use super::{DebugDisplay, DisplayUiExt};

fn nested_ptr<T: DebugDisplay>(ui: &Ui, label: impl AsRef<str>, ptr: Option<NonNull<T>>) {
    ui.nested_opt(label, ptr.map(|value| unsafe { value.as_ref() }));
}

fn format_play_time(milliseconds: u32) -> String {
    let hours = milliseconds / 3_600_000;
    let minutes = (milliseconds % 3_600_000) / 60_000;
    let seconds = (milliseconds % 60_000) / 1000;
    let millis = milliseconds % 1000;
    format!("{hours}:{minutes:02}:{seconds:02}.{millis:03} ({milliseconds} ms)")
}

fn byte_preview(bytes: &[u8], max: usize) -> String {
    let shown = bytes
        .iter()
        .take(max)
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ");

    if bytes.len() > max {
        format!("{shown} … ({} bytes)", bytes.len())
    } else {
        format!("{shown} ({} bytes)", bytes.len())
    }
}

impl DebugDisplay for GameDataMan {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Trophy Data", &self.trophy_equip_data);
        ui.display("Death Count", self.death_count);
        ui.display("Play Time", format_play_time(self.play_time));
        ui.display("NG Level", self.ng_lvl);
        ui.display("Random Appearance Lot Slot", self.random_appear_lot_slot);
        ui.debug("Post Map Load Chr Type", self.post_map_load_chr_type);
        ui.nested("Gaitem Game Data", &self.gaitem_game_data);

        ui.separator();
        ui.display("Has Bloodstain", self.has_bloodstain);
        nested_ptr(ui, "Bloodstain", self.bloodstain);
        ui.display("Bloodstain Entity ID", self.bloodstain_entity_id);

        ui.separator();
        ui.display("Boss Fight Active", self.boss_fight_active);
        ui.display("Is In Boss Fight", self.is_in_boss_fight);
        ui.debug("Boss Fight Timer", self.boss_fight_timer.time);
        ui.display("Boss Health Bar Entity ID", self.boss_health_bar_entity_id);
        ui.display(
            "Boss Health Bar NPC Param ID",
            self.boss_health_bar_npc_param_id,
        );
        ui.display("White Phantom Count", self.white_phantom_count);

        ui.separator();
        ui.debug("Death State", self.death_state);
        ui.display("Just Died", self.just_died);
        ui.display(
            "Has Death Preventing Effect",
            self.has_death_preventing_effect,
        );

        ui.separator();
        ui.display("Request Full Recovery", self.request_full_recovery);
        ui.display("Request Restore HP", self.request_restore_hp);
        ui.display(
            "Award Phantom Great Rune",
            self.award_phantom_great_rune_requested,
        );
        ui.display(
            "Award Rebreak-in Item",
            self.award_rebreak_in_item_requested,
        );

        ui.separator();
        ui.display("Net Penalty Requested", self.net_penalty_requested);
        ui.display("Net Penalty Points", self.net_penalty_points);
        ui.display("Net Penalty Item Cooldown", self.is_net_penalized);
        ui.display(
            "Net Penalty Limit Time",
            self.net_penalty_forgive_item_limit_time,
        );

        ui.nested("Main Player Game Data", &self.main_player_game_data);
        nested_ptr(
            ui,
            "Quickmatch Scaling Baseline Game Data",
            self.quickmatch_scaling_baseline_game_data,
        );
        ui.nested("Game Settings", &self.game_settings);
        ui.nested("Game Version Data", &self.game_version_data);
        nested_ptr(ui, "Menu System Save/Load", self.menu_system_save_load);
        nested_ptr(ui, "Menu Profile Save/Load", self.menu_profile_save_load);
        nested_ptr(ui, "Key Config Save/Load", self.key_config_save_load);
        nested_ptr(ui, "Profile Summary", self.profile_summary);

        ui.separator();
        ui.display("DLC List Up To Date", self.dlc_list_up_to_date);
        ui.header("DLC List", || {
            ui.table(
                "dlc-list",
                [
                    TableColumnSetup::new("Index"),
                    TableColumnSetup::new("DLC ID"),
                ],
                self.pending_dlc_list.iter(),
                |ui, i, dlc_id| {
                    ui.table_next_column();
                    ui.text(format!("{}", i));
                    ui.table_next_column();
                    ui.text(format!("{}", dlc_id));
                },
            );
        });

        ui.list(
            "Player Game Data List",
            self.player_game_data_list.iter(),
            |ui, i, item| ui.nested(format!("Slot {}", i), item),
        );

        ui.header("Remote Game Data States", || {
            ui.table(
                "remote-game-data-states",
                [
                    TableColumnSetup::new("Slot"),
                    TableColumnSetup::new("State"),
                ],
                self.remote_game_data_states.iter(),
                |ui, i, state| {
                    ui.table_next_column();
                    ui.text(format!("Slot {}", i));
                    ui.table_next_column();
                    ui.text(format!("{:?}", state));
                },
            );
        });

        ui.header("Leave Requests", || {
            ui.table(
                "game-data-leave-requests",
                [
                    TableColumnSetup::new("Slot"),
                    TableColumnSetup::new("Requested"),
                ],
                self.leave_requests.iter(),
                |ui, i, req| {
                    ui.table_next_column();
                    ui.text(format!("Slot {}", i));
                    ui.table_next_column();
                    ui.text(format!("{}", req));
                },
            );
        });

        ui.list(
            "Session Player Game Data",
            self.session_player_game_data_list.iter(),
            |ui, i, item| ui.nested_opt(format!("Slot {}", i), item.as_ref()),
        );
    }
}

impl DebugDisplay for GameDataManBloodstainData {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("MSB Position", self.msb_position);
        ui.nested("Rotation", self.rotation);
        ui.display("Base Hero Point 2", self.base_hero_point_2);
        ui.display("Rune Count", self.rune_count);
        ui.debug("Block ID", self.block_id);
        ui.display("Has Death Blight Effect", self.has_death_blight_effect);
    }
}

impl DebugDisplay for CSMenuSaveLoad {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Definition ID", self.definition_id);
        ui.display("Definition Variant", self.definition_variant);
        ui.display("Serialized Capacity Bytes", self.serialized_capacity_bytes);
    }
}

impl DebugDisplay for FaceData {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Face Data Buffer", &self.face_data_buffer);
    }
}

impl DebugDisplay for FaceDataBuffer {
    fn render_debug(&self, ui: &Ui) {
        ui.display_copiable("Magic", byte_preview(&self.magic, self.magic.len()));
        ui.display("Version", self.version);
        ui.display("Buffer Size", self.buffer_size);
        ui.display_copiable("Buffer Preview", byte_preview(&self.buffer, 32));
    }
}

impl DebugDisplay for CSMenuFaceData {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Header", self.header);
        ui.nested("Face Data Buffer", &self.face_data_buffer);
        ui.display("Footer", self.footer);
    }
}

impl DebugDisplay for CSMenuSystemSaveLoadFaceChunk {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Entry Count", self.entries.len());
        ui.display("Entry Capacity", self.entries.capacity());
        ui.list("Face Entries", self.entries.iter(), |ui, i, face| {
            ui.nested(format!("Face {i}"), face);
        });
    }
}

impl DebugDisplay for DetailStatusViewStateEntry {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Selected Index", self.selected_index());
        ui.display("Scroll Position", self.scroll_position());
        ui.display("Packed View State", self.packed_view_state());
        ui.display("Active", self.active);
    }
}

impl DebugDisplay for DetailStatusViewStateSaveData {
    fn render_debug(&self, ui: &Ui) {
        ui.table(
            "detail-status-view-state",
            [
                TableColumnSetup::new("Index"),
                TableColumnSetup::new("Selected"),
                TableColumnSetup::new("Scroll"),
                TableColumnSetup::new("Packed"),
                TableColumnSetup::new("Active"),
            ],
            self.entries.iter(),
            |ui, i, entry| {
                ui.table_next_column();
                ui.text(format!("{i}"));
                ui.table_next_column();
                ui.text(format!("{}", entry.selected_index()));
                ui.table_next_column();
                ui.text(format!("{}", entry.scroll_position()));
                ui.table_next_column();
                ui.text(format!("{}", entry.packed_view_state()));
                ui.table_next_column();
                ui.text(format!("{}", entry.active));
            },
        );
    }
}

impl DebugDisplay for MenuInputHistoryPayloadStorage {
    fn render_debug(&self, ui: &Ui) {
        let bytes = self.as_bytes();
        ui.display("Byte Count", bytes.len());
        ui.display(
            "Non-Zero Byte Count",
            bytes.iter().filter(|byte| **byte != 0).count(),
        );
        ui.display_copiable("Payload Preview", byte_preview(bytes, 64));
    }
}

impl DebugDisplay for MenuInputHistorySaveData {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Header Word 0", self.header_word_0);
        ui.display("Header Word 2", self.header_word_2);
        ui.display("Payload Byte Count", self.payload_byte_count);
        ui.nested("Payload Storage", &self.payload_storage);
    }
}

impl DebugDisplay for MenuTitleFlowSaveData {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Save Slot", self.save_slot);
        ui.display("Flags", self.flags);
    }
}

impl<T: DebugDisplay> DebugDisplay for CSMenuSimpleSaveDataChunk<T> {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Data", &self.data);
    }
}

impl DebugDisplay for MenuSortState {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Raw", self.raw());
        ui.display("High Bit Flag", self.high_bit_flag());
        ui.display("Value Bits", self.value_bits());
    }
}

impl DebugDisplay for MenuSortStates {
    fn render_debug(&self, ui: &Ui) {
        ui.table(
            "menu-sort-states",
            [
                TableColumnSetup::new("Index"),
                TableColumnSetup::new("Raw"),
                TableColumnSetup::new("High Bit"),
                TableColumnSetup::new("Value Bits"),
            ],
            self.as_ref().iter(),
            |ui, i, state| {
                ui.table_next_column();
                ui.text(format!("{i}"));
                ui.table_next_column();
                ui.text(format!("{}", state.raw()));
                ui.table_next_column();
                ui.text(format!("{}", state.high_bit_flag()));
                ui.table_next_column();
                ui.text(format!("{}", state.value_bits()));
            },
        );
    }
}

impl DebugDisplay for CSMenuSystemSaveLoad {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Base", &self.base);
        ui.nested("Face Chunk", &self.face_chunk);
        ui.nested("Title Flow Save Data", &self.title_flow_save_data);
        ui.nested("Detail Status View State", &self.detail_status_view_state);
        ui.nested("Menu Input History", &self.menu_input_history);
        ui.nested("Menu Sort States", &self.menu_sort_states);
    }
}

impl DebugDisplay for ProfileSummaryCharacterName {
    fn render_debug(&self, ui: &Ui) {
        ui.display_copiable("Name", self.to_string_lossy());
        ui.display("UTF-16 Code Units", self.code_units_until_nul().len());
    }
}

impl DebugDisplay for ProfileSummaryRecord {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Name", &self.name);
        ui.display("Level", self.level);
        ui.display("Play Time", format_play_time(self.play_time));
        ui.display("Rune Memory", self.rune_memory);
        ui.display("Map", self.map);
        ui.nested("Face Data", &self.face_data);
        ui.nested("Character Assembly", &self.chr_asm);
        ui.display("Gender", self.gender);
        ui.display("Archetype", self.archetype);
        ui.display("Starting Gift", self.starting_gift);
        ui.display("Player Game Data Unk C4", self.player_game_data_unkc4);
    }
}

impl DebugDisplay for ProfileSummary {
    fn render_debug(&self, ui: &Ui) {
        ui.table(
            "profile-summary-records",
            [
                TableColumnSetup::new("Slot"),
                TableColumnSetup::new("Name"),
                TableColumnSetup::new("Level"),
                TableColumnSetup::new("Play Time"),
                TableColumnSetup::new("Runes"),
                TableColumnSetup::new("Map"),
            ],
            self.records.iter(),
            |ui, i, record| {
                ui.table_next_column();
                ui.text(format!("{i}"));
                ui.table_next_column();
                ui.text(record.name.to_string_lossy());
                ui.table_next_column();
                ui.text(format!("{}", record.level));
                ui.table_next_column();
                ui.text(format_play_time(record.play_time));
                ui.table_next_column();
                ui.text(format!("{}", record.rune_memory));
                ui.table_next_column();
                ui.text(format!("{}", record.map));
            },
        );

        ui.list("Profile Records", self.records.iter(), |ui, i, record| {
            ui.nested(format!("Slot {i}"), record);
        });
    }
}

impl DebugDisplay for GameSettings {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Camera Speed", self.camera_speed);
        ui.display("Rumble Strength", self.controller_rumble_strength);
        ui.display("Brightness", self.brightness);

        ui.separator();
        ui.display("Master Volume", self.master_volume);
        ui.display("Music Volume", self.music_volume);
        ui.display("SFX Volume", self.sfx_volume);
        ui.display("Voice Volume", self.voice_volume);
        ui.display("Sound Type", self.sound_type);

        ui.separator();
        ui.debug("Display Blood", self.display_blood);
        ui.debug("HUD Type", self.hud_type);
        ui.debug("Performance", self.performance_setting);
        ui.display("Ray Tracing", self.enable_ray_tracing);
        ui.display("Start Offline", self.start_offline);
        ui.display("Cross Region Play", self.enable_cross_region_play);
        ui.display("Show Subtitles", self.show_subtitles);
        ui.display("Show Gamer Tags", self.show_gamer_tags);

        ui.separator();
        ui.text("Control Settings");
        ui.display("Reverse Camera X", self.reverse_camera_xaxis);
        ui.display("Reverse Camera Y", self.reverse_camera_yaxis);
        ui.display("Auto Lock-on", self.auto_lock_on);
        ui.display("Camera Auto Wall Recovery", self.camera_auto_wall_recovery);
        ui.display("Camera Auto Rotation", self.camera_auto_rotation);
        ui.display("Reset Camera Y Axis", self.reset_camera_yaxis);
        ui.display("Jump Button L3", self.jump_button_l3);
        ui.display("Manual Attack Aiming", self.manual_attack_aiming);
        ui.display("Auto Target", self.auto_target);

        ui.separator();
        ui.text("Misc Settings");
        ui.display("Cinematic Effects", self.cinematic_effects);
        ui.display("Voice Chat", self.voice_chat);
        ui.display("Send Summon Signs", self.send_summon_signs);
        ui.display("Show Tutorials", self.show_tutorials);
        ui.display("Mark New Items", self.mark_new_items);
        ui.display("Show Recent Items", self.show_recent_items);
        ui.display("Unused GR System 103000", self.unused_gr_system_103000);

        ui.separator();
        ui.text("HDR Settings");
        ui.display("HDR Brightness", self.hdr_brightness);
        ui.display("HDR Max Brightness", self.hdr_max_brightness);
        ui.display("HDR Contrast", self.hdr_contrast);
    }
}

impl DebugDisplay for CSGaitemGameData {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Entries Count", self.gaitem_entries.len());
        ui.header("Gaitem Entries", || {
            ui.table(
                "gaitem-entries",
                [
                    TableColumnSetup::new("Index"),
                    TableColumnSetup::new("Item ID"),
                    TableColumnSetup::new("Is already acquired"),
                ],
                self.gaitem_entries.iter(),
                |ui, i, entry| {
                    ui.table_next_column();
                    ui.text(format!("{}", i));
                    ui.table_next_column();
                    ui.text(format!("{:?}", entry.item_id));
                    ui.table_next_column();
                    ui.text(format!("{}", entry.already_acquired));
                },
            );
        });
    }
}

impl DebugDisplay for GameVersionData {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Game Data Version", self.game_data_version);
        ui.display(
            "Last Saved Game Data Version",
            self.last_saved_game_data_version,
        );
        ui.display(
            "Saved Game Data Version is the Latest",
            self.saved_game_data_version_is_the_latest,
        );
        ui.display("Unused", self.unused);
    }
}

impl DebugDisplay for TrophyEquipData {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Legendary Armaments", self.weapon_stats);
        ui.nested("Legendary Spells & Ashes", self.goods_stats);
        ui.nested("Legendary Talismans", self.accessory_stats);
    }
}

impl DebugDisplay for TrophyWeaponStats<[u8; 0x10]> {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Sword of Night and Flame", self.sword_of_night_and_flame());
        ui.display("Dark Moon Greatsword", self.dark_moon_greatsword());
        ui.display(
            "Marais Executioner's Sword",
            self.marais_executioners_sword(),
        );
        ui.display("Golden Order Greatsword", self.golden_order_greatsword());
        ui.display("Ruins Greatsword", self.ruins_greatsword());
        ui.display("Grafted Blade Greatsword", self.grafted_blade_greatsword());
        ui.display("Eclipse Shotel", self.eclipse_shotel());
        ui.display("Devourer's Scepter", self.devourers_scepter());
        ui.display("Bolt of Gransax", self.bolt_of_gransax());
    }
}

impl DebugDisplay for TrophyGoodsStats<[u8; 0x10]> {
    fn render_debug(&self, ui: &Ui) {
        ui.header("Sorceries & Incantations", || {
            ui.display("Comet Azur", self.comet_azur());
            ui.display("Founding Rain of Stars", self.founding_rain_of_stars());
            ui.display("Stars of Ruin", self.stars_of_ruin());
            ui.display("Ranni's Dark Moon", self.rannis_dark_moon());
            ui.display("Flame of the Fell God", self.flame_of_the_fell_god());
            ui.display("Elden Stars", self.elden_stars());
            ui.display("Greyoll's Roar", self.greyolls_roar());
        });

        ui.header("Legendary Ashes", || {
            ui.display("Black Knife Tiche", self.black_knife_tiche());
            ui.display("Mimic Tear Ashes", self.mimic_tear_ashes());
            ui.display("Cleanrot Knight Finlay", self.cleanrot_knight_finlay());
            ui.display(
                "Ancient Dragon Knight Kristoff",
                self.ancient_dragon_knight_kristoff(),
            );
            ui.display("Redmane Knight Ogha", self.redmane_knight_ogha());
            ui.display("Lhutel the Headless", self.lhutel_the_headless());
        });
    }
}

impl DebugDisplay for TrophyAccessoryStats<[u8; 0x10]> {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Erdtree's Favor +2", self.erdtree_favor_p2());
        ui.display("Radagon's Soreseal", self.radagons_soreseal());
        ui.display("Moon of Nokstella", self.moon_of_nokstella());
        ui.display("Marika's Soreseal", self.marikas_soreseal());
        ui.display("Old Lord's Talisman", self.old_lords_talisman());
        ui.display("Radagon Icon", self.radagon_icon());
        ui.display("Godfrey Icon", self.godfrey_icon());
        ui.display("Dragoncrest Greatshield", self.dragoncrest_greatshield());
    }
}
