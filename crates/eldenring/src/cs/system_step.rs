use std::{borrow::Cow, marker::PhantomData, ptr::NonNull};

use fromsoftware_shared::{FromStatic, InstanceResult, StepperStates, load_static_indirect};

use super::{CSEzTask, CSEzUpdateTask};
use crate::fd4::{FD4StepBase, FD4TaskBase};

/// Common layout of `CS::EzChildStep<T>` task-owner fields.
#[repr(C)]
pub struct EzChildStep<T> {
    vftable: usize,
    /// Child step task registered through `EzChildStepBase::RegisterStepTask`.
    pub task: Option<NonNull<T>>,
    /// Latches after `EzChildStepBase::RequestFinish` has invoked the finish helper.
    pub finish_latch: u8,
    pub finish_helper: CSSetFinishHelper<T>,
}

impl<T> EzChildStep<T> {
    pub fn task(&self) -> Option<NonNull<T>> {
        self.task
    }
}

/// `CS::CSSetFinishHelper<T>` object stored inside each child-step wrapper.
#[repr(C)]
pub struct CSSetFinishHelper<T> {
    vftable: usize,
    _phantom_data: PhantomData<T>,
}

/// State indices for `CS::CSSystemStep`.
///
/// Variant order follows the native `CSSystemStep::STEP_*` labels in the
/// deobfuscated executable.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, StepperStates)]
pub enum CSSystemStepState {
    NotExecuting = -1,
    Init = 0,
    InitForBootPhase1 = 1,
    WaitForBootPhase1 = 2,
    InitForBootPhase2 = 3,
    WaitForBootPhase2 = 4,
    InitForBootPhase3 = 5,
    WaitForBootPhase3 = 6,
    InitForBootPhase4 = 7,
    WaitForBootPhase4 = 8,
    InitForBootPhase5 = 9,
    WaitForBootPhase5 = 10,
    InitForGameFlow = 11,
    WaitForGameFlow = 12,
    FinishForGameFlow = 13,
    WaitForPreGraphicsStep = 14,
    WaitForGraphicsStep = 15,
    WaitForPadStep = 16,
    WaitForResStep = 17,
    WaitForSoundStep = 18,
    WaitForFileStep = 19,
    Finish = 20,
}

/// Root system step registered in `CSTaskImp`'s SystemStep task group.
#[repr(C)]
pub struct CSSystemStep {
    pub stepper: FD4StepBase<Self, FD4TaskBase, CSSystemStepState>,
    pub cs_task: CSStepTaskFields,
    /// Child task wrapper; RTTI shows the task `CS::CSDelayDeleteStep` derives [`FD4TaskBase`].
    pub delay_delete_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSDbgMenuStep` derives [`FD4TaskBase`].
    pub dbg_menu_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSResStep` derives [`FD4TaskBase`].
    pub res_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSFileStep` derives [`FD4TaskBase`].
    pub file_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSPadStep` derives [`FD4TaskBase`].
    pub pad_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSSoundStep` derives [`FD4TaskBase`].
    pub sound_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSGraphicsStep` derives [`FD4TaskBase`].
    pub graphics_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSScaleformStep` derives [`FD4TaskBase`].
    pub scaleform_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSFD4LocationStep` derives [`FD4TaskBase`].
    pub fd4_location_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSRemoStep` derives [`FD4TaskBase`].
    pub remo_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSCameraStep` derives [`FD4TaskBase`].
    pub camera_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::DbgRemoteStep` derives [`FD4TaskBase`].
    pub dbg_remote_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSDbgDispStep` derives [`FD4TaskBase`].
    pub dbg_disp_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSReportSystemStep` derives [`FD4TaskBase`].
    pub report_system_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSDbgIdNameStep` derives [`FD4TaskBase`].
    pub dbg_id_name_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSPlaylogSystemStep` derives [`FD4TaskBase`].
    pub playlog_system_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSBehStringStep` derives [`FD4TaskBase`].
    pub beh_string_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSSystemParamStep` derives [`FD4TaskBase`].
    pub system_param_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSEventFlagResStep` derives [`FD4TaskBase`].
    pub event_flag_res_step: EzChildStep<FD4TaskBase>,
    pub title_flow_step: EzChildStep<TitleFlowStep>,
    pub update_tasks: [CSEzUpdateTask<CSEzTask, CSSystemStep>; 5],
}

