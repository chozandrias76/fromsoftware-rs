use std::ptr::NonNull;

use bitfield::bitfield;
use pelite::pe64::Pe;
use shared::program::Program;

use super::{CSEzTask, CSEzUpdateTask, MenuJobBase, MenuString, OpenMenuJob, OptionalItemId};
use crate::{
    DLVector,
    dlkr::DLAllocator,
    dltx::DLString,
    dlut::DLFixedVector,
    fd4::{FD4DebugMenuNode, FD4FileCap},
    rva,
};

pub const STATUS_MESSAGE_DEMIGOD_FELLED: i32 = 1;
pub const STATUS_MESSAGE_LEGEND_FELLED: i32 = 2;
pub const STATUS_MESSAGE_GREAT_ENEMY_FELLED: i32 = 3;
pub const STATUS_MESSAGE_ENEMY_FELLED: i32 = 4;
pub const STATUS_MESSAGE_YOU_DIED: i32 = 5;
pub const STATUS_MESSAGE_HOST_VANQUISHED: i32 = 7;
pub const STATUS_MESSAGE_BLOOD_FINGER_VANQUISHED: i32 = 8;
pub const STATUS_MESSAGE_DUTY_FULL_FILLED: i32 = 9;
pub const STATUS_MESSAGE_LOST_GRACE_DISCOVERED: i32 = 11;
pub const STATUS_MESSAGE_COMMENCE: i32 = 13;
pub const STATUS_MESSAGE_VICTORY: i32 = 14;
pub const STATUS_MESSAGE_STALEMATE: i32 = 15;
pub const STATUS_MESSAGE_DEFEAT: i32 = 16;
pub const STATUS_MESSAGE_MAP_FOUND: i32 = 17;
pub const STATUS_MESSAGE_GREAT_RUNE_RESTORED: i32 = 21;
pub const STATUS_MESSAGE_GOD_SLAIN: i32 = 22;
pub const STATUS_MESSAGE_DUELIST_VANQUISHED: i32 = 23;
pub const STATUS_MESSAGE_RECUSANT_VANQUISHED: i32 = 24;
pub const STATUS_MESSAGE_INVADER_VANQUISHED: i32 = 25;
pub const STATUS_MESSAGE_FURLED_FINGER_RANK_ADVANCED: i32 = 30;
pub const STATUS_MESSAGE_FURLED_FINGER_RANK_ADVANCED2: i32 = 31;
pub const STATUS_MESSAGE_DUELIST_RANK_ADVANCED: i32 = 32;
pub const STATUS_MESSAGE_DUELIST_RANK_ADVANCED2: i32 = 33;
pub const STATUS_MESSAGE_BLOODY_FINGER_RANK_ADVANCED: i32 = 34;
pub const STATUS_MESSAGE_BLOODY_FINGER_RANK_ADVANCED2: i32 = 35;
pub const STATUS_MESSAGE_RECUSANT_RANK_ADVANCED: i32 = 36;
pub const STATUS_MESSAGE_RECUSANT_RANK_ADVANCED2: i32 = 37;
pub const STATUS_MESSAGE_HUNTER_RANK_ADVANCED: i32 = 38;
pub const STATUS_MESSAGE_HUNTER_RANK_ADVANCED2: i32 = 39;
pub const STATUS_MESSAGE_HEART_STOLEN: i32 = 40;
pub const STATUS_MESSAGE_MENU_TEXT: i32 = 41;

/// Native menu input flag table length used by `CSMenuManImp` callers.
const CS_MENU_INPUT_FLAG_COUNT: usize = 0x47;

/// Native indexed menu-manager state-word count accepted by the game setter.
const CS_MENU_MAN_INDEXED_STATE_WORD_COUNT: usize = 0x15f;
const _: usize = CS_MENU_MAN_INDEXED_STATE_WORD_COUNT;

/// Native `DLReferenceCountObject` header used when only base ref-counted
/// ownership is known.
///
/// Source of layout: ref-counted menu/job structs in this crate share vtable +
/// reference-count header layout, and Ghidra decompilation of `CSMenuManImp::Unref`
/// calls `DLUT::DLReferenceCountObject::~DLReferenceCountObject` on these slots.
#[repr(C)]
pub struct DLReferenceCountObjectHeader {
    vftable: usize,
    pub reference_count: u32,
    _pad_00c: u32,
}

type ReferenceCountedObjectSlot = Option<NonNull<DLReferenceCountObjectHeader>>;

/// Native `CS::NullPlayerMenuCtrl` inline fallback controller.
///
/// Source of name: RTTI. The `CSMenuManImp` constructor writes this vtable at
/// `CSMenuManImp+0x6a8`, clears the native state word at `+0x8`, and uses the
/// fixed-vector helper on the aligned block at `+0x10` with its count at `+0x58`.
#[repr(C)]
pub struct NullPlayerMenuCtrl {
    vftable: usize,
    state: u16,
    /// Helper-backed fixed pointer queue used by the fallback player-menu controller.
    fallback_queue_10: MenuJobQueue8,
    /// Reference-counted fallback object released by `CSMenuManImp::Unref`.
    fallback_reference: ReferenceCountedObjectSlot,
}

/// Native `CS::PlayerStatusCalculator` owned by [`CSMenuManImp`].
///
/// Source of layout: Ghidra structures `CS::PlayerStatusCalculator` and
/// `CS::PlayerStatusCalculatorData`; the menu-manager constructor allocates
/// this object from `GLOBAL_MenuHeapAllocator`, and `CSMenuManImp::Unref` calls
/// its vtable destructor before deallocating it.
#[repr(C)]
pub struct PlayerStatusCalculator {
    vftable: usize,
    pub data: PlayerStatusCalculatorData,
    _pad_0f4: [u8; 0x4],
}

#[repr(C)]
pub struct PlayerStatusCalculatorData {
    pub current_hp: i32,
    pub current_max_hp: i32,
    pub current_fp: i32,
    pub current_max_fp: i32,
    pub current_max_stamina: i32,
    pub equipment_weight: f32,
    pub max_equip_load: f32,
    pub weight_type: i32,
    /// Native float stored between weight type and all-item weight change rate.
    equipment_load_ratio: f32,
    pub all_item_weight_change_rate: f32,
    pub toughness_damage_cut_rate: i32,
    pub item_drop_rate: i32,
    pub effective_vigor: u32,
    pub effective_mind: u32,
    pub effective_endurance: u32,
    pub effective_vitality: u32,
    pub effective_strength: u32,
    pub effective_dexterity: u32,
    pub effective_intelligence: u32,
    pub effective_faith: u32,
    pub effective_arcane: u32,
    pub attack: PlayerStatusCalculatorAttack,
    pub defense: PlayerStatusDefense,
    pub damage_negation: PlayerStatusCalculatorDamageNegation,
    pub total_resistance: StatusEffectFloats,
    pub resistance: StatusEffectFloats,
    pub magic_slots_count: u32,
    pub rune_count: i32,
}

#[repr(C)]
pub struct PlayerStatusCalculatorAttack {
    pub left_armament_primary: u32,
    pub left_armament_secondary: u32,
    pub left_armament_tertiary: u32,
    pub right_armament_primary: u32,
    pub right_armament_secondary: u32,
    pub right_armament_tertiary: u32,
}

#[repr(C)]
pub struct PlayerStatusDefense {
    pub physical: u32,
    pub strike: u32,
    pub slash: u32,
    pub pierce: u32,
    pub magic: u32,
    pub fire: u32,
    pub lightning: u32,
    pub holy: u32,
}

#[repr(C)]
pub struct PlayerStatusCalculatorDamageNegation {
    pub physical: f32,
    pub slash: f32,
    pub strike: f32,
    pub pierce: f32,
    pub magic: f32,
    pub fire: f32,
    pub lightning: f32,
    pub holy: f32,
}

#[repr(C)]
pub struct StatusEffectFloats {
    pub poison: f32,
    pub disease: f32,
    pub blood: f32,
    pub curse: f32,
    pub freeze: f32,
    pub sleep: f32,
    pub madness: f32,
}

/// Native 16-byte menu-manager stream state at `CSMenuManImp+0xe0`.
///
/// Native save/load helpers transfer this block as one 16-byte payload while
/// other methods inspect individual words and flags inside it.
#[repr(C, align(16))]
pub struct CSMenuManStreamStateE0 {
    /// Leading bytes transferred with the stream-state payload; no narrower static meaning yet.
    state_prefix_00: [u8; 0x4],
    /// Saved game-data version compared against native `GameData::GetGameDataVersion`.
    game_data_version: u32,
    /// Native flag at `+0x8` used to decide whether the state word should refresh.
    pub refresh_requested_flag: u8,
    /// Tail bytes transferred with the stream-state payload; no narrower static meaning yet.
    state_tail_09: [u8; 0x7],
}

impl CSMenuManStreamStateE0 {
    /// Native flag at `+0x8` used to decide whether the state word should refresh.
    pub fn refresh_requested(&self) -> bool {
        self.refresh_requested_flag != 0
    }
}

/// Native menu-manager four-byte lookup vector at `CSMenuManImp+0xf8`.
///
/// Native code indexes from the vector `first` qword at `+0x100` and computes
/// the active element count from `last - first` at `+0x108 - +0x100`.
pub type CSMenuManDwordLookupVectorF8 = DLVector<u32>;

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CSMenuShownMenuFlags(u32);
    impl Debug;
    /// Input-derived visibility bit from native input id `0x1d`.
    pub input_1d, _: 0;
    /// Input id `0x2c`, or the forced menu state at `CSMenuManImp+0x130`.
    pub input_2c_or_forced, _: 1;
    /// Active menu state in either `CSMenuManImp+0x104` or `+0x340`, and also
    /// forced by the popup-top-menu fallback path.
    pub player_menu_state_active, _: 2;
    /// Input-derived visibility bit from native input id `0x2e`.
    pub input_2e, _: 3;
    /// Input-derived visibility bit from native input id `0x1c`.
    pub input_1c, _: 4;
    /// `CSFeMan` overlay/status bit exposed by [`crate::cs::CSFeManImp::front_end_overlay_input_active`].
    pub fe_overlay_active, _: 5;
    /// Input-derived visibility bit from native input id `0x1f`.
    pub input_1f, _: 6;
    /// Input-derived visibility bit from native input id `0x24`.
    pub input_24, _: 7;
    /// Input-derived visibility bit from native input id `0x3d`.
    pub input_3d, _: 8;
    /// Input-derived visibility bit from native input id `0x26`.
    pub input_26, _: 9;
    /// Input-derived visibility bit from native input id `0x3c`.
    pub input_3c, _: 10;
    /// Input-derived visibility bit from native input id `0x3e`.
    pub input_3e, _: 11;
    /// Input-derived visibility bit from native input id `0x30`.
    pub input_30, _: 12;
    /// Input-derived visibility bit from native input id `0x21`.
    pub input_21, _: 13;
    /// Input-derived visibility bit from native input id `0x40`.
    pub input_40, _: 14;
    /// Input-derived visibility bit from native input id `0x25`.
    pub input_25, _: 15;
    /// Input-derived visibility bit from native input id `0x32`.
    pub input_32, _: 17;
    /// Input-derived visibility bit from native input id `0x41`.
    pub input_41, _: 18;
    /// Input-derived visibility bit from native input id `0x31`, and also forced
    /// by the popup-top-menu fallback path.
    pub input_31_or_popup_top_menu, _: 19;
    /// `CSMenuManImp::menuBrake` was non-zero during `getShownMenuFlags`.
    pub menu_brake_active, _: 20;
}

