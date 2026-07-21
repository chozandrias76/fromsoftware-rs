use std::fmt::Debug;

use debug::UiExt;
use eldenring::{
    cs::{
        CSEzUpdateTask, CSMoveMapListStep, CSSetFinishHelper, CSStepTaskFields, CSSystemStep,
        EzChildStep, GameRootStep, TitleFlowStep, TitleStep,
    },
    fd4::FD4StepTemplateBase,
};
use fromsoftware_shared::StepperStates;
use hudhook::imgui::{TableColumnSetup, Ui};

use super::{DebugDisplay, DisplayUiExt};

impl<Subject, Base, States> DebugDisplay for FD4StepTemplateBase<Subject, Base, States>
where
    States: StepperStates + Debug + Copy,
{
    fn render_debug(&self, ui: &Ui) {
        ui.debug_copiable("Current State", self.current_state);
        ui.debug_copiable("Requested State", self.requested_state);
        ui.pointer("Stepper Functions", self.stepper_fns.as_ptr());
        ui.pointer("Debug State Label", self.debug_state_label);
    }
}

impl<T> DebugDisplay for EzChildStep<T> {
    fn render_debug(&self, ui: &Ui) {
        match self.task {
            Some(task) => ui.pointer("Task", task.as_ptr()),
            None => ui.text("Task: None"),
        }
        ui.display("Finish Latch", self.finish_latch);
        ui.nested("Finish Helper", &self.finish_helper);
    }
}

impl<T> DebugDisplay for CSSetFinishHelper<T> {
    fn render_debug(&self, ui: &Ui) {
        ui.text("Known layout: vtable prefix and type marker only.");
    }
}

impl DebugDisplay for CSStepTaskFields {
    fn render_debug(&self, ui: &Ui) {
        ui.display("Finish Counter", self.finish_counter);
        ui.display("Finish Requested", self.finish_requested);
    }
}

impl<TEzTask, TSubject> DebugDisplay for CSEzUpdateTask<TEzTask, TSubject> {
    fn render_debug(&self, ui: &Ui) {
        ui.pointer("Subject", self.subject.as_ptr());
        ui.display_copiable("Executor", format!("{:p}", self.executor as *const ()));
    }
}

impl DebugDisplay for CSSystemStep {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Stepper", &self.stepper);
        ui.nested("CS Task", &self.cs_task);

        ui.header("Child Steps", || {
            ui.nested("Delay Delete", &self.delay_delete_step);
            ui.nested("Debug Menu", &self.dbg_menu_step);
            ui.nested("Resources", &self.res_step);
            ui.nested("File", &self.file_step);
            ui.nested("Pad", &self.pad_step);
            ui.nested("Sound", &self.sound_step);
            ui.nested("Graphics", &self.graphics_step);
            ui.nested("Scaleform", &self.scaleform_step);
            ui.nested("FD4 Location", &self.fd4_location_step);
            ui.nested("Remo", &self.remo_step);
            ui.nested("Camera", &self.camera_step);
            ui.nested("Debug Remote", &self.dbg_remote_step);
            ui.nested("Debug Display", &self.dbg_disp_step);
            ui.nested("Report System", &self.report_system_step);
            ui.nested("Debug ID Name", &self.dbg_id_name_step);
            ui.nested("Playlog System", &self.playlog_system_step);
            ui.nested("Behavior String", &self.beh_string_step);
            ui.nested("System Param", &self.system_param_step);
            ui.nested("Event Flag Resources", &self.event_flag_res_step);
            ui.nested("Title Flow Wrapper", &self.title_flow_step);
        });

        ui.nested_opt(
            "Title Flow Step",
            self.title_flow_step().map(|step| unsafe { step.as_ref() }),
        );

        ui.table(
            "system-step-update-tasks",
            [
                TableColumnSetup::new("Index"),
                TableColumnSetup::new("Subject"),
                TableColumnSetup::new("Executor"),
            ],
            self.update_tasks.iter(),
            |ui, i, task| {
                ui.table_next_column();
                ui.text(format!("{i}"));
                ui.table_next_column();
                ui.text(format!("{:p}", task.subject.as_ptr()));
                ui.table_next_column();
                ui.text(format!("{:p}", task.executor as *const ()));
            },
        );
    }
}

impl DebugDisplay for TitleFlowStep {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Stepper", &self.stepper);
        ui.nested("CS Task", &self.cs_task);
        ui.debug("Flow Mode", self.flow_mode());
        ui.display("Raw Flow Mode", self.flow_mode);
        ui.nested("Move Map List Wrapper", &self.move_map_list_step);
        ui.nested_opt(
            "Move Map List Step",
            self.move_map_list_step()
                .map(|step| unsafe { step.as_ref() }),
        );
    }
}

impl DebugDisplay for CSMoveMapListStep {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Stepper", &self.stepper);
        ui.nested("CS Task", &self.cs_task);
        ui.nested("Game Root Wrapper", &self.game_root_step);
        ui.nested_opt(
            "Game Root Step",
            self.game_root_step().map(|step| unsafe { step.as_ref() }),
        );
    }
}

impl DebugDisplay for GameRootStep {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Stepper", &self.stepper);
        ui.nested("CS Task", &self.cs_task);
        ui.header("Child Steps", || {
            ui.nested("Message", &self.msg_step);
            ui.nested("Dummy", &self.dummy_step);
            ui.nested("Param", &self.param_step);
            ui.nested("Title Wrapper", &self.title_step);
            ui.nested("Regulation", &self.regulation_step);
        });
        ui.nested_opt(
            "Title Step",
            self.title_step().map(|step| unsafe { step.as_ref() }),
        );
    }
}

impl DebugDisplay for TitleStep {
    fn render_debug(&self, ui: &Ui) {
        ui.nested("Stepper", &self.stepper);
    }
}