impl CSSystemStep {
    pub fn title_flow_step(&self) -> Option<NonNull<TitleFlowStep>> {
        self.title_flow_step.task()
    }
}

impl FromStatic for CSSystemStep {
    fn name() -> Cow<'static, str> {
        Cow::Borrowed("CSSystemStep")
    }

    fn instance_ptr() -> InstanceResult<*mut Self> {
        unsafe { load_static_indirect(crate::rva::get().cs_system_step) }
    }
}

/// State indices for `CS::TitleFlowStep`.
///
/// Variant order follows the native `TitleFlowStep::STEP_*` labels in the
/// deobfuscated executable.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, StepperStates)]
pub enum TitleFlowStepState {
    NotExecuting = -1,
    Init = 0,
    Wait = 1,
    Finish = 2,
}

/// Constructor mode stored by [`TitleFlowStep`].
///
/// The recovered `CSSystemStep::STEP_Init_forGameFlow` allocation passes `1`,
/// but the native constructor accepts a raw `i32` parameter and no complete
/// validation/switch boundary has been recovered. Use [`Self::from_raw`] for a
/// developer-friendly view without claiming the memory field is exhaustive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TitleFlowMode {
    GameFlow,
    Unknown(i32),
}

impl TitleFlowMode {
    pub fn from_raw(raw: i32) -> Self {
        match raw {
            1 => Self::GameFlow,
            value => Self::Unknown(value),
        }
    }

    pub fn raw(self) -> i32 {
        match self {
            Self::GameFlow => 1,
            Self::Unknown(value) => value,
        }
    }
}

/// Title flow step that owns the move-map/list step during title/menu boot.
///
/// RTTI identifies this as `FD4::FD4StepTemplateBase<CS::TitleFlowStep,
/// FD4::FD4TaskBase>`.
#[repr(C)]
pub struct TitleFlowStep {
    pub stepper: FD4StepBase<Self, FD4TaskBase, TitleFlowStepState>,
    pub cs_task: CSStepTaskFields,
    /// Constructor-supplied raw flow mode.
    ///
    /// The normal `CSSystemStep` path passes `1`; retain raw storage because
    /// unrecovered/native callers could still pass other values.
    pub flow_mode: i32,
    pub move_map_list_step: EzChildStep<CSMoveMapListStep>,
}

impl TitleFlowStep {
    pub fn flow_mode(&self) -> TitleFlowMode {
        TitleFlowMode::from_raw(self.flow_mode)
    }

    pub fn move_map_list_step(&self) -> Option<NonNull<CSMoveMapListStep>> {
        self.move_map_list_step.task()
    }
}

/// State indices for `CS::CSMoveMapListStep`.
///
/// Variant order follows the native `CSMoveMapListStep::STEP_*` labels in the
/// deobfuscated executable.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, StepperStates)]
pub enum CSMoveMapListStepState {
    NotExecuting = -1,
    Init = 0,
    InitForData = 1,
    WaitForData = 2,
    Wait = 3,
    Finish = 4,
}

/// Move-map list step that owns the game-root step while the title flow is active.
///
/// RTTI identifies this as `FD4::FD4StepTemplateBase<CS::CSMoveMapListStep,
/// FD4::FD4TaskBase>`.
#[repr(C)]
pub struct CSMoveMapListStep {
    pub stepper: FD4StepBase<Self, FD4TaskBase, CSMoveMapListStepState>,
    pub cs_task: CSStepTaskFields,
    pub game_root_step: EzChildStep<GameRootStep>,
}

impl CSMoveMapListStep {
    pub fn game_root_step(&self) -> Option<NonNull<GameRootStep>> {
        self.game_root_step.task()
    }
}

/// State indices for `CS::GameRootStep`.
///
/// Variant order follows the native `GameRootStep::STEP_*` labels in the
/// deobfuscated executable.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, StepperStates)]
pub enum GameRootStepState {
    NotExecuting = -1,
    BootInitA = 0,
    BootInitWaitA = 1,
    BootInitB = 2,
    BootInitWaitB = 3,
    BootInitC = 4,
    BootInitWaitC = 5,
    BootInitD = 6,
    BootInitWaitD = 7,
    BootUpdate = 8,
    ShutdownWait = 9,
    Shutdown = 10,
}