/// Native `CS::CSMenuManImp` singleton layout.
///
/// Many `state_*` and `indexed_state_words_*` fields remain raw by design: the
/// recovered indexed-state setter accepts any
/// `index < CS_MENU_MAN_INDEXED_STATE_WORD_COUNT` and writes the caller-supplied dword into this table. That
/// disproves a single closed enum for the table as a whole; individual fields
/// are promoted to booleans, bitfields, enums, or semantic names only when their
/// own producer/consumer evidence is narrower.
#[repr(C)]
#[shared::singleton("CSMenuMan")]
pub struct CSMenuManImp {
    vftable: usize,
    pub menu_data: Option<NonNull<CSMenuData>>,
    player_status_calculator: Option<NonNull<PlayerStatusCalculator>>,
    /// Title/menu-job wait flag written by the menu-manager title wait helper.
    ///
    /// The constructor writes one qword at `+0x18`, but native code later accesses
    /// this prefix as independent byte/word fields rather than one aggregate.
    title_menu_job_wait_active: bool,
    /// Second title/menu-job wait latch toggled by the title-step menu-job wait path.
    title_menu_job_wait_latch: bool,
    pub disable_mouse_cursor: bool,
    /// Previous-frame player-death/death-effect latch used to close menu UI on the
    /// false-to-true transition.
    ///
    /// `CSMenuManImp::Update` derives the current value from main-player HP-zero or
    /// special effect `0x75`; when this latch was false and the current value is
    /// true, it closes player menus/popups and suppresses UI before storing it here.
    player_death_ui_cleanup_latch: bool,
    /// Current bitmask produced each frame by native `getShownMenuFlags`.
    ///
    /// The helper ORs together input-derived bits, menu-state bits, FE overlay
    /// state, popup-top-menu fallback bits, and `menuBrake`. It is therefore a
    /// bitmask, not a closed enum discriminant.
    shown_menu_flags: CSMenuShownMenuFlags,
    /// Native prefix byte at `+0x20`; constructor-cleared, with no non-constructor
    /// read/write recovered in the current static evidence. Kept split out because
    /// `+0x21` is independently read as the loading-screen prompt job gate.
    unresolved_prefix_byte_20: u8,
    /// Enables the loading-screen prompt queue/job update path in `CSMenuManImp::Update`.
    loading_screen_prompt_job_enabled: bool,
    /// Native `mtmskbnd` file-cap unloaded through `CSFile::UnloadFileCap` on teardown.
    mtmskbnd_file_cap: Option<NonNull<FD4FileCap>>,
    /// Helper-backed fixed pointer queue used by early menu-manager jobs.
    menu_job_queue_30: MenuJobQueue8,
    pub popup_menu: Option<NonNull<CSPopupMenu>>,
    /// Native menu-window job pointer; RTTI shows `CS::MenuWindowJob` derives [`MenuJobBase`].
    pub window_job: Option<NonNull<MenuJobBase>>,
    menu_man_state_block_90: CSMenuManStateBlock90,
    /// Native 16-byte menu-manager state loaded from and saved to a stream as one payload.
    stream_state_e0: CSMenuManStreamStateE0,
    /// Native stream-parse flag updated by the `+0xf0` helper path.
    pub stream_flag_f0: bool,
    /// Native stream-parse flag updated by the `+0xf1` helper path.
    pub stream_flag_f1: bool,
    /// Native four-byte lookup vector also queried by the status-mask helper.
    pub dword_lookup_vector_f8: CSMenuManDwordLookupVectorF8,
    /// Native qword cleared by the menu-manager reset helper.
    reset_state_118_le_words: [u32; 0x2],
    /// Native menu-text state qword copied from the `+0x128` pair by menu-text handlers.
    menu_text_state_120_le_words: [u32; 0x2],
    /// Native menu-text state qword populated by the menu-text status message handler.
    menu_text_state_128_le_words: [u32; 0x2],
    /// Native menu-text qword state whose lower word is read while building the status mask.
    menu_text_state_130_le_words: [u32; 0x2],
    /// Native menu-text dirty flag set when `+0x128` is refreshed.
    pub menu_text_dirty_138: bool,
    /// Native menu-text shadow-state flag used when swapping `+0x120` and `+0x130`.
    pub menu_text_shadow_active_139: bool,
    /// disables all save menu callbacks
    /// additionally, can disable auto save
    pub disable_save_menu: u32,
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_140: u32,
    /// Native indexed dword state-table slot covered by the native indexed-state setter.
    indexed_state_word_144: u32,
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_148: u32,
    /// Native menu-text state word written by the menu-text display helper.
    menu_text_state_14c: i32,
    /// Native menu-text state word copied into the active menu-text qword when non-negative.
    menu_text_state_150: i32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_154: [u32; 0x7],
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_170: u32,
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_174: u32,
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_178: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_17c: [u32; 0x3],
    /// Native menu-manager status state value read while building the status mask.
    status_state_188: i32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_18c: [u32; 0x4],
    /// Native menu-manager state value read and written by queue/update helper paths.
    state_value_19c: i32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_1a0: [u32; 0x6],
    /// Native menu-manager state value written by menu transition helper paths.
    state_value_1b8: i32,
    /// Native indexed dword state-table slot covered by the native indexed-state setter.
    indexed_state_word_1bc: u32,
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_1c0: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_1c4: [u32; 0x4],
    /// Native menu-manager state value reset by title/menu helper paths.
    state_value_1d4: i32,
    /// Native menu-manager state value reset by title/menu helper paths.
    state_value_1d8: i32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_1dc: [u32; 0x4],
    /// Native menu-manager state value read and written by queue/update helper paths.
    state_value_1ec: i32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_1f0: [u32; 0x6],
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_208: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_20c: [u32; 0x9],
    /// Popup selection result mirrored from [`CSPopupMenu::input_buffer_184`].
    ///
    /// The popup selection helper writes the same result index to `PopupMenuInputBuffer+0x1c`
    /// and this manager slot; native cancel helpers write `0`, while queued selection
    /// helpers write the selected index plus one.
    pub popup_selection_result_index: i32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_234: [u32; 0x7],
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_250: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_254: [u32; 0x3],
    /// Native menu-manager state value reset to `-1` by menu helper paths.
    state_value_260: i32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_264: [u32; 0x13],
    /// Native menu-manager state flag written by menu helper paths.
    state_flag_2b0: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_2b4: [u32; 0xb],
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_2e0: u32,
    /// Native menu-manager state value written by menu helper paths.
    state_value_2e4: i32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_2e8: [u32; 0xf],
    /// Native menu-brake value from the 1.16.2 `CSMenuManImp` layout.
    menu_brake: i32,
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_328: u32,
    /// Native menu-manager state value read by status-message handling.
    state_value_32c: i32,
    /// Native state word preserved across the menu-manager state reset path.
    pub reset_preserved_word_330: u32,
    /// Native indexed dword state-table slot covered by the native indexed-state setter.
    indexed_state_word_334: u32,
    /// Native state word preserved across the menu-manager state reset path.
    pub reset_preserved_word_338: u32,
    /// Native indexed dword state-table slot covered by the native indexed-state setter.
    indexed_state_word_33c: u32,
    /// Native menu-manager status state value read while building the status mask.
    status_state_340: i32,
    /// Native menu-manager state value reset to `-1` by menu helper paths.
    state_value_344: i32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_348: [u32; 0x5],
    /// Native menu-manager state value written by menu helper paths.
    state_value_35c: i32,
    /// Native indexed dword state-table slot covered by the native indexed-state setter.
    indexed_state_word_360: u32,
    /// Native menu-manager state value reset to `-1` by menu helper paths.
    state_value_364: i32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_368: [u32; 0x2],
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_370: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_374: [u32; 0x7],
    /// Native menu-manager state value read while evaluating menu helper paths.
    state_value_390: i32,
    /// Native menu-manager state value cached and reused by menu helper paths.
    state_value_394: i32,
    /// Native menu-manager state value cached and compared by menu helper paths.
    state_value_398: i32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_39c: [u32; 0x7],
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_3b8: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_3bc: [u32; 0x11],
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_400: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_404: [u32; 0x11],
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_448: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_44c: [u32; 0xc],
    /// Native menu-manager state word read and reset by event/status helper paths.
    state_word_47c: u32,
    /// Native menu-manager state word read and reset by event/status helper paths.
    state_word_480: u32,
    /// Native menu-manager state word read and reset by event/status helper paths.
    state_word_484: u32,
    /// Native menu-manager state word read and reset by event/status helper paths.
    state_word_488: u32,
    /// Native indexed dword state-table slot covered by the native indexed-state setter.
    indexed_state_word_48c: u32,
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_490: u32,
    /// Native menu-manager state flag set before event/status helper dispatch.
    event_state_flag_494: u32,
    /// Native menu-manager state flag read by event/status helper dispatch.
    event_state_flag_498: u32,
    /// Native indexed dword state-table slot covered by the native indexed-state setter.
    indexed_state_word_49c: u32,
    /// Native menu-manager state flag set by the title/menu transition helper.
    transition_state_flag_4a0: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_4a4: [u32; 0xd],
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_4d8: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_4dc: [u32; 0x11],
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_520: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_524: [u32; 0x11],
    /// Native menu-manager state word with a direct runtime write xref in the 1.16.2 MCP dump.
    state_word_568: u32,
    /// Native indexed dword state-table segment covered by the native indexed-state setter.
    indexed_state_words_56c: [u32; 0x27],
    /// Native reset-helper dword range cleared from `+0x608` through `+0x653`.
    reset_state_words_608: [u32; 0x13],
    /// Final native indexed dword state-table slot covered by the native indexed-state setter.
    indexed_state_word_654: u32,
    /// Forces the same cutscene/menu-blocking condition that `CSRemoImp::IsInCutscene` reports.
    ///
    /// The cutscene-menu visibility helper returns true when either this flag is set or
    /// the remo/cutscene singleton reports an active cutscene. No writer beyond
    /// construction has been recovered yet, so the producer is still unresolved.
    cutscene_menu_block_override: bool,
    pub player_menu_ctrl: CSPlayerMenuCtrl,
    pub null_player_menu_ctrl: NullPlayerMenuCtrl,
    pub back_screen_data: BackScreenData,
    pub loading_screen_data: LoadingScreenData,
    loading_screen_prompt_queue: MenuJobQueue8,
    /// Reference-counted loading-screen/prompt object released by `CSMenuManImp::Unref`.
    loading_screen_prompt_reference: ReferenceCountedObjectSlot,
    menu_man_state_block_7a0: CSMenuManStateBlock7a0,
    /// Last low-byte result returned by the `CSMenuManStateBlock7a0` text-processing helper.
    ///
    /// The menu-manager text-processing update stores the status-text processor result byte
    /// here each update. No later consumer has been recovered, so this remains
    /// raw status storage rather than an enum.
    text_processing_result: u8,
    /// Native padding byte after [`Self::text_processing_result`].
    _pad_859: u8,
    /// Constructor-cleared tail flag next to the text-processing result; no
    /// non-constructor read/write has been recovered yet.
    unresolved_text_processing_tail_flag: bool,
    /// Native `CS::FeSystemAnnounceViewModel` pointer allocated by the constructor.
    pub system_announce_view_model: Option<NonNull<FeSystemAnnounceViewModel>>,
    pub update_task: CSEzUpdateTask<CSEzTask, Self>,
    /// Debug-menu root used when registering menu-manager debug entries.
    ///
    /// `CSMenuManImp` debug setup stores the FD4 debug-menu root node here when
    /// `GLOBAL_FD4DebugMenuManager` exists.
    debug_menu_root: Option<NonNull<FD4DebugMenuNode>>,
    pub tail_flags_898: CSMenuManTailFlags898,
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CSMenuInputFlags(u8);
    impl Debug;
    /// Menu input is currently down/active.
    pub is_down, _: 0;
    /// Menu input became active in the current input sample.
    ///
    /// Native call sites test `(flags & 3) == 3` when they require both this
    /// edge bit and [`Self::is_down`].
    pub triggered, _: 1;
}

#[repr(C)]
pub struct CSMenuManStateBlock90 {
    /// Per-menu-input state table indexed by native menu input ids.
    ///
    /// Native call sites bounds-check indices against `0x47` and read
    /// `CSMenuManImp+0x90+index`, testing bit 0 for active/down input and
    /// `(bits 0..=1) == 3` for a newly-triggered active input.
    pub input_flags: [CSMenuInputFlags; CS_MENU_INPUT_FLAG_COUNT],
    _pad_047: u8,
    /// Constructor-cleared trailing state word after the input flag table.
    state_word_48: u16,
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct CSMenuManStateBlock7a0TextFlags(u32);
    impl Debug;
    /// Copied from the source text object at `+0x68` and tested before text processing.
    pub source_text_flag0, _: 0;
    /// Copied from the source text object at `+0x69` and selects the normalization path.
    pub source_text_flag1, _: 1;
}

