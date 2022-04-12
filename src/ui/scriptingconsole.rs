use crate::ui::{UiComponent, ViewComponent};
use egui::{Color32, Context, ScrollArea, TextStyle, Ui};

#[derive(Default)]
pub struct ScriptingConsole {
    pub messages: Vec<String>,
}
impl ViewComponent for ScriptingConsole {
    fn ui(&mut self, ui: &mut Ui) {
        ui.add_space(4.0);
        if ui.button("Clear").clicked() {
            self.messages.clear();
        }
        ui.separator();
        ui.text_style_height(&TextStyle::Body);
        ScrollArea::vertical().stick_to_bottom().show_rows(
            ui,
            2.0,
            self.messages.len(),
            |ui, _row_range| {
                for message in &self.messages {
                    ui.colored_label(Color32::from_rgb(128, 140, 255), message);
                }
                self.messages.clear();
            },
        );
    }
}
impl UiComponent for ScriptingConsole {
    fn name(&self) -> &'static str {
        "Scripting Console"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        egui::Window::new(self.name())
            .resizable(true)
            .collapsible(true)
            .show(ctx, |ui| {
                self.ui(ui);
            });
    }
}