/// `CS::CSStepTask<T>` tail fields appended after the FD4 stepper base.
#[repr(C)]
pub struct CSStepTaskFields {
    /// Warmup/finish counter used by CS step-task finish handling.
    pub finish_counter: u32,
    /// Native finish-request byte.
    ///
    /// Kept as a raw byte because the native layout does not define a distinct
    /// public finish-request type here.
    pub finish_requested: u8,
}

/// Game-root step that owns the title step while booted into the title flow.
///
/// RTTI identifies this as `FD4::FD4StepTemplateBase<CS::GameRootStep,
/// FD4::FD4TaskBase>`.
#[repr(C)]
pub struct GameRootStep {
    pub stepper: FD4StepBase<Self, FD4TaskBase, GameRootStepState>,
    pub cs_task: CSStepTaskFields,
    /// Child task wrapper; RTTI shows the task `CS::MsgStep` derives [`FD4TaskBase`].
    pub msg_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::CSDummyStep` derives [`FD4TaskBase`].
    pub dummy_step: EzChildStep<FD4TaskBase>,
    /// Child task wrapper; RTTI shows the task `CS::ParamStep` derives [`FD4TaskBase`].
    pub param_step: EzChildStep<FD4TaskBase>,
    pub title_step: EzChildStep<TitleStep>,
    /// Child task wrapper; RTTI shows the task `CS::CSRegulationStep` derives [`FD4TaskBase`].
    pub regulation_step: EzChildStep<FD4TaskBase>,
}

impl GameRootStep {
    pub fn msg_step(&self) -> Option<NonNull<FD4TaskBase>> {
        self.msg_step.task()
    }

    pub fn dummy_step(&self) -> Option<NonNull<FD4TaskBase>> {
        self.dummy_step.task()
    }

    pub fn param_step(&self) -> Option<NonNull<FD4TaskBase>> {
        self.param_step.task()
    }

    pub fn title_step(&self) -> Option<NonNull<TitleStep>> {
        self.title_step.task()
    }

    pub fn regulation_step(&self) -> Option<NonNull<FD4TaskBase>> {
        self.regulation_step.task()
    }
}

/// State indices for `CS::TitleStep`.
///
/// Variant order follows the native `TitleStep::STEP_*` labels in the
/// deobfuscated executable.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, StepperStates)]
pub enum TitleStepState {
    NotExecuting = -1,
    InitMenu = 0,
    BeginXr117Dialog = 1,
    BeginLogo = 2,
    BeginTitle = 3,
    BeginNewGame = 4,
    PlayGame = 5,
    GameStepWait = 6,
    EndFlow = 7,
    EndFlowWait = 8,
    NextLap = 9,
    MenuJobWait = 10,
    Finish = 11,
}

/// `CS::TitleStep`, the native title-flow owner step.
///
/// RTTI identifies this as `FD4::FD4StepTemplateBase<CS::TitleStep,
/// FD4::FD4TaskBase>`.
#[repr(C)]
pub struct TitleStep {
    pub stepper: FD4StepBase<Self, FD4TaskBase, TitleStepState>,
}

impl TitleStep {
    pub fn vtable_rva() -> u32 {
        crate::rva::get().title_step_vmt
    }

    pub fn state_table_rva() -> u32 {
        crate::rva::get().title_step_state_table
    }
}

#[cfg(test)]
mod tests {
    use std::mem::{offset_of, size_of};

    use crate::fd4::FD4TaskBase;

    use super::{
        CSMoveMapListStep, CSSetFinishHelper, CSStepTaskFields, CSSystemStep, EzChildStep,
        GameRootStep, TitleFlowStep, TitleStep,
    };