#[repr(C)]
pub struct CSMenuManStateBlock7a0 {
    pub active: bool,
    pub primary_text: DLString,
    pub secondary_text: DLString,
    /// Status-text state word initialized to `0x200` by the native constructor.
    ///
    /// No non-constructor read/write has been recovered yet, so this remains raw
    /// storage rather than a named enum or bitfield.
    unresolved_status_text_word_68: u32,
    /// Native padding before the `+0x70` status-text flag/result block.
    _pad_6c: [u8; 0x4],
    /// Flags copied from the active source text object before status-text processing.
    ///
    /// The status-text request helper copies bits from source object `+0x68` into this word;
    /// the status-text processing helper then tests bit 0 before processing the status text and
    /// bit 1 to select the text-normalization path.
    text_processing_flags: CSMenuManStateBlock7a0TextFlags,
    /// Text request phase checked for `1`, advanced to `2`, and reset to `0`.
    text_request_phase: u32,
    /// Result kind derived from `FUN_1424170c0` (`2 -> 1`, `3 -> 2`).
    text_result_kind: u32,
    pub status_text: DLString,
    /// Status-text normalization fallback flag set by the status-text processing helper.
    ///
    /// The native helper stores the text-normalization result here after it converts
    /// the pending status text into [`Self::status_text`], so this is a byte flag
    /// rather than one half of a constructor-only word.
    status_text_normalization_fallback: bool,
    /// Set while status-text processing is pending.
    ///
    /// The status-text request helper sets this when a text-processing request is accepted and
    /// both the request and abandon helpers clear it after the request is
    /// consumed or abandoned.
    status_text_processing_pending: bool,
    /// Native tail padding extending the inline block to `CSMenuManImp+0x858`.
    _pad_b2: [u8; 0x6],
}

#[repr(C)]
pub struct CSMenuManTailFlags898 {
    /// Constructor-initialized tail-state enable byte, defaulting to `1`.
    enabled: u8,
    /// Result of `MenuMan.EnableTopMenuDebug`.
    top_menu_debug_enabled: u8,
    /// Result of `MenuMan.EnableHelpMenuWorkaround`.
    help_menu_workaround_enabled: u8,
    /// Native tail flag read by event/status dispatch paths.
    state_flag_03: u8,
    /// Native tail flag read together with `+0x89b` by event/status dispatch paths.
    state_flag_04: u8,
    /// Native tail flag read by event/status dispatch paths.
    state_flag_05: u8,
    /// Constructor-cleared tail bytes with no narrower static evidence yet.
    reserved_tail_06: [u8; 0x2],
}

impl CSMenuManTailFlags898 {
    /// Whether `MenuMan.EnableTopMenuDebug` returned true during construction.
    pub fn top_menu_debug_enabled(&self) -> bool {
        self.top_menu_debug_enabled != 0
    }

    /// Whether `MenuMan.EnableHelpMenuWorkaround` returned true during construction.
    pub fn help_menu_workaround_enabled(&self) -> bool {
        self.help_menu_workaround_enabled != 0
    }
}

/// Fixed-capacity native menu-job queue used by menu manager/popup menu storage.
///
/// Source of layout: helper methods assert through `DLFixedVector.inl`, append
/// up to eight nullable menu-job pointers, and track the active entry count at
/// `+0x48`.
type MenuJobQueue8 = DLFixedVector<Option<NonNull<MenuJobBase>>, 8>;

impl CSMenuManImp {
    pub(crate) fn menu_input_flags(&self) -> &[CSMenuInputFlags; CS_MENU_INPUT_FLAG_COUNT] {
        &self.menu_man_state_block_90.input_flags
    }

    pub(crate) fn menu_input_flags_at(&self, index: usize) -> Option<CSMenuInputFlags> {
        self.menu_input_flags().get(index).copied()
    }

    // "You died", "Great enemy felled", etc
    pub fn display_status_message(&mut self, message: i32) -> bool {
        let rva = Program::current()
            .rva_to_va(rva::get().cs_menu_man_imp_display_status_message)
            .unwrap();

        let target = unsafe {
            std::mem::transmute::<u64, extern "C" fn(&mut CSMenuManImp, i32) -> bool>(rva)
        };
        target(self, message)
    }
}

/// Native `CS::CSMenuData` layout.
///
/// Constructor-only `state_*` slots stay raw until a producer/consumer path
/// proves a narrower invariant. Known closed or semi-closed fields are modeled
/// separately, for example [`CSMenuGaitemUseState`] and
/// [`CSMenuData::yes_no_sign_menu_result`].
#[repr(C)]
pub struct CSMenuDataDisplayGhostRequest {
    /// Non-zero word enables the display-ghost request consumer.
    ///
    /// The display-ghost request helper receives this record as a `short*` and returns false
    /// immediately when this word is zero.
    pub request_word: u16,
    /// Native reset writes `0xff` here before clearing the remaining payload.
    pub sentinel_byte_02: u8,
    _pad_03: u8,
    /// Tail payload copied as part of display ghost construction.
    ///
    /// The consumer copies the qword at `+0x0` and the dword at `+0x8`; this
    /// qword tail is stored as two words because it is only 4-byte aligned.
    payload_qword_04_le_words: [u32; 0x2],
}

#[repr(C)]
pub struct CSMenuData {
    vftable: usize,
    pub text_entry_8: CSMenuDataTextEntry,
    /// Native bytes preceding [`Self::yes_no_sign_menu_result`].
    menu_result_prefix_50: [u8; 0x3],
    /// Result flag read by `CSMenuManImp::IsYesSignMenuOptionSelected`.
    pub yes_no_sign_menu_result: bool,
    /// Raw menu-data mode byte initialized to `2`.
    ///
    /// No complete producer/consumer boundary has been recovered yet, so this is
    /// intentionally not modeled as an exhaustive enum.
    pub mode_54: u8,
    /// Raw menu-data index initialized to `-1`.
    ///
    /// No complete consumer has been recovered yet, so this remains a raw signed
    /// index rather than a closed set of states.
    state_index_58: i32,
    pub show_steam_names: bool,
    /// Tail byte paired with the native `show_steam_names` word store.
    pub name_display_flags_tail: u8,
    /// Bypasses summon-message priority filtering while transition/reload state needs priority.
    ///
    /// `MoveMapStep` recomputes this from warp, reload, dead-reset, multiplayer,
    /// name-display, and related transition state. `CS::SummonMsgQueue::AddEntry`
    /// then checks this byte and skips the normal priority comparison while it
    /// is set.
    summon_msg_priority_bypass: bool,
    /// Pending display-ghost payload consumed while the main player has special-effect state `417`.
    ///
    /// The player update path passes this record to the display-ghost request helper, which uses
    /// the current player position/name plus this payload to build `CSDisplayGhostData`,
    /// then clears the request word/sentinel/tail when the state has been consumed.
    display_ghost_request: CSMenuDataDisplayGhostRequest,
    /// Native tail word after [`Self::display_ghost_request`].
    ///
    /// The constructor initializes this to `2`; no non-constructor read/write has
    /// been recovered yet.
    unresolved_display_ghost_tail_word: u16,
    pub menu_gaitem_use_state: CSMenuGaitemUseState,
    /// Native by-value menu-data record at `+0x88`.
    ///
    /// Source of layout: `CSMenuData::CSMenuData` calls the menu-record initializer here;
    /// that helper clears one qword followed by one dword and no destructor or
    /// ownership path touches it, so it is modeled as value storage rather than
    /// as an owned pointer slot.
    menu_record_88: CSMenuDataMenuRecord,
    /// Native by-value record initialized by the aux-record initializer at `CSMenuData+0x94`.
    ///
    /// That helper clears the qword, word, and dword fields as one record. No
    /// non-constructor xref to the helper has been recovered yet, so the semantic
    /// role of the record remains unresolved rather than split into independent
    /// constructor-only words.
    aux_record_94: CSMenuDataAuxRecord94,
    pub text_entry_a8: CSMenuDataTextEntry,
}

/// Small by-value native menu-data record constructed at `CSMenuData+0x88`.
#[repr(C)]
pub struct CSMenuDataMenuRecord {
    /// Native qword stored as words because the record is only 4-byte aligned
    /// inside [`CSMenuData`].
    qword_le_words: [u32; 0x2],
    /// Native dword at record offset `+0x8`.
    dword: u32,
}

/// Native by-value record constructed at `CSMenuData+0x94`.
///
/// `CSMenuData::CSMenuData` calls the aux-record initializer here; that helper clears one
/// qword, one word, and one dword while leaving alignment padding between and
/// after those fields.
#[repr(C)]
pub struct CSMenuDataAuxRecord94 {
    qword_le_words: [u32; 0x2],
    word_08: u16,
    _pad_0a: [u8; 0x2],
    dword_0c: u32,
    _pad_10: [u8; 0x4],
}

#[repr(C)]
pub struct CSMenuGaitemUseState {
    vftable: usize,
    /// Native use-state byte: constructor clears it, populate path sets `1`,
    /// transition path promotes `1` to `2`, and reset path clears `2` back to `0`.
    state: u8,
    state_pad_09: [u8; 0x3],
    /// Quick-slot item selected for the menu gaitem-use request.
    quick_slot_item_id: OptionalItemId,
    /// Native word copied from the source item/context at `+0x48` when populated.
    pub source_word_10: u32,
    /// Native request word copied from the populate helper argument.
    pub request_word_14: u32,
}

#[repr(C)]
pub struct CSMenuDataTextEntry {
    /// Constructor-supplied entry id, initialized from `0xffff` in the default path.
    pub entry_id: u16,
    /// Static UTF-16 string pointer copied from native [`MenuString`] storage.
    static_string: *const u16,
    pub text: DLString,
    pub active: bool,
    /// Caption component slot selected by caption/menu-window update code.
    ///
    /// The caption update path compares this value against caption component indices
    /// `0..3`; the matching component receives this text entry and the other
    /// components receive an empty entry.
    caption_component_index: u16,
    /// Caption display depth/order index.
    ///
    /// The caption component setup path clamps this signed value to `0..0xff`, adds one, and
    /// passes it to the Scaleform text component setup helper. `-1` therefore
    /// maps to the fallback depth/order value.
    caption_depth_index: i16,
}

/// Native `CS::MenuViewModel` base for concrete menu view models.
///
/// Source of name: RTTI. Derived view-model constructors initialize this base
/// before installing their concrete vtables.
#[repr(C)]
pub struct MenuViewModel {
    vftable: usize,
    /// Scene object proxy retained by the menu-view-model base.
    ///
    /// `CS::MenuViewModel::MenuViewModel` clears this pointer, and scaleform
    /// object glue constructs [`SceneObjProxy`] objects from loaded GFx values.
    view: Option<NonNull<SceneObjProxy>>,
}

/// Native component proxy header used by scaleform scene object proxies.
#[repr(C)]
pub struct ComponentProxy {
    vftable: usize,
}

/// Native `CS::SceneObjProxy` wrapper for a scaleform scene object.
///
/// Source of layout: Ghidra structure `CS::SceneObjProxy`; native
/// `SceneObjProxy` constructors initialize the component-proxy header, scene
/// object handle at `+0x20`, and [`CSScaleformValue`] at `+0x28`.
#[repr(C)]
pub struct SceneObjProxy {
    component_proxy: ComponentProxy,
    _reserved_008: [u8; 0x18],
    scene_object_handle: usize,
    scaleform_value: CSScaleformValue,
}

#[repr(C)]
pub struct CSScaleformValue {
    vftable: usize,
    value: ScaleformGfxValue,
}

#[repr(C)]
pub struct ScaleformGfxValue {
    gc_prev: Option<NonNull<ScaleformGfxValue>>,
    gc_next: Option<NonNull<ScaleformGfxValue>>,
    object_interface: Option<NonNull<ScaleformGfxObjectInterface>>,
    data_type: i32,
    _pad_01c: [u8; 0x4],
    /// Variant payload used by Scaleform GFx; interpretation depends on `data_type`.
    value_payload: usize,
    /// Auxiliary variant payload used by Scaleform GFx.
    data_aux: usize,
}

