use crate::{
    DLVector,
    cs::{BlockId, CSGaitemGameData, ChrAsm, ChrType, FaceData, FaceDataBuffer, PlayerGameData},
    dlut::DLFixedVector,
    fd4::{FD4DebugMenuNode, FD4Time},
};
use shared::{F32Vector3, FromStatic, OwnedPtr, load_static_indirect};
use std::{borrow::Cow, ptr::NonNull};

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RemotePlayerDataSlotState {
    /// Player data slot is free / unoccupied
    Free = 0,
    /// Player data slot is occupied but not yet synced
    Occupied = 1 << 0,
    /// Player data slot has base character data (packet 8)
    BaseData = 1 << 1,
    /// Player data slot has equipment data (packet 12)
    Equipment = 1 << 2,
    /// Player data slot has character type data (packet 11)
    Type = 1 << 3,
    /// Player data slot is fully synced
    FullySynced = 0xF,
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
/// Source of name: global_event.lua from DS3
pub enum DeathState {
    None = -1,
    Normal = 0,
    /// DS3 tears of denial style resurrection
    MagicResurrection = 1,
    /// Sacrificial Twig
    RingNormalResurrection = 2,
    /// Twiggy Cracked Tear
    RingCurseResurrection = 3,
}

const fn native_gap(end_offset: usize, start_offset: usize) -> usize {
    assert!(end_offset >= start_offset);
    end_offset - start_offset
}

#[repr(C)]
pub struct GameDataMan {
    pub trophy_equip_data: OwnedPtr<TrophyEquipData>,
    pub main_player_game_data: OwnedPtr<PlayerGameData>,
    pub player_game_data_list: OwnedPtr<[PlayerGameData; 5]>,
    /// Pointer to the game data of the player used for the baseline
    /// for the arena match multiplay scaling in active match.
    pub quickmatch_scaling_baseline_game_data: Option<NonNull<PlayerGameData>>,
    pub remote_game_data_states: OwnedPtr<[RemotePlayerDataSlotState; 5]>,
    pub session_player_game_data_list: OwnedPtr<[Option<OwnedPtr<PlayerGameData>>; 40]>,
    pub gaitem_game_data: OwnedPtr<CSGaitemGameData>,
    /// Native tutorial-data pointer; RTTI shows `CS::CSTutorialData` derives [`CSMenuSaveLoad`].
    tutorial_data: Option<NonNull<CSMenuSaveLoad>>,
    /// Whether a recoverable bloodstain record is currently available.
    ///
    /// Maintained by the `CSEventBloodStainCtrl` clear/update paths rooted from
    /// `CSEventManImp::bloodstain` and consumed by bloodstain replay/upload flow.
    pub has_bloodstain: bool,
    /// Current recoverable bloodstain data, allocated on demand and freed by
    /// [`GameDataMan`]'s destructor when present.
    pub bloodstain: Option<NonNull<GameDataManBloodstainData>>,
    /// Entity handle associated with the current recoverable bloodstain, or `-1`.
    ///
    /// This is intentionally raw rather than an exhaustive enum: the constructor
    /// and bloodstain reset helper write the `-1` sentinel, but the
    /// `CSEventBloodStainCtrl` death-update path stores
    /// `FieldInsBase::handle.entityHandle` here.
    /// That handle is allocated from runtime world state, not a closed static set.
    pub bloodstain_entity_id: i32,
    pub game_settings: OwnedPtr<GameSettings>,
    pub menu_system_save_load: Option<NonNull<CSMenuSystemSaveLoad>>,
    /// Native menu-profile save/load pointer; RTTI shows `CS::CSMenuProfileSaveLoad` derives [`CSMenuSaveLoad`].
    pub menu_profile_save_load: Option<NonNull<CSMenuSaveLoad>>,
    /// Native key-config save/load pointer; RTTI shows `CS::CSKeyConfigSaveLoad` derives [`CSMenuSaveLoad`].
    pub key_config_save_load: Option<NonNull<CSMenuSaveLoad>>,
    pub profile_summary: Option<NonNull<ProfileSummary>>,
    /// Native PC-option data pointer; RTTI shows `CS::PcOptionData` derives [`IGameDataElem`].
    pc_option_data: Option<NonNull<IGameDataElem>>,
    /// Raw save-serialized solo-break-in/great-rune state word.
    ///
    /// The player-data initialization/reset path clears this word beside the
    /// Lua-driven recovery request flags, and main-save serialization writes it immediately
    /// after NG level. No validating boundary or complete switch over this word
    /// has been recovered, so treating the raw `u32` as an exhaustive enum would
    /// currently overstate the native invariant.
    solo_break_in_state: u32,
    pub request_full_recovery: bool,
    /// Native request flag set by `CSLuaEventProxy::SetRequestRestoreHp`.
    pub request_restore_hp: bool,
    /// Save-serialized game-data byte between restore-HP and phantom-rune flags.
    recovery_state_byte: u8,
    /// Whether game should give the player the phantom great rune
    /// Will be true for some time during loading when Mogh's great rune is active and
    /// the player is invading someone else's world
    pub award_phantom_great_rune_requested: bool,
    /// Whether game should give the player the rebreak in item
    /// Will be true for some time during loading when the player is invading someone else's world
    pub award_rebreak_in_item_requested: bool,
    pub death_count: u32,
    /// Character type to switch to after loading a map
    /// [ChrType::None] if no switch is requested
    ///
    /// Set by `CS::CSLuaEventProxy::SetChrTypeDataGreyNext`
    pub post_map_load_chr_type: ChrType,
    /// Save-serialized game-data state bytes following `death_count`.
    save_state_after_death_count: [u8; 0x2],
    /// Play time as milliseconds
    /// will be maxed out at 999:59:59.999
    pub play_time: u32,
    /// Native game-data state bytes between the play-time word and boss-fight timer.
    boss_fight_state_a4: [u8; 0xc],
    /// Timer for tracking boss fight duration
    pub boss_fight_timer: FD4Time,
    /// Whether a boss fight is currently active
    pub boss_fight_active: bool,
    /// Count of white phantoms currently summoned
    /// Used to apply enemy level scaling
    pub white_phantom_count: u32,
    pub boss_health_bar_entity_id: u32,
    pub boss_health_bar_npc_param_id: u32,
    /// Whether the game is currently tracking an active boss fight.
    ///
    /// `CS::GameDataMan::IsInBossFight` returns this flag, and
    /// `ResetBossFightTrackingData` clears it after clearing the boss health-bar
    /// entity/NPC ids.
    pub is_in_boss_fight: bool,
    _pad_d1: [u8; 0x3],
    /// State of special death-related effects
    pub death_state: DeathState,
    /// Whether the player has a death preventing effect active
    pub has_death_preventing_effect: bool,
    /// Whether the player died recently
    pub just_died: bool,
    /// Leave request status for each player slot
    /// Used by lua script imitation to track on leave events
    pub leave_requests: [bool; 5],
    pub game_version_data: GameVersionData,
    unkf0: bool,
    /// Whether the DLC list is up to date and any pending DLCs have been applied ([`pending_dlc_list`] is empty)
    ///
    /// [`pending_dlc_list`]: Self::pending_dlc_list
    pub dlc_list_up_to_date: bool,
    /// Vector of indecies into `CSDlc` that are not applied to this game data yet
    pub pending_dlc_list: DLVector<u32>,
    pub is_net_penalized: bool,
    pub net_penalty_requested: bool,
    pub net_penalty_points: u16,
    pub net_penalty_forgive_item_limit_time: f32,
    pub ng_lvl: u32,
    /// Random-appearance lot slot for the local world.
    ///
    /// The normal reroll path writes `0..=99`, but this is intentionally raw:
    /// a non-negative `default_random_appear_lot_slot` override is copied without
    /// an upper-bound check, and `ApplyHostWorldValues` copies the host packet
    /// value into this field without validating the slot range.
    pub random_appear_lot_slot: i32,
    /// NG level supplied by the multiplayer host while host world values are pending.
    ///
    /// The local sender clamps its own NG level to `0..=7`, but the receive path
    /// passes the packet field into the host-world setter without revalidating it.
    host_ng_level: u32,
    /// Random-appearance lot slot supplied by the multiplayer host.
    ///
    /// The host normally sends [`Self::random_appear_lot_slot`], but
    /// `ReceivePacket30` only validates packet size/session/team fields before
    /// copying this value through the host-world setter. `ApplyHostWorldValues`
    /// then copies it into the active slot, so the native invariant is not a
    /// closed `0..=99` enum on the receiver.
    host_random_appear_lot_slot: i32,
    /// Local NG level saved while host world values are applied.
    backup_ng_level: u32,
    /// Local random-appearance lot slot saved while host world values are applied.
    ///
    /// This preserves the raw active value, including native override or host
    /// values that are outside the normal reroll range.
    backup_random_appear_lot_slot: i32,
    /// Whether new host NG/random-appearance values should be applied on map move.
    host_world_values_update_requested: bool,
    /// Whether `ng_lvl` and `random_appear_lot_slot` currently contain host values.
    host_world_values_applied: bool,
    _pad_13a: [u8; 0x6],
    /// Debug-menu root used when registering player-game-data debug entries.
    ///
    /// `GameDataMan` debug setup stores the FD4 debug-menu root node here when
    /// `GLOBAL_FD4DebugMenuManager` exists.
    player_game_data_debug_menu_root: Option<NonNull<FD4DebugMenuNode>>,
    /// Param-reload request flag initialized by the native game-data constructor.
    ///
    /// The constructor seeds this from a native config/helper call after
    /// `default_random_appear_lot_slot`; no complete writer/consumer pair has
    /// been recovered yet, so this remains raw flag storage instead of a richer
    /// request-state enum.
    param_reload_requested: bool,
    _pad_149: [u8; 0x3],
    /// Last character-init param applied to the main player, defaulting to `-1`.
    last_applied_chara_init_param_id: i32,
    /// Default random-appearance lot slot override, or `-1` to reroll randomly.
    ///
    /// `RerollRandomAppearLotSlot` uses `0..=99` only when this value is negative;
    /// otherwise it copies this override directly into the active slot.
    default_random_appear_lot_slot: i32,
}

/// Native `CS::GameDataMan` bloodstain recovery payload.
///
/// Source of layout: `CS::GameData::GetBloodstainData` returns
/// `GLOBAL_GameDataMan+0x48`; the `CSEventBloodStainCtrl` death-update path
/// writes map-local position, rotation, block id, death-blight flag, rune count,
/// and base hero-point values through that pointer before bloodstain replay/upload refresh.
#[repr(C)]
pub struct GameDataManBloodstainData {
    /// Map-local bloodstain position used for replay placement.
    pub msb_position: F32Vector3,
    /// Bloodstain replay rotation.
    pub rotation: F32Vector3,
    /// Native payload between transform data and reward/block metadata.
    replay_payload_18: [u8; 0x18],
    /// Base hero-point value saved before bloodstain recovery clears it.
    pub base_hero_point_2: i32,
    /// Rune count available for bloodstain recovery.
    pub rune_count: i32,
    /// Map block associated with the bloodstain position.
    pub block_id: BlockId,
    /// Whether the death that produced this bloodstain had death blight.
    pub has_death_blight_effect: bool,
    _pad_3d: [u8; 0x3],
}

/// Packed native sort state stored by [`CSMenuSystemSaveLoad`].
///
/// Source of layout: the Elden Ring `CSMenuSystemSaveLoad` constructor calls a
/// helper that initializes twenty-four packed `u32` sort-state entries. Native
/// code preserves the high bit while replacing selected low-bit payloads; the
/// exact menu-facing meaning of those bits is not fully reversed yet.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MenuSortState(u32);

impl MenuSortState {
    const HIGH_BIT_FLAG: u32 = 1 << (u32::BITS - 1);
    const VALUE_BITS_MASK: u32 = !Self::HIGH_BIT_FLAG;

    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Whether the native packed word has its high bit set.
    pub const fn high_bit_flag(self) -> bool {
        self.0 & Self::HIGH_BIT_FLAG != 0
    }

    /// Low 31 bits of the native packed word.
    pub const fn value_bits(self) -> u32 {
        self.0 & Self::VALUE_BITS_MASK
    }
}

/// Native count of remembered menu sort-state entries.
///
/// Source of value: the menu-sort initializer called by
/// `CS::CSMenuSystemSaveLoad::CSMenuSystemSaveLoad` initializes `0x18` native
/// `u32` sort-state entries.
const MENU_SORT_STATE_COUNT: usize = 0x18;

/// Native table of per-menu remembered sort states.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MenuSortStates {
    entries: [MenuSortState; MENU_SORT_STATE_COUNT],
}

