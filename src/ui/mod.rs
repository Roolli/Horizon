pub mod debugstats;


/// Something that can show some ui
pub trait UiComponent {
    fn name(&self) -> &'static str;

    fn show(&mut self,ctx:&egui::CtxRef,open: &mut bool);
}
/// Something to build Uis with.
pub trait ViewComponent {
    fn ui(&mut self,ui:&mut egui::Ui);
}