#[repr(C)]
pub struct ScaleformGfxObjectInterface {
    vftable: usize,
}

/// Native `CS::WorldMapViewModel` popup-menu view model.
///
/// Static evidence: `CSPopupMenu` allocates a `0x450`-byte object, calls the
/// `CS::WorldMapViewModel` constructor, stores the result at `CSPopupMenu+0x250`,
/// and that constructor writes the `CS::WorldMapViewModel` vtable at object `+0x0`.
///
/// This is intentionally only the known prefix. The remaining native object
/// tail is not modeled until its field layout is reversed.
#[repr(C)]
pub struct WorldMapViewModel {
    vftable: usize,
}

/// Native `CS::MultiPlayViewModel` popup-menu view model.
///
/// Source of layout: Ghidra structure `CS::MultiPlayViewModel`; its
/// constructor calls [`MenuViewModel`]'s constructor, installs the concrete
/// vtable, and constructs the three [`MenuString`] fields in order.
#[repr(C)]
pub struct MultiPlayViewModel {
    pub base: MenuViewModel,
    pub password: MenuString,
    pub vow_type: MenuString,
    pub humanity_count: MenuString,
}

/// Native `CS::WorldMapTileBackReader` object allocated by [`CSPopupMenu`].
///
/// Source of name: RTTI. Constructor code installs the vtable, initializes the
/// inline fixed vector at `+0x8` with capacity 8, and destructor code walks
/// entries as 0x18-byte records with the count at `+0xc8`.
#[repr(C)]
pub struct WorldMapTileBackReader {
    vftable: usize,
    entries: DLFixedVector<WorldMapTileBackBucket, 8>,
}

/// Bucket in `CS::WorldMapTileBackReader`'s fixed vector of tile-back maps.
///
/// Source of layout: the tile-back-reader constructor builds eight 0x18-byte maps;
/// the map initializer copies the allocator, allocates a sentinel node, and keeps a
/// node count at `+0x10`. The lookup helper indexes this bucket array by
/// world-map tab and searches nodes by tile id.
#[repr(C)]
pub struct WorldMapTileBackBucket {
    allocator: &'static DLAllocator,
    sentinel: Option<NonNull<WorldMapTileBackNode>>,
    len: usize,
}

/// Red-black tree node used by [`WorldMapTileBackBucket`].
///
/// Node cleanup releases the reference-counted menu job at
/// `+0x28`; lookup compares `tile_id` at `+0x20` and clones the
/// same job pointer when a matching tile exists.
#[repr(C)]
pub struct WorldMapTileBackNode {
    left: Option<NonNull<WorldMapTileBackNode>>,
    parent: Option<NonNull<WorldMapTileBackNode>>,
    right: Option<NonNull<WorldMapTileBackNode>>,
    color: u8,
    is_nil: bool,
    _pad_01a: [u8; 0x6],
    tile_id: u16,
    _pad_022: [u8; 0x6],
    job: Option<NonNull<MenuJobBase>>,
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PopupMoveMapFlags(u16);
    impl Debug;
    /// Set by `MoveMapStep` when menu input slot `0x1c`
    /// is outside the native input-state table or not currently active.
    pub menu_input_1c_inactive, _: 8;
}

#[repr(C)]
pub struct CSPopupMenu {
    vftable: usize,
    pub menu_man: NonNull<CSMenuManImp>,
    /// Helper-backed fixed pointer queue used by popup-menu input jobs.
    popup_input_queue_10: MenuJobQueue8,
    /// Helper-backed fixed pointer queue used by popup-menu action jobs.
    popup_action_queue_60: MenuJobQueue8,
    /// Reference-counted menu job slot cleared by the constructor and released by the destructor.
    pub current_top_menu_job: Option<NonNull<MenuJobBase>>,
    /// Deferred HUD-restore job for popup cleanup.
    ///
    /// The HUD-restore helper builds a two-frame wait job whose lambda sets the HUD
    /// state back to `Default` and clears [`Self::hud_restore_pending`], then
    /// assigns that job into `CSPopupMenu+0xb8`.
    restore_hud_job: Option<NonNull<MenuJobBase>>,
    /// Active job cloned from [`Self::popup_input_queue_10`].
    ///
    /// `CSPopupMenu::Update` waits for this slot to be empty, clones from the
    /// popup input queue at `+0x10`, and assigns the clone here at
    /// `CSPopupMenu+0xc0`.
    popup_input_queue_job: Option<NonNull<MenuJobBase>>,
    /// Active tutorial popup job.
    ///
    /// The tutorial-popup helper looks up `TutorialParam`, builds a tutorial popup/menu
    /// job chain, and assigns the resulting job into this slot at
    /// `CSPopupMenu+0xc8`.
    tutorial_popup_job: Option<NonNull<MenuJobBase>>,
    popup_slot_queue_d0: MenuJobQueue8,
    /// Move-map popup/input flags updated from `MoveMapStep`.
    ///
    /// The move-map popup update sets [`PopupMoveMapFlags::menu_input_1c_inactive`] after
    /// checking menu input slot `0x1c` against `CSMenuManImp` input state.
    move_map_flags: PopupMoveMapFlags,
    /// Set from `MoveMapStep` while a move-map popup
    /// transition has work pending.
    move_map_popup_pending: bool,
    /// Native `std::function` input handler invoked by popup-menu input dispatch.
    input_handler: PopupMenuInputHandler,
    /// Native popup-menu input counter payload.
    input_data: InputData,
    inline_slot_170: PopupMenuInlineSlot,
    /// Current talk id associated with the popup menu, or `-1` when absent.
    pub current_talk_id: i32,
    pub input_buffer_184: PopupMenuInputBuffer,
    pub popup_queue: MenuStringDeque,
    /// Popup-list runtime flag initialized after deque construction.
    ///
    /// The constructor seeds this to `1`, then immediately overwrites it with
    /// `thunk_FUN_14557256a()`. No non-constructor consumer has been recovered
    /// yet, so the exact feature gate remains unresolved.
    pub popup_list_runtime_flag: bool,
    menu_string_deque_1e8: MenuStringDeque,
    menu_string_deque_218: MenuStringDeque,
    pub world_map_tile_back_reader: Option<NonNull<WorldMapTileBackReader>>,
    /// Native `CS::WorldMapViewModel` pointer allocated by the constructor.
    pub world_map_view_model: Option<NonNull<WorldMapViewModel>>,
    /// Native view-model pointer; RTTI shows `CS::GestureEquipViewModel` derives [`MenuViewModel`].
    pub gesture_equip_view_model: Option<NonNull<MenuViewModel>>,
    /// Native `CS::MultiPlayViewModel` pointer allocated by the constructor.
    pub multi_play_view_model: Option<NonNull<MultiPlayViewModel>>,
    /// Native view-model pointer; RTTI shows `CS::KeywordViewModel` derives [`MenuViewModel`].
    pub keyword_view_model: Option<NonNull<MenuViewModel>>,
    /// Native view-model pointer; RTTI shows `CS::NetworkViewModel` derives [`MenuViewModel`].
    pub network_view_model: Option<NonNull<MenuViewModel>>,
    /// Native view-model pointer; RTTI shows `CS::MainTopViewModel` derives [`MenuViewModel`].
    pub main_top_view_model: Option<NonNull<MenuViewModel>>,
    /// Native view-model pointer; RTTI shows `CS::CSTutorialViewModel` derives [`MenuViewModel`].
    pub tutorial_view_model: Option<NonNull<MenuViewModel>>,
    /// Native view-model pointer; RTTI shows `CS::MatchingViewModel` derives [`MenuViewModel`].
    pub matching_view_model: Option<NonNull<MenuViewModel>>,
    pub show_failed_to_save: bool,
    /// Deferred failed-save popup job.
    ///
    /// The failed-save popup helper is gated by [`Self::show_failed_to_save`], creates a
    /// popup/menu job and assigns it into this slot at
    /// `CSPopupMenu+0x298` once popup conditions allow it to run.
    failed_save_popup_job: Option<NonNull<MenuJobBase>>,
    /// Success-result menu job cached by popup-menu tail handling.
    ///
    /// The success-result popup helper constructs a `MenuJobResult::Success` job and assigns it
    /// into this slot at `CSPopupMenu+0x2a0`.
    success_result_job: Option<NonNull<MenuJobBase>>,
    /// Helper-backed fixed pointer queue used by popup-menu tail jobs.
    popup_tail_queue_2a8: MenuJobQueue8,
    /// Pending popup job produced by guarded popup-input scheduling.
    ///
    /// The popup-input queue helper stores a cloned job here at `CSPopupMenu+0x2f8` after
    /// the popup-input readiness helper says popup input can proceed and bumps [`Self::input_data`]'s
    /// generation counter. If this slot is already occupied, that helper clones
    /// the existing job instead of creating a replacement.
    pending_popup_input_job: Option<NonNull<MenuJobBase>>,
    pub fade_state: PopupMenuFadeState,
    /// Set while popup cleanup has a deferred HUD restore pending.
    ///
    /// The `restore_hud_job` lambda restores the HUD state
    /// to `Default` and clears this byte.
    hud_restore_pending: bool,
    _pad_309: u8,
    /// Suppresses popup-menu input dispatch while set.
    ///
    /// The popup input-dispatch helper returns immediately when this byte is nonzero instead of
    /// forwarding input through the normal popup-menu path.
    input_dispatch_suppressed: bool,
    /// Native popup tail state word, defaulting to `1`.
    ///
    /// No non-constructor producer/consumer has been recovered yet, so this
    /// remains raw state storage rather than an enum.
    popup_tail_state_word_30c: u32,
    /// Native popup tail state flag; no non-constructor read/write has been recovered yet.
    popup_tail_state_flag_310: bool,
    /// Native popup tail index, defaulting to `-1`; no complete consumer has been recovered yet.
    popup_tail_index_314: i32,
    /// World-map dialog currently attached to the popup's world-map view model.
    ///
    /// Source of layout: the world-map dialog creation path passes this pointer to the newly
    /// allocated [`WorldMapViewModel`] when opening the world map. The
    /// `WorldMapDialogBase` teardown path later calls the world-map dialog destructor with the
    /// same dialog pointer, clearing the view model's back-reference if it
    /// still matches.
    world_map_dialog: Option<NonNull<WorldMapDialogBase>>,
}

/// Native `CS::WorldMapDialogBase` object attached to [`WorldMapViewModel`].
///
/// Only the vtable header is modeled here because `CSPopupMenu` stores this as
/// an opaque dialog back-reference and does not own or inspect the object body.
#[repr(C)]
pub struct WorldMapDialogBase {
    vftable: usize,
}

/// Native `std::function` storage for popup-menu input handling.
///
/// Source of layout: Ghidra identifies `CSPopupMenu+0x128` as `std::function`
/// and destructor code invokes the stored callable destructor when the impl
/// pointer is non-null.
#[repr(C)]
pub struct PopupMenuInputHandler {
    inline_storage: [u8; 0x38],
    /// Active std::function callable implementation, or null when empty.
    ///
    /// The pending-popup-input cleanup path tests this pointer and invokes the callable destructor
    /// through its vtable before clearing it.
    impl_ptr: Option<NonNull<PopupMenuInputHandlerCallable>>,
}

#[repr(C)]
pub struct PopupMenuInputHandlerCallable {
    vftable: usize,
}

/// Native popup fade configuration initialized at `CSPopupMenu+0x300`.
///
/// The constructor writes both floats together (`0.5` and `0.22...`) as popup
/// fade state; no narrower producer/consumer has been recovered yet, so the
/// values are kept as one by-value record instead of unrelated constructor-only
/// floats.
#[repr(C)]
pub struct PopupMenuFadeState {
    pub primary_value: f32,
    pub secondary_value: f32,
}

#[repr(C)]
pub struct InputData {
    pub count: u64,
}

#[repr(C)]
pub struct PopupMenuTextState {
    /// Static UTF-16 string pointer copied together with the allocated string payload.
    static_string: *const u16,
    pub text: DLString,
}