impl MenuSortStates {
    pub fn as_slice(&self) -> &[MenuSortState] {
        &self.entries
    }

    pub fn as_mut_slice(&mut self) -> &mut [MenuSortState] {
        &mut self.entries
    }
}

impl AsRef<[MenuSortState]> for MenuSortStates {
    fn as_ref(&self) -> &[MenuSortState] {
        self.as_slice()
    }
}

impl AsMut<[MenuSortState]> for MenuSortStates {
    fn as_mut(&mut self) -> &mut [MenuSortState] {
        self.as_mut_slice()
    }
}

/// Native `CS::IGameDataElem` base.
///
/// Source of name: RTTI.
#[repr(C)]
pub struct IGameDataElem {
    vftable: usize,
}

/// Native `CS::CSMenuSaveLoad` base.
///
/// Source of name and constructor stores: RTTI and `CSMenuSaveLoadDef.cpp`
/// constructor code in the Elden Ring executable.
#[repr(C)]
pub struct CSMenuSaveLoad {
    pub game_data_elem: IGameDataElem,
    /// Constructor-supplied save/load definition id.
    pub definition_id: u16,
    /// Constructor-supplied save/load definition variant.
    pub definition_variant: u16,
    /// Constructor-supplied serialized data capacity in bytes.
    pub serialized_capacity_bytes: u32,
}

