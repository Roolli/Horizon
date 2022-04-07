use crate::ui::{UiComponent, ViewComponent};
use egui::{Context, Ui};

#[derive(Default)]
pub struct Menu {
    pub window_should_close: bool,
    pub show_debug_window: bool,
    pub show_scripting_console: bool,
}

impl ViewComponent for Menu {
    fn ui(&mut self, ui: &mut Ui) {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Exit").clicked() {
                    self.window_should_close = true;
                }
            });
            ui.menu_button("View", |ui| {
                ui.checkbox(&mut self.show_scripting_console, "Show Scripting console");
                ui.checkbox(&mut self.show_debug_window, "Show debug window");
            });
        });
    }
}
impl UiComponent for Menu {
    fn name(&self) -> &'static str {
        "Main Menu"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        egui::TopBottomPanel::top(self.name()).show(ctx, |ui| {
            self.ui(ui);
        });
    }
}
