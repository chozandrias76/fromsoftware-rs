use eldenring::cs::MsgRepositoryImp;
use hudhook::imgui::Ui;

use super::DebugDisplay;

impl DebugDisplay for MsgRepositoryImp {
    fn render_debug(&self, ui: &Ui) {
        ui.text("Known layout: vtable prefix only; message tables are not modeled yet.");
    }
}