/// Native `CS::CSMenuFaceData` stored by `CSMenuSystemSaveLoad::FaceChunk`.
///
/// Source of name: RTTI. Constructor/copy code stores the vtable, copies two
/// native header bytes, copies one inline [`FaceDataBuffer`] at `+0xc`, and
/// copies a native footer word at `+0x12c`.
#[repr(C)]
pub struct CSMenuFaceData {
    vftable: usize,
    /// Native two-byte menu face-data header. Individual bits are not fully
    /// reversed yet.
    pub header: u16,
    pub face_data_buffer: FaceDataBuffer,
    /// Native two-byte menu face-data footer. Individual bits are not fully
    /// reversed yet.
    pub footer: u16,
}

/// Native fixed-vector capacity of `CSMenuSystemSaveLoad::FaceChunk`.
///
/// Source of value: the face-chunk initializer called by
/// `CS::CSMenuSystemSaveLoad::CSMenuSystemSaveLoad` loops `0xf` times,
/// appends one [`CSMenuFaceData`] per iteration with stride `0x130`, and panics
/// if the checked Dantelion fixed-vector length would exceed `0xf`.
const CS_MENU_SYSTEM_SAVE_LOAD_FACE_COUNT: usize = 0xf;

/// Native `CS::CSMenuSystemSaveLoad::FaceChunk` storage.
///
/// Source of name: RTTI. The constructor appends up to
/// `CS_MENU_SYSTEM_SAVE_LOAD_FACE_COUNT` [`CSMenuFaceData`] entries with a
/// native stride of `0x130`; helper code uses the existing Dantelion fixed-vector
/// layout with checked length at `+0x11d8` relative to the entries base
/// (`FaceChunk+0x11e0`).
#[repr(C)]
pub struct CSMenuSystemSaveLoadFaceChunk {
    vftable: usize,
    pub entries: DLFixedVector<CSMenuFaceData, CS_MENU_SYSTEM_SAVE_LOAD_FACE_COUNT>,
}

/// One native detail-status view-state entry in
/// `DETAIL_STATUS_VIEW_STATE_SAVEDATA`.
///
/// The native constructor writes the integer fields at unaligned offsets, so the
/// little-endian bytes are stored directly and exposed through accessors.
#[repr(C)]
pub struct DetailStatusViewStateEntry {
    selected_index_le_bytes: [u8; 0x2],
    scroll_position_le_bytes: [u8; 0x4],
    packed_view_state_le_bytes: [u8; 0x4],
    pub active: bool,
    reserved_tail: [u8; 0x5],
}

impl DetailStatusViewStateEntry {
    pub fn selected_index(&self) -> i16 {
        i16::from_le_bytes(self.selected_index_le_bytes)
    }

    pub fn scroll_position(&self) -> u32 {
        u32::from_le_bytes(self.scroll_position_le_bytes)
    }

    pub fn packed_view_state(&self) -> u32 {
        u32::from_le_bytes(self.packed_view_state_le_bytes)
    }
}

/// Native entry count for `DETAIL_STATUS_VIEW_STATE_SAVEDATA`.
///
/// Source of value: the detail-status initializer called by
/// `CS::CSMenuSystemSaveLoad::CSMenuSystemSaveLoad` initializes `0x10` records
/// and then iterates from the entries base to `base + 0x100` in `0x10`-byte
/// strides.
const DETAIL_STATUS_VIEW_STATE_ENTRY_COUNT: usize = 0x10;

/// Native `CS::CSMenuSimpleSaveDataChunk<DETAIL_STATUS_VIEW_STATE_SAVEDATA, 3>`.
#[repr(C)]
pub struct DetailStatusViewStateSaveData {
    pub entries: [DetailStatusViewStateEntry; DETAIL_STATUS_VIEW_STATE_ENTRY_COUNT],
}

/// Raw byte payload in native `MENU_INPUT_HISTORY`.
///
/// Source of layout: RTTI names the enclosing `MENU_INPUT_HISTORY` payload and
/// the native constructor zeroes `0x10e` bytes covering the two leading words,
/// byte-count word, and this raw payload. Individual payload entries are not
/// fully reversed yet.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MenuInputHistoryPayloadStorage {
    bytes: [u8; 0x106],
}

impl MenuInputHistoryPayloadStorage {
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn as_mut_bytes(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl AsRef<[u8]> for MenuInputHistoryPayloadStorage {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsMut<[u8]> for MenuInputHistoryPayloadStorage {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_bytes()
    }
}

/// Native `CS::CSMenuSimpleSaveDataChunk<MENU_INPUT_HISTORY, 4>` storage.
///
/// Source of name: RTTI. The native constructor zeroes `0x10e` bytes here;
/// save/load methods read the two leading words and byte-count word separately
/// before treating the remaining storage as a raw byte payload.
#[repr(C)]
pub struct MenuInputHistorySaveData {
    pub header_word_0: u16,
    pub header_word_2: u16,
    pub payload_byte_count: u32,
    pub payload_storage: MenuInputHistoryPayloadStorage,
}

/// Native `MENU_TITLEFLOW_SAVEDATA` payload.
///
/// Source of name: RTTI. The constructor zeroes four qwords at `+0x1200`;
/// the first qword is split into the known save slot and flag word used by the
/// title-flow save/load path.
#[repr(C)]
pub struct MenuTitleFlowSaveData {
    /// Save slot used by the title-flow save/load path.
    pub save_slot: i32,
    /// Native title-flow save/load flag word.
    ///
    /// The reset helper clears bits `0..=6` individually, but no complete
    /// producer/consumer mapping for those bits has been recovered yet. Keep this
    /// raw until each bit can be named from evidence.
    pub flags: u32,
    /// Native title-flow tail qwords after [`Self::flags`].
    ///
    /// The reset helper clears the first qword at record offset `+0x8`; no
    /// complete producer/consumer mapping for all three qwords has been recovered
    /// yet, so this is kept as tail storage rather than independent state names.
    tail_qwords_08: [u64; 3],
}

/// Native `CS::CSMenuSimpleSaveDataChunk<T>` wrapper.
///
/// Source of name: RTTI specializations in the Elden Ring executable.
#[repr(C)]
pub struct CSMenuSimpleSaveDataChunk<T> {
    vftable: usize,
    pub data: T,
}

/// Menu-system save/load object rooted from `GameDataMan`.
#[repr(C)]
pub struct CSMenuSystemSaveLoad {
    pub base: CSMenuSaveLoad,
    pub face_chunk: CSMenuSystemSaveLoadFaceChunk,
    pub title_flow_save_data: CSMenuSimpleSaveDataChunk<MenuTitleFlowSaveData>,
    pub detail_status_view_state: CSMenuSimpleSaveDataChunk<DetailStatusViewStateSaveData>,
    pub menu_input_history: CSMenuSimpleSaveDataChunk<MenuInputHistorySaveData>,
    /// Per-menu remembered native sort state.
    pub menu_sort_states: MenuSortStates,
}

/// UTF-16 code-unit count of `ProfileSummaryRecord::name`.
///
/// Source of value: Ghidra/PDB labels the game field as native `wchar_t[16]`,
/// and `ProfileSummaryRecord` construction clears the first code unit before
/// writing the next dword field at record offset `+0x24`.
const PROFILE_SUMMARY_CHARACTER_NAME_CODE_UNIT_COUNT: usize = 16;

/// Fixed inline native storage used by profile-summary character names.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProfileSummaryCharacterName {
    code_units: [u16; PROFILE_SUMMARY_CHARACTER_NAME_CODE_UNIT_COUNT],
    native_gap_20: [u8; native_gap(0x24, 0x20)],
}

impl ProfileSummaryCharacterName {
    pub fn code_units(&self) -> &[u16] {
        &self.code_units
    }