    #[test]
    fn layout_offsets_match_static_re() {
        assert_eq!(size_of::<CSSetFinishHelper<FD4TaskBase>>(), 0x8);
        assert_eq!(size_of::<EzChildStep<FD4TaskBase>>(), 0x20);
        assert_eq!(offset_of!(EzChildStep<FD4TaskBase>, task), 0x8);
        assert_eq!(offset_of!(EzChildStep<FD4TaskBase>, finish_latch), 0x10);
        assert_eq!(offset_of!(EzChildStep<FD4TaskBase>, finish_helper), 0x18);
        assert_eq!(size_of::<CSStepTaskFields>(), 0x8);
        assert_eq!(offset_of!(CSStepTaskFields, finish_counter), 0x0);
        assert_eq!(offset_of!(CSStepTaskFields, finish_requested), 0x4);
        assert_eq!(offset_of!(CSSystemStep, stepper), 0x0);
        assert_eq!(offset_of!(CSSystemStep, cs_task), 0xb0);
        assert_eq!(offset_of!(CSSystemStep, delay_delete_step), 0xb8);
        assert_eq!(offset_of!(CSSystemStep, dbg_menu_step), 0xd8);
        assert_eq!(offset_of!(CSSystemStep, res_step), 0xf8);
        assert_eq!(offset_of!(CSSystemStep, file_step), 0x118);
        assert_eq!(offset_of!(CSSystemStep, pad_step), 0x138);
        assert_eq!(offset_of!(CSSystemStep, sound_step), 0x158);
        assert_eq!(offset_of!(CSSystemStep, graphics_step), 0x178);
        assert_eq!(offset_of!(CSSystemStep, scaleform_step), 0x198);
        assert_eq!(offset_of!(CSSystemStep, fd4_location_step), 0x1b8);
        assert_eq!(offset_of!(CSSystemStep, remo_step), 0x1d8);
        assert_eq!(offset_of!(CSSystemStep, camera_step), 0x1f8);
        assert_eq!(offset_of!(CSSystemStep, dbg_remote_step), 0x218);
        assert_eq!(offset_of!(CSSystemStep, dbg_disp_step), 0x238);
        assert_eq!(offset_of!(CSSystemStep, report_system_step), 0x258);
        assert_eq!(offset_of!(CSSystemStep, dbg_id_name_step), 0x278);
        assert_eq!(offset_of!(CSSystemStep, playlog_system_step), 0x298);
        assert_eq!(offset_of!(CSSystemStep, beh_string_step), 0x2b8);
        assert_eq!(offset_of!(CSSystemStep, system_param_step), 0x2d8);
        assert_eq!(offset_of!(CSSystemStep, event_flag_res_step), 0x2f8);
        assert_eq!(offset_of!(CSSystemStep, title_flow_step), 0x318);
        assert_eq!(offset_of!(CSSystemStep, update_tasks), 0x338);
        assert_eq!(offset_of!(TitleFlowStep, stepper), 0x0);
        assert_eq!(offset_of!(TitleFlowStep, cs_task), 0xb0);
        assert_eq!(offset_of!(TitleFlowStep, flow_mode), 0xb8);
        assert_eq!(offset_of!(TitleFlowStep, move_map_list_step), 0xc0);
        assert_eq!(size_of::<TitleFlowStep>(), 0xe0);
        assert_eq!(offset_of!(CSMoveMapListStep, stepper), 0x0);
        assert_eq!(offset_of!(CSMoveMapListStep, cs_task), 0xb0);
        assert_eq!(offset_of!(CSMoveMapListStep, game_root_step), 0xb8);
        assert_eq!(size_of::<CSMoveMapListStep>(), 0xd8);
        assert_eq!(offset_of!(GameRootStep, stepper), 0x0);
        assert_eq!(offset_of!(GameRootStep, cs_task), 0xb0);
        assert_eq!(offset_of!(GameRootStep, msg_step), 0xb8);
        assert_eq!(offset_of!(GameRootStep, dummy_step), 0xd8);
        assert_eq!(offset_of!(GameRootStep, param_step), 0xf8);
        assert_eq!(offset_of!(GameRootStep, title_step), 0x118);
        assert_eq!(offset_of!(GameRootStep, regulation_step), 0x138);
        assert_eq!(offset_of!(TitleStep, stepper), 0x0);
        assert_eq!(size_of::<CSSystemStep>(), 0x400);
        assert_eq!(size_of::<GameRootStep>(), 0x158);
    }
}