#[repr(C)]
pub struct PopupMenuInlineSlot {
    /// Heap-owned vector of inline popup entries.
    ///
    /// Source of layout: the inline-entry append helper lazily allocates a `0x20`-byte vector
    /// object here from `GLOBAL_IngameHeapAllocator`, appends 16-byte entries,
    /// deduplicates by entry `dedupe_key`, and sort helpers compare
    /// [`PopupMenuInlineEntry::sort_key`]. The inline-entry vector cleanup helper destroys and
    /// deallocates this vector through the stored allocator.
    entries: Option<NonNull<PopupMenuInlineEntryVector>>,
    /// Inline-entry selection/index cache.
    ///
    /// The constructor initializes this to `-1`, and the inline-entry append helper resets it
    /// to `-1` before lazily allocating/appending/deduplicating inline entries.
    /// No complete consumer has been recovered yet, so this remains a raw index
    /// instead of a closed enum.
    pub index: i32,
}

#[repr(C)]
pub struct PopupMenuInlineEntryVector {
    allocator: &'static DLAllocator,
    begin: *mut PopupMenuInlineEntry,
    end: *mut PopupMenuInlineEntry,
    capacity_end: *mut PopupMenuInlineEntry,
}

#[repr(C)]
pub struct PopupMenuInlineEntry {
    /// Popup/dialog text id copied into the inline entry.
    pub message_id: u32,
    /// Key used by the inline-entry append helper to avoid adding duplicate inline entries.
    pub dedupe_key: u32,
    /// Key used by the inline-entry sort/lower-bound helpers.
    pub sort_key: i32,
    /// Native argument payload copied with the inline entry.
    pub payload: u32,
}

/// Text table selector used by popup-input message ids.
///
/// The native popup text-source helper handles values `0..=7` with a `switch`, but its
/// default arm constructs an empty [`MenuString`] instead of rejecting invalid
/// selectors. Store the raw byte in memory and use this enum for checked views.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PopupMenuTextSource {
    MsgRepository,
    ActionButton,
    EventTextForTalk,
    EventTextForMap,
    GoodsDialog,
    MenuTextEntry,
    Dialogues,
    NetworkMessage,
    Unknown(u8),
}

impl PopupMenuTextSource {
    pub fn from_raw(raw: u8) -> Self {
        match raw {
            0 => Self::MsgRepository,
            1 => Self::ActionButton,
            2 => Self::EventTextForTalk,
            3 => Self::EventTextForMap,
            4 => Self::GoodsDialog,
            5 => Self::MenuTextEntry,
            6 => Self::Dialogues,
            7 => Self::NetworkMessage,
            value => Self::Unknown(value),
        }
    }

    pub fn raw(self) -> u8 {
        match self {
            Self::MsgRepository => 0,
            Self::ActionButton => 1,
            Self::EventTextForTalk => 2,
            Self::EventTextForMap => 3,
            Self::GoodsDialog => 4,
            Self::MenuTextEntry => 5,
            Self::Dialogues => 6,
            Self::NetworkMessage => 7,
            Self::Unknown(value) => value,
        }
    }
}

#[repr(C)]
pub struct PopupMenuInputBuffer {
    /// Main popup/dialog text id copied by the popup-input constructor helper.
    pub prompt_message_id: u32,
    /// First choice text id; the popup builder reads this when the popup shape has choices.
    pub first_choice_message_id: u32,
    /// Second choice text id; the popup builder reads this for two-choice popup shapes.
    pub second_choice_message_id: u32,
    _message_pad_0c: u32,
    /// Raw text source used for [`Self::prompt_message_id`].
    ///
    /// The native popup text-source helper switches on this byte to select the FMG/text
    /// table; use [`Self::prompt_text_source`] for a checked view that preserves
    /// unknown/default-arm values.
    pub prompt_text_source: u8,
    /// Raw text source used by both choice message ids.
    ///
    /// Use [`Self::choice_text_source`] for a checked view that preserves
    /// unknown/default-arm values.
    pub choice_text_source: u8,
    _text_source_pad_12: u8,
    /// First choice is present but disabled by menu/world-state checks.
    pub first_choice_disabled: bool,
    /// Second choice is present but disabled by menu/world-state checks.
    pub second_choice_disabled: bool,
    _choice_flag_pad_15: [u8; 0x3],
    /// Constructor-cleared popup-input word; no narrower static meaning has been recovered yet.
    popup_input_word_18: u32,
    /// Result index written by popup input helpers (`-1` while unset, `0` on cancel).
    pub selection_result_index: i32,
    /// Cancel-result value cleared by one cancel path and set to `-1` by another cancel helper.
    pub cancel_result_value: i32,
    _cancel_result_pad_24: u32,
}

impl PopupMenuInputBuffer {
    pub fn prompt_text_source(&self) -> PopupMenuTextSource {
        PopupMenuTextSource::from_raw(self.prompt_text_source)
    }

    pub fn choice_text_source(&self) -> PopupMenuTextSource {
        PopupMenuTextSource::from_raw(self.choice_text_source)
    }
}

/// Native `deque<MenuString>` storage used by [`CSPopupMenu`].
///
/// Source of layout: `CSPopupMenu` constructs three deque instances at
/// `+0x1b0`, `+0x1e8`, and `+0x218`, stores `GLOBAL_MenuHeapAllocator`,
/// initializes the proxy/sentinel block, and maintains capacity/offset/length
/// qwords.
#[repr(C)]
pub struct MenuStringDeque {
    allocator: &'static DLAllocator,
    proxy: Option<NonNull<MenuStringDequeProxy>>,
    map: *mut *mut MenuString,
    map_capacity: usize,
    map_offset: usize,
    len: usize,
}

#[repr(C)]
struct MenuStringDequeProxy {
    owning_deque_proxy_slot: *mut Option<NonNull<MenuStringDequeProxy>>,
    next: Option<NonNull<MenuStringDequeProxy>>,
}

#[repr(C)]
pub struct CSPlayerMenuCtrl {
    vftable: usize,
    pub selected_goods_item: OptionalItemId,
    pub selected_magic_item: OptionalItemId,
    /// Phase for item/magic open-menu confirmation flow.
    ///
    /// Selection helpers set this to `1`; popup processing dispatches phases
    /// `1`, `2`, and `5`, and confirmation handlers promote successful goods
    /// and magic flows to phases `3`/`4` based on the active selection kind.
    item_use_menu_phase: i32,
    /// Stage returned to Lua event warp routing for multiplayer bonfire warps.
    ///
    /// The player-menu virtual stage accessor returns this field; `CSLuaEventManImp::WarpToBonfireByStage`
    /// compares it with stage `7` while deciding whether to advance to the next
    /// bonfire-warp stage or keep the kick/warp branch active.
    bonfire_warp_stage: i32,
    pub chr_menu_flags: CSChrMenuFlags,
    /// Cached open-menu job created while handling multiplayer item menu actions.
    ///
    /// Source of layout: `CSPlayerMenuCtrl` paths store the two-qword
    /// `OpenMenuJob` returned by the open-menu job builder at `+0x28`, then later pass
    /// those same qwords to `IsOpenMenuJobCurrentTop`.
    open_menu_job: OpenMenuJob,
    /// Constructor-initialized menu-control index, defaulting to `-1`.
    selected_slot_index: i32,
    /// Whether the selected menu entry belongs to a sign-puddle interaction.
    pub is_sign_puddle: bool,
    /// Native multiplayer-region flag for break-in menu context.
    pub is_break_in_multi_region: u8,
    _pad_03e: [u8; 0x2],
    /// Kick-select dialog phase used while confirming multiplayer kicks.
    ///
    /// The kick-select menu helper sets this to `2` after `ShowKickSelectMenu`, polls the
    /// dialog while it remains `2`, and uses `1` as the wait/blocked phase.
    kick_select_menu_phase: u32,
    /// Set when `ShowKickSelectMenu` is opened for the kick-select flow.
    kick_select_menu_opened: bool,
    _pad_045: [u8; 0x3],
}

#[repr(C)]
pub struct CSChrMenuFlags {
    vftable: usize,
    pub flags: ChrMenuFlags,
    // _padc: [u8; 0x4],
}

bitfield! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct ChrMenuFlags(u32);
    impl Debug;
    /// Set by TAE Event 0 (action 54 DISABLE_START_INPUTS)
    /// Controls whether the player can open the pause menu
    /// (Equipment, Crafting, Status, Messages, System, Multiplayer, Pouch, Gestures)
    pub pause_menu_state, set_pause_menu_state: 3;
}

#[repr(C)]
pub struct BackScreenData {
    vftable: usize,
    /// Constructor-initialized back-screen state byte.
    pub active: bool,
}

#[repr(C)]
pub struct LoadingScreenData {
    vftable: usize,
    /// Constructor-initialized loading-screen state byte.
    pub active: bool,
    /// Constructor-initialized transition state word.
    pub transition_state: u32,
    /// Constructor-initialized transition flags.
    pub transition_flags: u16,
    /// Current loading screen entry, or `-1` when no entry is selected.
    pub loading_screen_id: i32,
    /// Initial fade/sample value used by the loading-screen interpolation helper.
    pub fade_start: f32,
    /// Target fade/sample value used by the loading-screen interpolation helper.
    pub fade_end: f32,
    /// Interpolation duration; default constructor value is `1.0`.
    pub fade_duration: f32,
    reserved_024: u32,
}

#[repr(C)]
pub struct FeSystemAnnounceViewModel {
    pub base: MenuViewModel,
    pub message_queue: FeSystemAnnounceViewModelMessageQueue,
}

/// Native announcement entry displayed by [`FeSystemAnnounceViewModel`].
///
/// Source of layout: Ghidra structure `CS::AnnounceMessage`;
/// `CS::FeSystemAnnounceViewModel::FeSystemAnnounceViewModel` initializes a
/// queue whose map stores pointers to these messages.
#[repr(C)]
pub struct AnnounceMessage {
    pub is_active: bool,
    _pad_001: [u8; 0x7],
    /// Native qword carried with the announcement text; exact role is still private.
    message_key: u64,
    pub text: DLString,
}

/// Native queue/map backing system-announcement messages.
///
/// Source of layout: Ghidra structure
/// `CS::FeSystemAnnounceViewModelMessageQueue`; the
/// `CS::FeSystemAnnounceViewModel` constructor stores `GLOBAL_MenuHeapAllocator`,
/// allocates the proxy, and zeroes the map/capacity/offset/size fields before
/// linking the proxy back to this queue.
#[repr(C)]
pub struct FeSystemAnnounceViewModelMessageQueue {
    allocator: &'static DLAllocator,
    proxy: Option<NonNull<FeSystemAnnounceMessageQueueProxy>>,
    map: *mut *mut *mut AnnounceMessage,
    map_capacity: usize,
    map_offset: usize,
    len: usize,
}

#[repr(C)]
struct FeSystemAnnounceMessageQueueProxy {
    owning_queue_proxy_slot: *mut Option<NonNull<FeSystemAnnounceMessageQueueProxy>>,
    next: Option<NonNull<FeSystemAnnounceMessageQueueProxy>>,
}

#[cfg(test)]
mod test {
    use std::mem::{align_of, offset_of, size_of};

    use crate::cs::{
        AnnounceMessage, BackScreenData, CSMenuData, CSMenuDataAuxRecord94,
        CSMenuDataDisplayGhostRequest, CSMenuDataMenuRecord, CSMenuDataTextEntry,
        CSMenuGaitemUseState, CSMenuInputFlags, CSMenuManDwordLookupVectorF8, CSMenuManImp,
        CSMenuManStateBlock7a0, CSMenuManStateBlock7a0TextFlags, CSMenuManStateBlock90,
        CSMenuManStreamStateE0, CSMenuManTailFlags898, CSMenuShownMenuFlags, CSPlayerMenuCtrl,
        CSPopupMenu, CSScaleformValue, ComponentProxy, DLReferenceCountObjectHeader,
        FeSystemAnnounceViewModel, FeSystemAnnounceViewModelMessageQueue, InputData,
        LoadingScreenData, MenuStringDeque, MenuViewModel, MultiPlayViewModel, NullPlayerMenuCtrl,
        OpenMenuJob, PlayerStatusCalculator, PlayerStatusCalculatorAttack,
        PlayerStatusCalculatorDamageNegation, PlayerStatusCalculatorData, PlayerStatusDefense,
        PopupMenuFadeState, PopupMenuInlineSlot, PopupMenuInputBuffer, PopupMenuInputHandler,
        PopupMenuInputHandlerCallable, PopupMenuTextState, PopupMoveMapFlags, ScaleformGfxValue,
        SceneObjProxy, StatusEffectFloats, WorldMapDialogBase, WorldMapTileBackBucket,
        WorldMapTileBackNode, WorldMapTileBackReader,
    };