    pub fn code_units_mut(&mut self) -> &mut [u16] {
        &mut self.code_units
    }

    /// UTF-16 code units up to, but not including, the first NUL terminator.
    pub fn code_units_until_nul(&self) -> &[u16] {
        let length = self
            .code_units
            .iter()
            .position(|&code_unit| code_unit == 0)
            .unwrap_or(self.code_units.len());
        &self.code_units[..length]
    }

    /// Decode the NUL-terminated UTF-16 profile name without lossy replacement.
    pub fn try_to_string(&self) -> Result<String, std::string::FromUtf16Error> {
        String::from_utf16(self.code_units_until_nul())
    }

    /// Decode the NUL-terminated UTF-16 profile name, replacing invalid data.
    pub fn to_string_lossy(&self) -> String {
        String::from_utf16_lossy(self.code_units_until_nul())
    }
}

impl AsRef<[u16]> for ProfileSummaryCharacterName {
    fn as_ref(&self) -> &[u16] {
        self.code_units()
    }
}

impl AsMut<[u16]> for ProfileSummaryCharacterName {
    fn as_mut(&mut self) -> &mut [u16] {
        self.code_units_mut()
    }
}

/// Per-slot profile summary record.
#[repr(C)]
pub struct ProfileSummaryRecord {
    /// Fixed inline UTF-16 character-name storage identified as native `wchar_t[16]`.
    pub name: ProfileSummaryCharacterName,
    pub level: u32,
    pub play_time: u32,
    pub rune_memory: u32,
    pub map: u32,
    pub face_data: FaceData,
    pub chr_asm: ChrAsm,
    pub gender: u8,
    pub archetype: u8,
    pub starting_gift: u8,
    pub player_game_data_unkc4: u8,
    profile_renderer_byte_294: u8,
    loadable_flag_295: u8,
    /// Native record-stride tail bytes copied with each profile-summary entry.
    record_tail_296: [u8; 0xa],
}

/// Native save-slot count for [`ProfileSummary`].
///
/// Source of value: `CS::ProfileSummary::ProfileSummary` calls
/// `_eh_vector_constructor_iterator_(records, PROFILE_SUMMARY_RECORD_STRIDE_BYTES,
/// 10, ...)` and then clears `saveSlotsStates[0]` through `[9]`.
const PROFILE_SUMMARY_SLOT_COUNT: usize = 10;

/// Native byte stride for one [`ProfileSummaryRecord`] in the game constructor.
///
/// Source of value: `CS::ProfileSummary::ProfileSummary` passes `0x2a0` as the
/// vector-constructor element stride for the profile-summary record array.
const PROFILE_SUMMARY_RECORD_STRIDE_BYTES: usize = 0x2a0;

/// Byte extent of the native profile-summary record array.
const PROFILE_SUMMARY_RECORDS_BYTE_EXTENT: usize =
    PROFILE_SUMMARY_SLOT_COUNT * PROFILE_SUMMARY_RECORD_STRIDE_BYTES;
const _: usize = PROFILE_SUMMARY_RECORDS_BYTE_EXTENT;

/// Profile-summary table rooted from `GameDataMan`.
#[repr(C)]
pub struct ProfileSummary {
    vftable: usize,
    /// Native `saveSlotsStates[slot]` bytes at `ProfileSummary+0x8`.
    ///
    /// Kept as raw bytes because the native layout does not define a distinct
    /// public slot-state type here. The slot count is pulled from the game
    /// constructor's vector-constructor count and explicit `saveSlotsStates`
    /// clears, not from a guessed byte extent.
    active_slots: [u8; PROFILE_SUMMARY_SLOT_COUNT],
    pub records: [ProfileSummaryRecord; PROFILE_SUMMARY_SLOT_COUNT],
}

impl FromStatic for GameDataMan {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("GameDataMan")
    }

    fn instance_ptr() -> shared::InstanceResult<*mut Self> {
        unsafe { load_static_indirect(crate::rva::get().game_data_man) }
    }
}

#[repr(C)]
pub struct TrophyEquipData {
    vftable: usize,
    unk8: u32,
    /// Stats towards [`Legendary Armaments`] achievement
    ///
    /// [`Legendary Armaments`]: crate::cs::trophy::AchievementId::LegendaryArmaments
    pub weapon_stats: TrophyWeaponStats<[u8; 0x10]>,
    /// Stats towards [`Legendary Sorceries and Incantations`] and [`Legendary Ashen Remains`] achievements
    ///
    /// [`Legendary Sorceries and Incantations`]: crate::cs::trophy::AchievementId::LegendarySorceriesAndIncantations
    /// [`Legendary Ashen Remains`]: crate::cs::trophy::AchievementId::LegendaryAshenRemains
    pub goods_stats: TrophyGoodsStats<[u8; 0x10]>,
    /// Stats towards [`Legendary Talismans`] achievement
    ///
    /// [`Legendary Talismans`]: crate::cs::trophy::AchievementId::LegendaryTalismans
    pub accessory_stats: TrophyAccessoryStats<[u8; 0x10]>,
}

fn trophy_stat_bit(bytes: &[u8], bit: usize) -> bool {
    bytes[bit / 8] & (1u8 << (bit % 8)) != 0
}

fn trophy_stat_set_bit(bytes: &mut [u8], bit: usize, value: bool) {
    let mask = 1u8 << (bit % 8);
    let byte = &mut bytes[bit / 8];
    if value {
        *byte |= mask;
    } else {
        *byte &= !mask;
    }
}

fn trophy_stat_range(bytes: &[u8], high: usize, low: usize) -> u128 {
    let mut value = 0u128;
    for bit in low..=high {
        if trophy_stat_bit(bytes, bit) {
            value |= 1u128 << (bit - low);
        }
    }
    value
}

