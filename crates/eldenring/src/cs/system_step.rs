use std::{borrow::Cow, ptr::NonNull};

use fromsoftware_shared::{FromStatic, InstanceResult, load_static_indirect};

/// Common layout of `CS::EzChildStep<T>` task-owner fields.
#[repr(C)]
pub struct EzChildStep<T> {
    vftable: usize,
    /// Child step task registered through `EzChildStepBase::RegisterStepTask`.
    pub task: Option<NonNull<T>>,
    unk10: usize,
    finish_helper: usize,
}

impl<T> EzChildStep<T> {
    pub fn task(&self) -> Option<NonNull<T>> {
        self.task
    }
}

/// Root system step registered in `CSTaskImp`'s SystemStep task group.
#[repr(C)]
pub struct CSSystemStep {
    unknown_000: [u8; 0x318],
    pub title_flow_step: EzChildStep<TitleFlowStep>,
    unknown_338: [u8; 0xc8],
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

/// Title flow step that owns the move-map/list step during title/menu boot.
#[repr(C)]
pub struct TitleFlowStep {
    unknown_000: [u8; 0xc0],
    pub move_map_list_step: EzChildStep<CSMoveMapListStep>,
}

impl TitleFlowStep {
    pub fn move_map_list_step(&self) -> Option<NonNull<CSMoveMapListStep>> {
        self.move_map_list_step.task()
    }
}

/// Move-map list step that owns the game-root step while the title flow is active.
#[repr(C)]
pub struct CSMoveMapListStep {
    unknown_000: [u8; 0xb8],
    pub game_root_step: EzChildStep<GameRootStep>,
}

impl CSMoveMapListStep {
    pub fn game_root_step(&self) -> Option<NonNull<GameRootStep>> {
        self.game_root_step.task()
    }
}

/// Game-root step that owns the title step while booted into the title flow.
#[repr(C)]
pub struct GameRootStep {
    unknown_000: [u8; 0x118],
    pub title_step: EzChildStep<TitleStep>,
    unknown_138: [u8; 0x20],
}

impl GameRootStep {
    pub fn title_step(&self) -> Option<NonNull<TitleStep>> {
        self.title_step.task()
    }
}

/// `CS::TitleStep`, the native title-flow owner step.
#[repr(C)]
pub struct TitleStep {
    vftable: usize,
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

    use super::{CSMoveMapListStep, CSSystemStep, GameRootStep, TitleFlowStep};

    #[test]
    fn layout_offsets_match_static_re() {
        assert_eq!(offset_of!(CSSystemStep, title_flow_step), 0x318);
        assert_eq!(offset_of!(TitleFlowStep, move_map_list_step), 0xc0);
        assert_eq!(offset_of!(CSMoveMapListStep, game_root_step), 0xb8);
        assert_eq!(offset_of!(GameRootStep, title_step), 0x118);
        assert_eq!(size_of::<CSSystemStep>(), 0x400);
        assert_eq!(size_of::<GameRootStep>(), 0x158);
    }
}