    use super::{CS_MENU_INPUT_FLAG_COUNT, MenuJobQueue8};

    const MENU_FIXED_POINTER_QUEUE_COUNT_OFFSET: usize = 0x48;

    const _: () = {
        assert!(offset_of!(CSPopupMenu, input_buffer_184) == 0x184);
        assert!(size_of::<PopupMenuInputBuffer>() == 0x28);
    };

    #[test]
    fn proper_sizes() {
        assert_eq!(0x8a0, size_of::<CSMenuManImp>());
        assert_eq!(0x10, size_of::<DLReferenceCountObjectHeader>());
        assert_eq!(0xf8, size_of::<PlayerStatusCalculator>());
        assert_eq!(0x08, offset_of!(PlayerStatusCalculator, data));
        assert_eq!(0xec, size_of::<PlayerStatusCalculatorData>());
        assert_eq!(0x00, offset_of!(PlayerStatusCalculatorData, current_hp));
        assert_eq!(
            0x14,
            offset_of!(PlayerStatusCalculatorData, equipment_weight)
        );
        assert_eq!(0x54, offset_of!(PlayerStatusCalculatorData, attack));
        assert_eq!(0x6c, offset_of!(PlayerStatusCalculatorData, defense));
        assert_eq!(
            0x8c,
            offset_of!(PlayerStatusCalculatorData, damage_negation)
        );
        assert_eq!(
            0xac,
            offset_of!(PlayerStatusCalculatorData, total_resistance)
        );
        assert_eq!(0xc8, offset_of!(PlayerStatusCalculatorData, resistance));
        assert_eq!(
            0xe4,
            offset_of!(PlayerStatusCalculatorData, magic_slots_count)
        );
        assert_eq!(0xe8, offset_of!(PlayerStatusCalculatorData, rune_count));
        assert_eq!(0x18, size_of::<PlayerStatusCalculatorAttack>());
        assert_eq!(0x20, size_of::<PlayerStatusDefense>());
        assert_eq!(0x20, size_of::<PlayerStatusCalculatorDamageNegation>());
        assert_eq!(0x1c, size_of::<StatusEffectFloats>());
        assert_eq!(0x4a, size_of::<CSMenuManStateBlock90>());
        assert_eq!(0xb8, size_of::<CSMenuManStateBlock7a0>());
        assert_eq!(0x08, size_of::<CSMenuManTailFlags898>());
        assert_eq!(0x50, size_of::<MenuJobQueue8>());
        assert_eq!(0x08, offset_of!(CSMenuData, text_entry_8));
        assert_eq!(0x50, offset_of!(CSMenuData, menu_result_prefix_50));
        assert_eq!(0x53, offset_of!(CSMenuData, yes_no_sign_menu_result));
        assert_eq!(0x54, offset_of!(CSMenuData, mode_54));
        assert_eq!(0x58, offset_of!(CSMenuData, state_index_58));
        assert_eq!(0x5c, offset_of!(CSMenuData, show_steam_names));
        assert_eq!(0x5d, offset_of!(CSMenuData, name_display_flags_tail));
        assert_eq!(0x5e, offset_of!(CSMenuData, summon_msg_priority_bypass));
        assert_eq!(0x60, offset_of!(CSMenuData, display_ghost_request));
        assert_eq!(
            0x00,
            offset_of!(CSMenuDataDisplayGhostRequest, request_word)
        );
        assert_eq!(
            0x02,
            offset_of!(CSMenuDataDisplayGhostRequest, sentinel_byte_02)
        );
        assert_eq!(
            0x04,
            offset_of!(CSMenuDataDisplayGhostRequest, payload_qword_04_le_words)
        );
        assert_eq!(0x0c, size_of::<CSMenuDataDisplayGhostRequest>());
        assert_eq!(
            0x6c,
            offset_of!(CSMenuData, unresolved_display_ghost_tail_word)
        );
        assert_eq!(0x70, offset_of!(CSMenuData, menu_gaitem_use_state));
        assert_eq!(0x88, offset_of!(CSMenuData, menu_record_88));
        assert_eq!(0x00, offset_of!(CSMenuDataMenuRecord, qword_le_words));
        assert_eq!(0x08, offset_of!(CSMenuDataMenuRecord, dword));
        assert_eq!(0x0c, size_of::<CSMenuDataMenuRecord>());
        assert_eq!(0x94, offset_of!(CSMenuData, aux_record_94));
        assert_eq!(0x00, offset_of!(CSMenuDataAuxRecord94, qword_le_words));
        assert_eq!(0x08, offset_of!(CSMenuDataAuxRecord94, word_08));
        assert_eq!(0x0c, offset_of!(CSMenuDataAuxRecord94, dword_0c));
        assert_eq!(0x14, size_of::<CSMenuDataAuxRecord94>());
        assert_eq!(0xa8, offset_of!(CSMenuData, text_entry_a8));
        assert_eq!(0xF0, size_of::<CSMenuData>());
        assert_eq!(0x18, size_of::<CSMenuGaitemUseState>());
        assert_eq!(0x08, offset_of!(CSMenuGaitemUseState, state));
        assert_eq!(0x09, offset_of!(CSMenuGaitemUseState, state_pad_09));
        assert_eq!(0x0c, offset_of!(CSMenuGaitemUseState, quick_slot_item_id));
        assert_eq!(0x10, offset_of!(CSMenuGaitemUseState, source_word_10));
        assert_eq!(0x14, offset_of!(CSMenuGaitemUseState, request_word_14));
        assert_eq!(0x00, offset_of!(CSMenuDataTextEntry, entry_id));
        assert_eq!(0x08, offset_of!(CSMenuDataTextEntry, static_string));
        assert_eq!(0x10, offset_of!(CSMenuDataTextEntry, text));
        assert_eq!(0x40, offset_of!(CSMenuDataTextEntry, active));
        assert_eq!(
            0x42,
            offset_of!(CSMenuDataTextEntry, caption_component_index)
        );
        assert_eq!(0x44, offset_of!(CSMenuDataTextEntry, caption_depth_index));
        assert_eq!(0x48, size_of::<CSMenuDataTextEntry>());
        assert_eq!(0x320, size_of::<CSPopupMenu>());
        assert_eq!(0x40, size_of::<PopupMenuInputHandler>());
        assert_eq!(0x00, offset_of!(PopupMenuInputHandler, inline_storage));
        assert_eq!(0x38, offset_of!(PopupMenuInputHandler, impl_ptr));
        assert_eq!(0x08, size_of::<PopupMenuInputHandlerCallable>());
        assert_eq!(0x8, size_of::<InputData>());
        assert_eq!(0x0, offset_of!(InputData, count));
        assert_eq!(0x00, offset_of!(PopupMenuTextState, static_string));
        assert_eq!(0x08, offset_of!(PopupMenuTextState, text));
        assert_eq!(0x38, size_of::<PopupMenuTextState>());
        assert_eq!(0x10, size_of::<PopupMenuInlineSlot>());
        assert_eq!(0x28, size_of::<PopupMenuInputBuffer>());
        assert_eq!(0x30, size_of::<MenuStringDeque>());
        assert_eq!(0x10, offset_of!(CSPlayerMenuCtrl, item_use_menu_phase));
        assert_eq!(0x14, offset_of!(CSPlayerMenuCtrl, bonfire_warp_stage));
        assert_eq!(0x28, offset_of!(CSPlayerMenuCtrl, open_menu_job));
        assert_eq!(0x00, offset_of!(OpenMenuJob, finalize_callback_job));
        assert_eq!(0x08, offset_of!(OpenMenuJob, input_data_count));
        assert_eq!(0x10, size_of::<OpenMenuJob>());
        assert_eq!(0x38, offset_of!(CSPlayerMenuCtrl, selected_slot_index));
        assert_eq!(0x3c, offset_of!(CSPlayerMenuCtrl, is_sign_puddle));
        assert_eq!(0x3d, offset_of!(CSPlayerMenuCtrl, is_break_in_multi_region));
        assert_eq!(0x40, offset_of!(CSPlayerMenuCtrl, kick_select_menu_phase));
        assert_eq!(0x44, offset_of!(CSPlayerMenuCtrl, kick_select_menu_opened));
        assert_eq!(0x48, size_of::<CSPlayerMenuCtrl>());
        assert_eq!(0x8, offset_of!(NullPlayerMenuCtrl, state));
        assert_eq!(0x10, offset_of!(NullPlayerMenuCtrl, fallback_queue_10));
        assert_eq!(
            0x58,
            offset_of!(NullPlayerMenuCtrl, fallback_queue_10)
                + MENU_FIXED_POINTER_QUEUE_COUNT_OFFSET
        );
        assert_eq!(0x60, offset_of!(NullPlayerMenuCtrl, fallback_reference));
        assert_eq!(0x68, size_of::<NullPlayerMenuCtrl>());
        assert_eq!(0x00, offset_of!(CSMenuManStateBlock90, input_flags));
        assert_eq!(0x1, size_of::<CSMenuInputFlags>());
        assert_eq!(
            CS_MENU_INPUT_FLAG_COUNT,
            offset_of!(CSMenuManStateBlock90, _pad_047)
        );
        assert_eq!(0x48, offset_of!(CSMenuManStateBlock90, state_word_48));
        assert_eq!(0x4a, size_of::<CSMenuManStateBlock90>());
        assert_eq!(0x00, offset_of!(CSMenuManStreamStateE0, state_prefix_00));
        assert_eq!(0x04, offset_of!(CSMenuManStreamStateE0, game_data_version));
        assert_eq!(
            0x08,
            offset_of!(CSMenuManStreamStateE0, refresh_requested_flag)
        );
        assert_eq!(0x09, offset_of!(CSMenuManStreamStateE0, state_tail_09));
        assert_eq!(0x10, size_of::<CSMenuManStreamStateE0>());
        assert_eq!(0x10, align_of::<CSMenuManStreamStateE0>());
        assert_eq!(0x20, size_of::<CSMenuManDwordLookupVectorF8>());
        assert_eq!(0x00, offset_of!(CSMenuManStateBlock7a0, active));
        assert_eq!(0x08, offset_of!(CSMenuManStateBlock7a0, primary_text));
        assert_eq!(0x38, offset_of!(CSMenuManStateBlock7a0, secondary_text));
        assert_eq!(
            0x68,
            offset_of!(CSMenuManStateBlock7a0, unresolved_status_text_word_68)
        );
        assert_eq!(
            0x70,
            offset_of!(CSMenuManStateBlock7a0, text_processing_flags)
        );
        assert_eq!(0x74, offset_of!(CSMenuManStateBlock7a0, text_request_phase));
        assert_eq!(0x78, offset_of!(CSMenuManStateBlock7a0, text_result_kind));
        assert_eq!(0x4, size_of::<CSMenuManStateBlock7a0TextFlags>());
        assert_eq!(0x80, offset_of!(CSMenuManStateBlock7a0, status_text));
        assert_eq!(
            0xb0,
            offset_of!(CSMenuManStateBlock7a0, status_text_normalization_fallback)
        );
        assert_eq!(
            0xb1,
            offset_of!(CSMenuManStateBlock7a0, status_text_processing_pending)
        );
        assert_eq!(0x00, offset_of!(CSMenuManTailFlags898, enabled));
        assert_eq!(
            0x01,
            offset_of!(CSMenuManTailFlags898, top_menu_debug_enabled)
        );
        assert_eq!(
            0x02,
            offset_of!(CSMenuManTailFlags898, help_menu_workaround_enabled)
        );
        assert_eq!(0x03, offset_of!(CSMenuManTailFlags898, state_flag_03));
        assert_eq!(0x04, offset_of!(CSMenuManTailFlags898, state_flag_04));
        assert_eq!(0x05, offset_of!(CSMenuManTailFlags898, state_flag_05));
        assert_eq!(0x06, offset_of!(CSMenuManTailFlags898, reserved_tail_06));
        assert_eq!(0x08, size_of::<CSMenuManTailFlags898>());
        assert_eq!(0x50, size_of::<MenuJobQueue8>());
        assert_eq!(0x10, size_of::<BackScreenData>());
        assert_eq!(0x28, size_of::<LoadingScreenData>());
        assert_eq!(0x10, size_of::<MenuViewModel>());
        assert_eq!(0x0, offset_of!(MenuViewModel, vftable));
        assert_eq!(0x8, offset_of!(MenuViewModel, view));
        assert_eq!(0x08, size_of::<ComponentProxy>());
        assert_eq!(0x60, size_of::<SceneObjProxy>());
        assert_eq!(0x20, offset_of!(SceneObjProxy, scene_object_handle));
        assert_eq!(0x28, offset_of!(SceneObjProxy, scaleform_value));
        assert_eq!(0x38, size_of::<CSScaleformValue>());
        assert_eq!(0x08, offset_of!(CSScaleformValue, value));
        assert_eq!(0x30, size_of::<ScaleformGfxValue>());
        assert_eq!(0x10, offset_of!(ScaleformGfxValue, object_interface));
        assert_eq!(0x18, offset_of!(ScaleformGfxValue, data_type));
        assert_eq!(0x20, offset_of!(ScaleformGfxValue, value_payload));
        assert_eq!(0x28, offset_of!(ScaleformGfxValue, data_aux));
        assert_eq!(0x00, offset_of!(MultiPlayViewModel, base));
        assert_eq!(0x10, offset_of!(MultiPlayViewModel, password));
        assert_eq!(0x48, offset_of!(MultiPlayViewModel, vow_type));
        assert_eq!(0x80, offset_of!(MultiPlayViewModel, humanity_count));
        assert_eq!(0xb8, size_of::<MultiPlayViewModel>());
        assert_eq!(0x0, offset_of!(FeSystemAnnounceViewModel, base));
        assert_eq!(0x10, offset_of!(FeSystemAnnounceViewModel, message_queue));
        assert_eq!(0x40, size_of::<FeSystemAnnounceViewModel>());
        assert_eq!(0x0, offset_of!(AnnounceMessage, is_active));
        assert_eq!(0x10, offset_of!(AnnounceMessage, text));
        assert_eq!(0x40, size_of::<AnnounceMessage>());
        assert_eq!(
            0x0,
            offset_of!(FeSystemAnnounceViewModelMessageQueue, allocator)
        );
        assert_eq!(
            0x8,
            offset_of!(FeSystemAnnounceViewModelMessageQueue, proxy)
        );
        assert_eq!(0x10, offset_of!(FeSystemAnnounceViewModelMessageQueue, map));
        assert_eq!(
            0x18,
            offset_of!(FeSystemAnnounceViewModelMessageQueue, map_capacity)
        );
        assert_eq!(
            0x20,
            offset_of!(FeSystemAnnounceViewModelMessageQueue, map_offset)
        );
        assert_eq!(0x28, offset_of!(FeSystemAnnounceViewModelMessageQueue, len));
        assert_eq!(0x30, size_of::<FeSystemAnnounceViewModelMessageQueue>());
        assert_eq!(0xd8, size_of::<WorldMapTileBackReader>());
        assert_eq!(0x08, offset_of!(WorldMapTileBackReader, entries));
        assert_eq!(0x00, offset_of!(WorldMapTileBackBucket, allocator));
        assert_eq!(0x08, offset_of!(WorldMapTileBackBucket, sentinel));
        assert_eq!(0x10, offset_of!(WorldMapTileBackBucket, len));
        assert_eq!(0x18, size_of::<WorldMapTileBackBucket>());
        assert_eq!(0x00, offset_of!(WorldMapTileBackNode, left));
        assert_eq!(0x08, offset_of!(WorldMapTileBackNode, parent));
        assert_eq!(0x10, offset_of!(WorldMapTileBackNode, right));
        assert_eq!(0x18, offset_of!(WorldMapTileBackNode, color));
        assert_eq!(0x19, offset_of!(WorldMapTileBackNode, is_nil));
        assert_eq!(0x20, offset_of!(WorldMapTileBackNode, tile_id));
        assert_eq!(0x28, offset_of!(WorldMapTileBackNode, job));
        assert_eq!(0x30, size_of::<WorldMapTileBackNode>());
    }