fn trophy_stat_set_range(bytes: &mut [u8], high: usize, low: usize, value: u128) {
    for bit in low..=high {
        trophy_stat_set_bit(bytes, bit, ((value >> (bit - low)) & 1) != 0);
    }
}

macro_rules! trophy_stats {
    (
        $name:ident, $unused_low:literal,
        $(
            $(#[$field_attr:meta])*
            $get:ident, $set:ident: $bit:literal;
        )*
    ) => {
        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct $name<T: ?Sized>(T);

        impl<T: AsRef<[u8]> + ?Sized> $name<T> {
            $(
                $(#[$field_attr])*
                pub fn $get(&self) -> bool {
                    trophy_stat_bit(self.0.as_ref(), $bit)
                }
            )*

            pub fn unused(&self) -> u128 {
                trophy_stat_range(self.0.as_ref(), 127, $unused_low)
            }
        }

        impl<T: AsMut<[u8]> + AsRef<[u8]> + ?Sized> $name<T> {
            $(
                pub fn $set(&mut self, value: bool) {
                    trophy_stat_set_bit(self.0.as_mut(), $bit, value);
                }
            )*

            pub fn set_unused(&mut self, value: u128) {
                trophy_stat_set_range(self.0.as_mut(), 127, $unused_low, value);
            }
        }
    };
}

trophy_stats! {
    TrophyWeaponStats, 9,
    // Legendary Armaments
    /// Id: 2140000 Sword of Night and Flame
    sword_of_night_and_flame, set_sword_of_night_and_flame: 0;
    /// Id: 3090000 Dark Moon Greatsword
    dark_moon_greatsword, set_dark_moon_greatsword: 1;
    /// Id: 3150000 Marais Executioner's Sword
    marais_executioners_sword, set_marais_executioners_sword: 2;
    /// Id: 3170000 Golden Order Greatsword
    golden_order_greatsword, set_golden_order_greatsword: 3;
    /// Id: 4080000 Ruins Greatsword
    ruins_greatsword, set_ruins_greatsword: 4;
    /// Id: 4100000 Grafted Blade Greatsword
    grafted_blade_greatsword, set_grafted_blade_greatsword: 5;
    /// Id: 7100000 Eclipse Shotel
    eclipse_shotel, set_eclipse_shotel: 6;
    /// Id: 12200000 Devourer's Scepter
    devourers_scepter, set_devourers_scepter: 7;
    /// Id: 16090000 Bolt of Gransax
    bolt_of_gransax, set_bolt_of_gransax: 8;
}

trophy_stats! {
    TrophyGoodsStats, 13,
    // Legendary Sorceries and Incantations
    /// Id: 4200 Sorcery Comet Azur
    comet_azur, set_comet_azur: 0;
    /// Id: 4210 Sorcery Founding Rain of Stars
    founding_rain_of_stars, set_founding_rain_of_stars: 1;
    /// Id: 4220 Sorcery Stars of Ruin
    stars_of_ruin, set_stars_of_ruin: 2;
    /// Id: 4361 Sorcery Ranni's Dark Moon
    rannis_dark_moon, set_rannis_dark_moon: 3;
    /// Id: 6110 Incantation Flame of the Fell God
    flame_of_the_fell_god, set_flame_of_the_fell_god: 4;
    /// Id: 6720 Incantation Elden Stars
    elden_stars, set_elden_stars: 5;
    /// Id: 7090 Incantation Greyoll's Roar
    greyolls_roar, set_greyolls_roar: 6;
    // Legendary Ashen Remains
    /// Id: 200000 Black Knife Tiche
    black_knife_tiche, set_black_knife_tiche: 7;
    /// Id: 207000 Mimic Tear Ashes
    mimic_tear_ashes, set_mimic_tear_ashes: 8;
    /// Id: 223000 Cleanrot Knight Finlay
    cleanrot_knight_finlay, set_cleanrot_knight_finlay: 9;
    /// Id: 256000 Ancient Dragon Knight Kristoff
    ancient_dragon_knight_kristoff, set_ancient_dragon_knight_kristoff: 10;
    /// Id: 257000 Redmane Knight Ogha
    redmane_knight_ogha, set_redmane_knight_ogha: 11;
    /// Id: 258000 Lhutel the Headless
    lhutel_the_headless, set_lhutel_the_headless: 12;
}

trophy_stats! {
    TrophyAccessoryStats, 8,
    // Legendary Talismans
    /// Id: 1042 Erdtree's Favor +2
    erdtree_favor_p2, set_erdtree_favor_p2: 0;
    /// Id: 1051 Radagon's Soreseal
    radagons_soreseal, set_radagons_soreseal: 1;
    /// Id: 1140 Moon of Nokstella
    moon_of_nokstella, set_moon_of_nokstella: 2;
    /// Id: 1221 Marika's Soreseal
    marikas_soreseal, set_marikas_soreseal: 3;
    /// Id: 3060 Old Lord's Talisman
    old_lords_talisman, set_old_lords_talisman: 4;
    /// Id: 3070 Radagon Icon
    radagon_icon, set_radagon_icon: 5;
    /// Id: 3090 Godfrey Icon
    godfrey_icon, set_godfrey_icon: 6;
    /// Id: 4003 Dragoncrest Greatshield Talisman
    dragoncrest_greatshield, set_dragoncrest_greatshield: 7;
}

#[repr(C)]
pub struct GameVersionData {
    /// Current version of the game data structure
    pub game_data_version: u32,
    /// Version of the game data read from the last save
    pub last_saved_game_data_version: u32,
    /// Whether the saved game data version is the latest
    pub saved_game_data_version_is_the_latest: bool,
    pub unused: u32,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DisplayBlood {
    Off = 0,
    On = 1,
    Mild = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PerformanceSetting {
    PrioritizeQuality = 0,
    PrioritizeFramerate = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HudType {
    Off = 0,
    On = 1,
    Auto = 2,
}

#[repr(C)]
pub struct GameSettings {
    /// Camera rotation speed
    /// Range: 0-10
    /// Default value is read from Game.Option.Control.RotSpeed property
    pub camera_speed: u8,
    /// Controls the strength of controller rumble
    /// Range: 0-10
    /// Default value is read from Game.Option.Control.Rumble property
    pub controller_rumble_strength: u8,
    /// Controls the brightness of the game
    /// Range: 0-10
    /// Default value is read from Game.Option.Disp.Brightness property
    pub brightness: u8,
    /// Range: 0-10
    /// Default value is read from Game.Option.Sound.SoundType property
    pub sound_type: u8,
    /// Controls the volume of the music
    /// Range: 0-10
    /// Default value is read from Game.Option.Sound.MusicVol property
    pub music_volume: u8,
    /// Controls the volume of sound effects
    /// Range: 0-10
    /// Default value is read from Game.Option.Sound.SeVol property
    pub sfx_volume: u8,
    /// Controls the volume of the voice chat
    /// Range: 0-10
    /// Default value is read from Game.Option.Sound.VoiceVol property
    pub voice_volume: u8,
    /// Controls how blood is displayed
    /// Default value is read from Game.Option.Disp.Blood property
    pub display_blood: DisplayBlood,
    /// Controls whether subtitles are shown
    /// Default value is read from Game.Option.Disp.Subtitle property
    pub show_subtitles: bool,
    /// Type of HUD display
    /// Default value is read from Game.Option.Disp.HUD property
    pub hud_type: HudType,
    /// Controls whether the camera X axis is reversed
    /// Default value is read from Game.Option.Control.RotLR property
    pub reverse_camera_xaxis: bool,
    /// Controls whether the camera Y axis is reversed
    /// Default value is read from Game.Option.Control.RotUD property
    pub reverse_camera_yaxis: bool,
    /// Controls whether camera should automatically lock on to the next target
    /// after the current target is defeated or lost
    pub auto_lock_on: bool,
    /// Controls whether camera automatically adjusts when near walls
    pub camera_auto_wall_recovery: bool,
    unke: u8,
    /// Unused, but read from
    /// Game.Option.Control.JumpButtonL3 property
    pub jump_button_l3: bool,
    /// Controls whether camera recenters vertically when resetting
    /// Default value is read from Game.Option.Control.CameraResetUD property
    pub reset_camera_yaxis: bool,
    /// Controls whether game allowed or not to take control of camera during
    /// certain cinematic moments
    /// Default value is read from Game.Option.Control.CameraDirection property
    pub cinematic_effects: bool,
    unk12: u8,
    /// Controls whether cross-region play is enabled
    /// Doesn't work on PC version
    pub enable_cross_region_play: bool,
    /// Controls whether voice chat is enabled
    /// Locked behind release flag 51 on PC release
    pub voice_chat: bool,
    /// Controls whether gamer tags are shown instead of character names
    /// Locked behind release flag 49 on PC release
    pub show_gamer_tags: bool,
    /// Controls whether manual attack aiming is enabled
    /// Only works on Ringed Finger weapon
    pub manual_attack_aiming: bool,
    /// Controls whether camera automatically targets enemies when attacking
    /// with no lock-on
    pub auto_target: bool,
    /// Controls whether game starts in offline mode
    pub start_offline: bool,
    /// Default value is read from Game.Option.Network.HideWhiteSignInSignEnemyWorld property
    pub send_summon_signs: bool,
    /// Unused setting enabled by release flag 37
    /// Uses GR System Message 103000 for the name
    /// and 3001 for the description
    pub unused_gr_system_103000: bool,
    unk1b: u8,
    /// Controls HDR brightness level
    /// Range: 0-10
    pub hdr_brightness: u8,
    /// Controls HDR max brightness level
    /// Range: 0-10
    pub hdr_max_brightness: u8,
    /// Controls HDR contrast level
    /// Range: 0-10
    pub hdr_contrast: u8,
    /// Controls how game utilizes system resources
    /// Locked behind release flag 39 on PC release
    pub performance_setting: PerformanceSetting,
    /// Controls the master volume
    /// Range: 0-10
    pub master_volume: u8,
    /// Controls whether ray tracing is enabled
    /// Locked behind release flag 38 on PC release
    pub enable_ray_tracing: bool,
    /// Controls whether newly acquired items are marked in inventory
    pub mark_new_items: bool,
    /// Controls whether recent items tab is shown in inventory
    pub show_recent_items: bool,
    /// Constructor-cleared option bytes that the native settings export path skips.
    reserved_options_24: [u8; 10],
    /// Controls whether tutorials are shown
    pub show_tutorials: bool,
    /// Controls whether camera automatically rotates to follow player movement
    pub camera_auto_rotation: bool,
    /// Unused space, will allways be memset on deserialization
    unused_space: [u8; 0x110],
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use crate::cs::{ChrAsm, FaceData, FaceDataBuffer};

    use super::{
        CS_MENU_SYSTEM_SAVE_LOAD_FACE_COUNT, CSMenuFaceData, CSMenuSaveLoad,
        CSMenuSimpleSaveDataChunk, CSMenuSystemSaveLoad, CSMenuSystemSaveLoadFaceChunk,
        DETAIL_STATUS_VIEW_STATE_ENTRY_COUNT, DetailStatusViewStateEntry,
        DetailStatusViewStateSaveData, GameDataMan, GameDataManBloodstainData, GameSettings,
        IGameDataElem, MENU_SORT_STATE_COUNT, MenuInputHistoryPayloadStorage,
        MenuInputHistorySaveData, MenuSortState, MenuSortStates, MenuTitleFlowSaveData,
        PROFILE_SUMMARY_CHARACTER_NAME_CODE_UNIT_COUNT, PROFILE_SUMMARY_RECORD_STRIDE_BYTES,
        PROFILE_SUMMARY_RECORDS_BYTE_EXTENT, PROFILE_SUMMARY_SLOT_COUNT, ProfileSummary,
        ProfileSummaryCharacterName, ProfileSummaryRecord, TrophyAccessoryStats, TrophyGoodsStats,
        TrophyWeaponStats,
    };

    const _: () = {
        assert!(offset_of!(GameDataMan, boss_fight_timer) == 0xb0);
        assert!(size_of::<MenuInputHistorySaveData>() == 0x110);
        assert!(size_of::<ProfileSummaryRecord>() == PROFILE_SUMMARY_RECORD_STRIDE_BYTES);
        assert!(
            size_of::<ProfileSummary>()
                == offset_of!(ProfileSummary, records) + PROFILE_SUMMARY_RECORDS_BYTE_EXTENT
        );
    };

    #[test]
    fn public_layout_offsets_match_static_re() {
        assert_eq!(0x24, offset_of!(GameSettings, reserved_options_24));
        assert_eq!(0x2e, offset_of!(GameSettings, show_tutorials));
        assert_eq!(0x2f, offset_of!(GameSettings, camera_auto_rotation));
        assert_eq!(0x30, offset_of!(GameSettings, unused_space));
        assert_eq!(0x140, size_of::<GameSettings>());
        assert_eq!(0x38, offset_of!(GameDataMan, tutorial_data));
        assert_eq!(0x40, offset_of!(GameDataMan, has_bloodstain));
        assert_eq!(0x48, offset_of!(GameDataMan, bloodstain));
        assert_eq!(0x50, offset_of!(GameDataMan, bloodstain_entity_id));
        assert_eq!(0x58, offset_of!(GameDataMan, game_settings));
        assert_eq!(0x0, offset_of!(GameDataManBloodstainData, msb_position));
        assert_eq!(0xc, offset_of!(GameDataManBloodstainData, rotation));
        assert_eq!(
            0x30,
            offset_of!(GameDataManBloodstainData, base_hero_point_2)
        );
        assert_eq!(0x34, offset_of!(GameDataManBloodstainData, rune_count));
        assert_eq!(0x38, offset_of!(GameDataManBloodstainData, block_id));
        assert_eq!(
            0x3c,
            offset_of!(GameDataManBloodstainData, has_death_blight_effect)
        );
        assert_eq!(0x40, size_of::<GameDataManBloodstainData>());
        assert_eq!(0x60, offset_of!(GameDataMan, menu_system_save_load));
        assert_eq!(0x68, offset_of!(GameDataMan, menu_profile_save_load));
        assert_eq!(0x70, offset_of!(GameDataMan, key_config_save_load));
        assert_eq!(0x78, offset_of!(GameDataMan, profile_summary));
        assert_eq!(0x80, offset_of!(GameDataMan, pc_option_data));
        assert_eq!(0x88, offset_of!(GameDataMan, solo_break_in_state));
        assert_eq!(0x8d, offset_of!(GameDataMan, request_restore_hp));
        assert_eq!(0x8e, offset_of!(GameDataMan, recovery_state_byte));
        assert_eq!(
            0x8f,
            offset_of!(GameDataMan, award_phantom_great_rune_requested)
        );
        assert_eq!(0x9c, offset_of!(GameDataMan, save_state_after_death_count));
        assert_eq!(0xa0, offset_of!(GameDataMan, play_time));
        assert_eq!(0xa4, offset_of!(GameDataMan, boss_fight_state_a4));
        assert_eq!(0xb0, offset_of!(GameDataMan, boss_fight_timer));
        assert_eq!(0xc0, offset_of!(GameDataMan, boss_fight_active));
        assert_eq!(0xc4, offset_of!(GameDataMan, white_phantom_count));
        assert_eq!(0xd0, offset_of!(GameDataMan, is_in_boss_fight));
        assert_eq!(0xd4, offset_of!(GameDataMan, death_state));
        assert_eq!(0x124, offset_of!(GameDataMan, random_appear_lot_slot));
        assert_eq!(0x128, offset_of!(GameDataMan, host_ng_level));
        assert_eq!(0x12c, offset_of!(GameDataMan, host_random_appear_lot_slot));
        assert_eq!(0x130, offset_of!(GameDataMan, backup_ng_level));
        assert_eq!(
            0x134,
            offset_of!(GameDataMan, backup_random_appear_lot_slot)
        );
        assert_eq!(
            0x138,
            offset_of!(GameDataMan, host_world_values_update_requested)
        );
        assert_eq!(0x139, offset_of!(GameDataMan, host_world_values_applied));
        assert_eq!(
            0x140,
            offset_of!(GameDataMan, player_game_data_debug_menu_root)
        );
        assert_eq!(0x148, offset_of!(GameDataMan, param_reload_requested));
        assert_eq!(
            0x14c,
            offset_of!(GameDataMan, last_applied_chara_init_param_id)
        );
        assert_eq!(
            0x150,
            offset_of!(GameDataMan, default_random_appear_lot_slot)
        );
        assert_eq!(0x158, size_of::<GameDataMan>());
    }

    #[test]
    fn menu_system_save_load_layout_matches_static_re() {
        assert_eq!(0x8, size_of::<IGameDataElem>());
        assert_eq!(0x0, offset_of!(CSMenuSaveLoad, game_data_elem));
        assert_eq!(0x8, offset_of!(CSMenuSaveLoad, definition_id));
        assert_eq!(0xa, offset_of!(CSMenuSaveLoad, definition_variant));
        assert_eq!(0xc, offset_of!(CSMenuSaveLoad, serialized_capacity_bytes));
        assert_eq!(0x10, size_of::<CSMenuSaveLoad>());
        assert_eq!(0x120, size_of::<FaceDataBuffer>());
        assert_eq!(0x0, offset_of!(CSMenuFaceData, vftable));
        assert_eq!(0x8, offset_of!(CSMenuFaceData, header));
        assert_eq!(0xc, offset_of!(CSMenuFaceData, face_data_buffer));
        assert_eq!(0x12c, offset_of!(CSMenuFaceData, footer));
        assert_eq!(0x130, size_of::<CSMenuFaceData>());
        assert_eq!(0xf, CS_MENU_SYSTEM_SAVE_LOAD_FACE_COUNT);
        assert_eq!(0x8, offset_of!(CSMenuSystemSaveLoadFaceChunk, entries));
        assert_eq!(
            0x11e0,
            offset_of!(CSMenuSystemSaveLoadFaceChunk, entries) + 0x11d8
        );
        assert_eq!(0x11e0, size_of::<CSMenuSystemSaveLoadFaceChunk>() - 0x8);
        assert_eq!(0x11e8, size_of::<CSMenuSystemSaveLoadFaceChunk>());
        assert_eq!(0x10, DETAIL_STATUS_VIEW_STATE_ENTRY_COUNT);
        assert_eq!(
            0x0,
            offset_of!(DetailStatusViewStateEntry, selected_index_le_bytes)
        );
        assert_eq!(
            0x2,
            offset_of!(DetailStatusViewStateEntry, scroll_position_le_bytes)
        );
        assert_eq!(
            0x6,
            offset_of!(DetailStatusViewStateEntry, packed_view_state_le_bytes)
        );
        assert_eq!(0xa, offset_of!(DetailStatusViewStateEntry, active));
        assert_eq!(0xb, offset_of!(DetailStatusViewStateEntry, reserved_tail));
        assert_eq!(0x10, size_of::<DetailStatusViewStateEntry>());
        assert_eq!(0x0, offset_of!(MenuTitleFlowSaveData, save_slot));
        assert_eq!(0x4, offset_of!(MenuTitleFlowSaveData, flags));
        assert_eq!(0x8, offset_of!(MenuTitleFlowSaveData, tail_qwords_08));
        assert_eq!(0x20, size_of::<MenuTitleFlowSaveData>());
        assert_eq!(
            0x8,
            offset_of!(CSMenuSimpleSaveDataChunk<MenuTitleFlowSaveData>, data)
        );
        assert_eq!(
            0x28,
            size_of::<CSMenuSimpleSaveDataChunk<MenuTitleFlowSaveData>>()
        );
        assert_eq!(
            0x8,
            offset_of!(
                CSMenuSimpleSaveDataChunk<DetailStatusViewStateSaveData>,
                data
            )
        );
        assert_eq!(
            0x108,
            size_of::<CSMenuSimpleSaveDataChunk<DetailStatusViewStateSaveData>>()
        );
        assert_eq!(0x0, offset_of!(MenuInputHistorySaveData, header_word_0));
        assert_eq!(0x2, offset_of!(MenuInputHistorySaveData, header_word_2));
        assert_eq!(
            0x4,
            offset_of!(MenuInputHistorySaveData, payload_byte_count)
        );
        assert_eq!(0x8, offset_of!(MenuInputHistorySaveData, payload_storage));
        assert_eq!(0x106, size_of::<MenuInputHistoryPayloadStorage>());
        assert_eq!(0x110, size_of::<MenuInputHistorySaveData>());
        assert_eq!(
            0x118,
            size_of::<CSMenuSimpleSaveDataChunk<MenuInputHistorySaveData>>()
        );
        assert_eq!(0x0, offset_of!(CSMenuSystemSaveLoad, base));
        assert_eq!(0x10, offset_of!(CSMenuSystemSaveLoad, face_chunk));
        assert_eq!(
            0x11f8,
            offset_of!(CSMenuSystemSaveLoad, title_flow_save_data)
        );
        assert_eq!(
            0x1220,
            offset_of!(CSMenuSystemSaveLoad, detail_status_view_state)
        );
        assert_eq!(0x1328, offset_of!(CSMenuSystemSaveLoad, menu_input_history));
        assert_eq!(0x1440, offset_of!(CSMenuSystemSaveLoad, menu_sort_states));
        assert_eq!(0x4, size_of::<MenuSortState>());
        assert_eq!(0x18, MENU_SORT_STATE_COUNT);
        assert_eq!(
            MENU_SORT_STATE_COUNT * size_of::<MenuSortState>(),
            size_of::<MenuSortStates>()
        );
        assert_eq!(0x14a0, size_of::<CSMenuSystemSaveLoad>());
    }

    #[test]
    fn menu_sort_state_exposes_known_packed_bits() {
        let state = MenuSortState::from_raw(0x8000_0068);

        assert_eq!(0x8000_0068, state.raw());
        assert!(state.high_bit_flag());
        assert_eq!(0x68, state.value_bits());
    }

    #[test]
    fn trophy_stats_layout_and_accessors_match_bit_storage() {
        assert_eq!(0x10, size_of::<TrophyWeaponStats<[u8; 0x10]>>());
        assert_eq!(0x10, size_of::<TrophyGoodsStats<[u8; 0x10]>>());
        assert_eq!(0x10, size_of::<TrophyAccessoryStats<[u8; 0x10]>>());

        let mut weapon_stats = TrophyWeaponStats([0; 0x10]);
        assert!(!weapon_stats.sword_of_night_and_flame());
        weapon_stats.set_sword_of_night_and_flame(true);
        assert!(weapon_stats.sword_of_night_and_flame());
        assert_eq!(0, weapon_stats.unused());
    }

    #[test]
    fn profile_summary_character_name_decodes_until_nul() {
        let name = ProfileSummaryCharacterName {
            code_units: [
                b'A' as u16,
                b'B' as u16,
                0,
                b'C' as u16,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            native_gap_20: [0; 0x4],
        };

        assert_eq!(&[b'A' as u16, b'B' as u16], name.code_units_until_nul());
        assert_eq!("AB", name.try_to_string().unwrap());
        assert_eq!("AB", name.to_string_lossy());
    }

    #[test]
    fn profile_summary_character_name_decodes_full_capacity_without_nul() {
        let name = ProfileSummaryCharacterName {
            code_units: [
                b'A' as u16,
                b'B' as u16,
                b'C' as u16,
                b'D' as u16,
                b'E' as u16,
                b'F' as u16,
                b'G' as u16,
                b'H' as u16,
                b'I' as u16,
                b'J' as u16,
                b'K' as u16,
                b'L' as u16,
                b'M' as u16,
                b'N' as u16,
                b'O' as u16,
                b'P' as u16,
            ],
            native_gap_20: [0; 0x4],
        };

        assert_eq!(16, name.code_units_until_nul().len());
        assert_eq!("ABCDEFGHIJKLMNOP", name.try_to_string().unwrap());
    }

    #[test]
    fn profile_summary_character_name_reports_invalid_utf16() {
        let name = ProfileSummaryCharacterName {
            code_units: [0xd800, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            native_gap_20: [0; 0x4],
        };

        assert!(name.try_to_string().is_err());
        assert_eq!("�", name.to_string_lossy());
    }

    #[test]
    fn profile_summary_layout_matches_static_re() {
        assert_eq!(
            0x20,
            PROFILE_SUMMARY_CHARACTER_NAME_CODE_UNIT_COUNT * size_of::<u16>()
        );
        assert_eq!(0x20, offset_of!(ProfileSummaryCharacterName, native_gap_20));
        assert_eq!(0x24, size_of::<ProfileSummaryCharacterName>());
        assert_eq!(0x170, size_of::<FaceData>());
        assert_eq!(0xe8, size_of::<ChrAsm>());
        assert_eq!(10, PROFILE_SUMMARY_SLOT_COUNT);
        assert_eq!(0x00, offset_of!(ProfileSummary, vftable));
        assert_eq!(0x08, offset_of!(ProfileSummary, active_slots));
        assert_eq!(0x18, offset_of!(ProfileSummary, records));
        assert_eq!(
            PROFILE_SUMMARY_SLOT_COUNT,
            size_of::<[u8; PROFILE_SUMMARY_SLOT_COUNT]>()
        );
        assert_eq!(0x00, offset_of!(ProfileSummaryRecord, name));
        assert_eq!(0x24, offset_of!(ProfileSummaryRecord, level));
        assert_eq!(0x28, offset_of!(ProfileSummaryRecord, play_time));
        assert_eq!(0x2c, offset_of!(ProfileSummaryRecord, rune_memory));
        assert_eq!(0x30, offset_of!(ProfileSummaryRecord, map));
        assert_eq!(0x38, offset_of!(ProfileSummaryRecord, face_data));
        assert_eq!(0x1a8, offset_of!(ProfileSummaryRecord, chr_asm));
        assert_eq!(0x290, offset_of!(ProfileSummaryRecord, gender));
        assert_eq!(0x291, offset_of!(ProfileSummaryRecord, archetype));
        assert_eq!(0x292, offset_of!(ProfileSummaryRecord, starting_gift));
        assert_eq!(
            0x293,
            offset_of!(ProfileSummaryRecord, player_game_data_unkc4)
        );
        assert_eq!(
            0x294,
            offset_of!(ProfileSummaryRecord, profile_renderer_byte_294)
        );
        assert_eq!(0x295, offset_of!(ProfileSummaryRecord, loadable_flag_295));
        assert_eq!(0x296, offset_of!(ProfileSummaryRecord, record_tail_296));
        assert_eq!(
            PROFILE_SUMMARY_RECORD_STRIDE_BYTES,
            size_of::<ProfileSummaryRecord>()
        );
        assert_eq!(
            PROFILE_SUMMARY_RECORDS_BYTE_EXTENT,
            size_of::<[ProfileSummaryRecord; PROFILE_SUMMARY_SLOT_COUNT]>()
        );
        assert_eq!(
            offset_of!(ProfileSummary, records) + PROFILE_SUMMARY_RECORDS_BYTE_EXTENT,
            size_of::<ProfileSummary>()
        );
    }
}