    #[test]
    fn indexed_state_word_offsets_match_native_setter() {
        const INDEXED_STATE_WORD_FIRST_OFFSET: usize = 0xdc;
        const INDEXED_STATE_WORD_LAST_OFFSET: usize = 0x654;
        const INDEXED_STATE_WORD_COUNT: usize = (INDEXED_STATE_WORD_LAST_OFFSET
            - INDEXED_STATE_WORD_FIRST_OFFSET)
            / std::mem::size_of::<u32>()
            + 1;

        const fn indexed_state_word_offset(index: usize) -> Option<usize> {
            if index < INDEXED_STATE_WORD_COUNT {
                Some(INDEXED_STATE_WORD_FIRST_OFFSET + index * std::mem::size_of::<u32>())
            } else {
                None
            }
        }

        assert_eq!(0x15f, INDEXED_STATE_WORD_COUNT);
        assert_eq!(Some(0xdc), indexed_state_word_offset(0));
        assert_eq!(Some(0xe0), indexed_state_word_offset(1));
        assert_eq!(Some(0x654), indexed_state_word_offset(0x15e));
        assert_eq!(None, indexed_state_word_offset(0x15f));
    }

    #[test]
    fn public_layout_offsets_match_static_re() {
        assert_eq!(0x08, offset_of!(CSMenuManImp, menu_data));
        assert_eq!(0x18, offset_of!(CSMenuManImp, title_menu_job_wait_active));
        assert_eq!(0x19, offset_of!(CSMenuManImp, title_menu_job_wait_latch));
        assert_eq!(0x1a, offset_of!(CSMenuManImp, disable_mouse_cursor));
        assert_eq!(
            0x1b,
            offset_of!(CSMenuManImp, player_death_ui_cleanup_latch)
        );
        assert_eq!(0x1c, offset_of!(CSMenuManImp, shown_menu_flags));
        assert_eq!(0x4, size_of::<CSMenuShownMenuFlags>());
        assert_eq!(0x20, offset_of!(CSMenuManImp, unresolved_prefix_byte_20));
        assert_eq!(
            0x21,
            offset_of!(CSMenuManImp, loading_screen_prompt_job_enabled)
        );
        assert_eq!(0x28, offset_of!(CSMenuManImp, mtmskbnd_file_cap));
        assert_eq!(0x30, offset_of!(CSMenuManImp, menu_job_queue_30));
        assert_eq!(
            0x78,
            offset_of!(CSMenuManImp, menu_job_queue_30) + MENU_FIXED_POINTER_QUEUE_COUNT_OFFSET
        );
        assert_eq!(0x80, offset_of!(CSMenuManImp, popup_menu));
        assert_eq!(0x88, offset_of!(CSMenuManImp, window_job));
        assert_eq!(0x90, offset_of!(CSMenuManImp, menu_man_state_block_90));
        assert_eq!(0xe0, offset_of!(CSMenuManImp, stream_state_e0));
        assert_eq!(0xf0, offset_of!(CSMenuManImp, stream_flag_f0));
        assert_eq!(0xf1, offset_of!(CSMenuManImp, stream_flag_f1));
        assert_eq!(0xf8, offset_of!(CSMenuManImp, dword_lookup_vector_f8));
        assert_eq!(0x118, offset_of!(CSMenuManImp, reset_state_118_le_words));
        assert_eq!(
            0x120,
            offset_of!(CSMenuManImp, menu_text_state_120_le_words)
        );
        assert_eq!(
            0x128,
            offset_of!(CSMenuManImp, menu_text_state_128_le_words)
        );
        assert_eq!(
            0x130,
            offset_of!(CSMenuManImp, menu_text_state_130_le_words)
        );
        assert_eq!(0x138, offset_of!(CSMenuManImp, menu_text_dirty_138));
        assert_eq!(0x139, offset_of!(CSMenuManImp, menu_text_shadow_active_139));
        assert_eq!(0x13c, offset_of!(CSMenuManImp, disable_save_menu));
        assert_eq!(0x140, offset_of!(CSMenuManImp, state_word_140));
        assert_eq!(0x144, offset_of!(CSMenuManImp, indexed_state_word_144));
        assert_eq!(0x148, offset_of!(CSMenuManImp, state_word_148));
        assert_eq!(0x14c, offset_of!(CSMenuManImp, menu_text_state_14c));
        assert_eq!(0x150, offset_of!(CSMenuManImp, menu_text_state_150));
        assert_eq!(0x154, offset_of!(CSMenuManImp, indexed_state_words_154));
        assert_eq!(0x170, offset_of!(CSMenuManImp, state_word_170));
        assert_eq!(0x174, offset_of!(CSMenuManImp, state_word_174));
        assert_eq!(0x178, offset_of!(CSMenuManImp, state_word_178));
        assert_eq!(0x17c, offset_of!(CSMenuManImp, indexed_state_words_17c));
        assert_eq!(0x188, offset_of!(CSMenuManImp, status_state_188));
        assert_eq!(0x18c, offset_of!(CSMenuManImp, indexed_state_words_18c));
        assert_eq!(0x19c, offset_of!(CSMenuManImp, state_value_19c));
        assert_eq!(0x1a0, offset_of!(CSMenuManImp, indexed_state_words_1a0));
        assert_eq!(0x1b8, offset_of!(CSMenuManImp, state_value_1b8));
        assert_eq!(0x1bc, offset_of!(CSMenuManImp, indexed_state_word_1bc));
        assert_eq!(0x1c0, offset_of!(CSMenuManImp, state_word_1c0));
        assert_eq!(0x1c4, offset_of!(CSMenuManImp, indexed_state_words_1c4));
        assert_eq!(0x1d4, offset_of!(CSMenuManImp, state_value_1d4));
        assert_eq!(0x1d8, offset_of!(CSMenuManImp, state_value_1d8));
        assert_eq!(0x1dc, offset_of!(CSMenuManImp, indexed_state_words_1dc));
        assert_eq!(0x1ec, offset_of!(CSMenuManImp, state_value_1ec));
        assert_eq!(0x1f0, offset_of!(CSMenuManImp, indexed_state_words_1f0));
        assert_eq!(0x208, offset_of!(CSMenuManImp, state_word_208));
        assert_eq!(0x20c, offset_of!(CSMenuManImp, indexed_state_words_20c));
        assert_eq!(
            0x230,
            offset_of!(CSMenuManImp, popup_selection_result_index)
        );
        assert_eq!(0x234, offset_of!(CSMenuManImp, indexed_state_words_234));
        assert_eq!(0x250, offset_of!(CSMenuManImp, state_word_250));
        assert_eq!(0x254, offset_of!(CSMenuManImp, indexed_state_words_254));
        assert_eq!(0x260, offset_of!(CSMenuManImp, state_value_260));
        assert_eq!(0x264, offset_of!(CSMenuManImp, indexed_state_words_264));
        assert_eq!(0x2b0, offset_of!(CSMenuManImp, state_flag_2b0));
        assert_eq!(0x2b4, offset_of!(CSMenuManImp, indexed_state_words_2b4));
        assert_eq!(0x2e0, offset_of!(CSMenuManImp, state_word_2e0));
        assert_eq!(0x2e4, offset_of!(CSMenuManImp, state_value_2e4));
        assert_eq!(0x2e8, offset_of!(CSMenuManImp, indexed_state_words_2e8));
        assert_eq!(0x324, offset_of!(CSMenuManImp, menu_brake));
        assert_eq!(0x328, offset_of!(CSMenuManImp, state_word_328));
        assert_eq!(0x32c, offset_of!(CSMenuManImp, state_value_32c));
        assert_eq!(0x330, offset_of!(CSMenuManImp, reset_preserved_word_330));
        assert_eq!(0x334, offset_of!(CSMenuManImp, indexed_state_word_334));
        assert_eq!(0x338, offset_of!(CSMenuManImp, reset_preserved_word_338));
        assert_eq!(0x33c, offset_of!(CSMenuManImp, indexed_state_word_33c));
        assert_eq!(0x340, offset_of!(CSMenuManImp, status_state_340));
        assert_eq!(0x344, offset_of!(CSMenuManImp, state_value_344));
        assert_eq!(0x348, offset_of!(CSMenuManImp, indexed_state_words_348));
        assert_eq!(0x35c, offset_of!(CSMenuManImp, state_value_35c));
        assert_eq!(0x360, offset_of!(CSMenuManImp, indexed_state_word_360));
        assert_eq!(0x364, offset_of!(CSMenuManImp, state_value_364));
        assert_eq!(0x368, offset_of!(CSMenuManImp, indexed_state_words_368));
        assert_eq!(0x370, offset_of!(CSMenuManImp, state_word_370));
        assert_eq!(0x374, offset_of!(CSMenuManImp, indexed_state_words_374));
        assert_eq!(0x390, offset_of!(CSMenuManImp, state_value_390));
        assert_eq!(0x394, offset_of!(CSMenuManImp, state_value_394));
        assert_eq!(0x398, offset_of!(CSMenuManImp, state_value_398));
        assert_eq!(0x39c, offset_of!(CSMenuManImp, indexed_state_words_39c));
        assert_eq!(0x3b8, offset_of!(CSMenuManImp, state_word_3b8));
        assert_eq!(0x3bc, offset_of!(CSMenuManImp, indexed_state_words_3bc));
        assert_eq!(0x400, offset_of!(CSMenuManImp, state_word_400));
        assert_eq!(0x404, offset_of!(CSMenuManImp, indexed_state_words_404));
        assert_eq!(0x448, offset_of!(CSMenuManImp, state_word_448));
        assert_eq!(0x44c, offset_of!(CSMenuManImp, indexed_state_words_44c));
        assert_eq!(0x47c, offset_of!(CSMenuManImp, state_word_47c));
        assert_eq!(0x480, offset_of!(CSMenuManImp, state_word_480));
        assert_eq!(0x484, offset_of!(CSMenuManImp, state_word_484));
        assert_eq!(0x488, offset_of!(CSMenuManImp, state_word_488));
        assert_eq!(0x48c, offset_of!(CSMenuManImp, indexed_state_word_48c));
        assert_eq!(0x490, offset_of!(CSMenuManImp, state_word_490));
        assert_eq!(0x494, offset_of!(CSMenuManImp, event_state_flag_494));
        assert_eq!(0x498, offset_of!(CSMenuManImp, event_state_flag_498));
        assert_eq!(0x49c, offset_of!(CSMenuManImp, indexed_state_word_49c));
        assert_eq!(0x4a0, offset_of!(CSMenuManImp, transition_state_flag_4a0));
        assert_eq!(0x4a4, offset_of!(CSMenuManImp, indexed_state_words_4a4));
        assert_eq!(0x4d8, offset_of!(CSMenuManImp, state_word_4d8));
        assert_eq!(0x4dc, offset_of!(CSMenuManImp, indexed_state_words_4dc));
        assert_eq!(0x520, offset_of!(CSMenuManImp, state_word_520));
        assert_eq!(0x524, offset_of!(CSMenuManImp, indexed_state_words_524));
        assert_eq!(0x568, offset_of!(CSMenuManImp, state_word_568));
        assert_eq!(0x56c, offset_of!(CSMenuManImp, indexed_state_words_56c));
        assert_eq!(0x608, offset_of!(CSMenuManImp, reset_state_words_608));
        assert_eq!(0x654, offset_of!(CSMenuManImp, indexed_state_word_654));
        assert_eq!(
            0x658,
            offset_of!(CSMenuManImp, cutscene_menu_block_override)
        );
        assert_eq!(0x660, offset_of!(CSMenuManImp, player_menu_ctrl));
        assert_eq!(0x6a8, offset_of!(CSMenuManImp, null_player_menu_ctrl));
        assert_eq!(0x748, offset_of!(CSMenuManImp, loading_screen_prompt_queue));
        assert_eq!(
            0x798,
            offset_of!(CSMenuManImp, loading_screen_prompt_reference)
        );
        assert_eq!(0x7a0, offset_of!(CSMenuManImp, menu_man_state_block_7a0));
        assert_eq!(0x858, offset_of!(CSMenuManImp, text_processing_result));
        assert_eq!(0x859, offset_of!(CSMenuManImp, _pad_859));
        assert_eq!(
            0x85a,
            offset_of!(CSMenuManImp, unresolved_text_processing_tail_flag)
        );
        assert_eq!(0x860, offset_of!(CSMenuManImp, system_announce_view_model));
        assert_eq!(0x868, offset_of!(CSMenuManImp, update_task));
        assert_eq!(0x890, offset_of!(CSMenuManImp, debug_menu_root));
        assert_eq!(0x898, offset_of!(CSMenuManImp, tail_flags_898));
        assert_eq!(0x10, offset_of!(CSPopupMenu, popup_input_queue_10));
        assert_eq!(
            0x58,
            offset_of!(CSPopupMenu, popup_input_queue_10) + MENU_FIXED_POINTER_QUEUE_COUNT_OFFSET
        );
        assert_eq!(0x60, offset_of!(CSPopupMenu, popup_action_queue_60));
        assert_eq!(
            0xa8,
            offset_of!(CSPopupMenu, popup_action_queue_60) + MENU_FIXED_POINTER_QUEUE_COUNT_OFFSET
        );
        assert_eq!(0xb0, offset_of!(CSPopupMenu, current_top_menu_job));
        assert_eq!(0xb8, offset_of!(CSPopupMenu, restore_hud_job));
        assert_eq!(0xc0, offset_of!(CSPopupMenu, popup_input_queue_job));
        assert_eq!(0xc8, offset_of!(CSPopupMenu, tutorial_popup_job));
        assert_eq!(0xd0, offset_of!(CSPopupMenu, popup_slot_queue_d0));
        assert_eq!(0x120, offset_of!(CSPopupMenu, move_map_flags));
        assert_eq!(0x2, size_of::<PopupMoveMapFlags>());
        assert_eq!(0x122, offset_of!(CSPopupMenu, move_map_popup_pending));
        assert_eq!(0x128, offset_of!(CSPopupMenu, input_handler));
        assert_eq!(0x168, offset_of!(CSPopupMenu, input_data));
        assert_eq!(0x170, offset_of!(CSPopupMenu, inline_slot_170));
        assert_eq!(0x180, offset_of!(CSPopupMenu, current_talk_id));
        assert_eq!(0x184, offset_of!(CSPopupMenu, input_buffer_184));
        assert_eq!(0x1b0, offset_of!(CSPopupMenu, popup_queue));
        assert_eq!(0x1e0, offset_of!(CSPopupMenu, popup_list_runtime_flag));
        assert_eq!(0x1e8, offset_of!(CSPopupMenu, menu_string_deque_1e8));
        assert_eq!(0x218, offset_of!(CSPopupMenu, menu_string_deque_218));
        assert_eq!(0x248, offset_of!(CSPopupMenu, world_map_tile_back_reader));
        assert_eq!(0x250, offset_of!(CSPopupMenu, world_map_view_model));
        assert_eq!(0x258, offset_of!(CSPopupMenu, gesture_equip_view_model));
        assert_eq!(0x260, offset_of!(CSPopupMenu, multi_play_view_model));
        assert_eq!(0x268, offset_of!(CSPopupMenu, keyword_view_model));
        assert_eq!(0x270, offset_of!(CSPopupMenu, network_view_model));
        assert_eq!(0x278, offset_of!(CSPopupMenu, main_top_view_model));
        assert_eq!(0x280, offset_of!(CSPopupMenu, tutorial_view_model));
        assert_eq!(0x288, offset_of!(CSPopupMenu, matching_view_model));
        assert_eq!(0x290, offset_of!(CSPopupMenu, show_failed_to_save));
        assert_eq!(0x298, offset_of!(CSPopupMenu, failed_save_popup_job));
        assert_eq!(0x2a0, offset_of!(CSPopupMenu, success_result_job));
        assert_eq!(0x2a8, offset_of!(CSPopupMenu, popup_tail_queue_2a8));
        assert_eq!(
            0x2f0,
            offset_of!(CSPopupMenu, popup_tail_queue_2a8) + MENU_FIXED_POINTER_QUEUE_COUNT_OFFSET
        );
        assert_eq!(0x2f8, offset_of!(CSPopupMenu, pending_popup_input_job));
        assert_eq!(0x300, offset_of!(CSPopupMenu, fade_state));
        assert_eq!(0x00, offset_of!(PopupMenuFadeState, primary_value));
        assert_eq!(0x04, offset_of!(PopupMenuFadeState, secondary_value));
        assert_eq!(0x308, offset_of!(CSPopupMenu, hud_restore_pending));
        assert_eq!(0x30a, offset_of!(CSPopupMenu, input_dispatch_suppressed));
        assert_eq!(0x30c, offset_of!(CSPopupMenu, popup_tail_state_word_30c));
        assert_eq!(0x310, offset_of!(CSPopupMenu, popup_tail_state_flag_310));
        assert_eq!(0x314, offset_of!(CSPopupMenu, popup_tail_index_314));
        assert_eq!(0x318, offset_of!(CSPopupMenu, world_map_dialog));
        assert_eq!(0x8, size_of::<WorldMapDialogBase>());
        assert_eq!(0x00, offset_of!(PopupMenuInlineSlot, entries));
        assert_eq!(0x08, offset_of!(PopupMenuInlineSlot, index));
        assert_eq!(0x00, offset_of!(PopupMenuInputBuffer, prompt_message_id));
        assert_eq!(
            0x04,
            offset_of!(PopupMenuInputBuffer, first_choice_message_id)
        );
        assert_eq!(
            0x08,
            offset_of!(PopupMenuInputBuffer, second_choice_message_id)
        );
        assert_eq!(0x0c, offset_of!(PopupMenuInputBuffer, _message_pad_0c));
        assert_eq!(0x10, offset_of!(PopupMenuInputBuffer, prompt_text_source));
        assert_eq!(0x11, offset_of!(PopupMenuInputBuffer, choice_text_source));
        assert_eq!(
            0x13,
            offset_of!(PopupMenuInputBuffer, first_choice_disabled)
        );
        assert_eq!(
            0x14,
            offset_of!(PopupMenuInputBuffer, second_choice_disabled)
        );
        assert_eq!(0x18, offset_of!(PopupMenuInputBuffer, popup_input_word_18));
        assert_eq!(
            0x1c,
            offset_of!(PopupMenuInputBuffer, selection_result_index)
        );
        assert_eq!(0x20, offset_of!(PopupMenuInputBuffer, cancel_result_value));
        assert_eq!(
            0x24,
            offset_of!(PopupMenuInputBuffer, _cancel_result_pad_24)
        );
        assert_eq!(0x00, offset_of!(MenuStringDeque, allocator));
        assert_eq!(0x08, offset_of!(MenuStringDeque, proxy));
        assert_eq!(0x10, offset_of!(MenuStringDeque, map));
        assert_eq!(0x18, offset_of!(MenuStringDeque, map_capacity));
        assert_eq!(0x20, offset_of!(MenuStringDeque, map_offset));
        assert_eq!(0x28, offset_of!(MenuStringDeque, len));
        assert_eq!(0x08, offset_of!(BackScreenData, active));
        assert_eq!(0x08, offset_of!(LoadingScreenData, active));
        assert_eq!(0x0c, offset_of!(LoadingScreenData, transition_state));
        assert_eq!(0x10, offset_of!(LoadingScreenData, transition_flags));
        assert_eq!(0x14, offset_of!(LoadingScreenData, loading_screen_id));
        assert_eq!(0x18, offset_of!(LoadingScreenData, fade_start));
        assert_eq!(0x1c, offset_of!(LoadingScreenData, fade_end));
        assert_eq!(0x20, offset_of!(LoadingScreenData, fade_duration));
        assert_eq!(0x24, offset_of!(LoadingScreenData, reserved_024));
    }
}
