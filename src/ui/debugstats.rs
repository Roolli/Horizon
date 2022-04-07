use crate::ui::{UiComponent, ViewComponent};
use crate::TextureViewTypes;
use egui::{Context, TextureId, Ui};
use rapier3d::na::Point3;
use specs::Entity;

pub struct DebugStats {
    pub fps: u16,
    pub unique_model_count: u32,
    pub debug_texture: Option<wgpu::Texture>,
    pub debug_texture_view: Option<wgpu::TextureView>,
    pub cam_pos: Point3<f32>,
    pub cam_yaw_pitch: (f32, f32),
    pub texture_id: Option<TextureId>,
    pub selected_texture_name: TextureViewTypes,
    pub debug_texture_renderer: Option<(wgpu::BindGroup, wgpu::RenderPipeline)>,
    pub selected_entity: Option<Entity>,
    pub selected_material: usize,
    pub selected_texture: usize,
    pub show_collision_wireframes: bool,
}

impl UiComponent for DebugStats {
    fn name(&self) -> &'static str {
        "Debug Window"
    }

    fn show(&mut self, ctx: &Context, open: &mut bool) {
        egui::Window::new(self.name())
            .collapsible(true)
            .resizable(true)
            .show(ctx, |ui| {
                use super::ViewComponent as _;
                self.ui(ui);
            });
    }
}

impl ViewComponent for DebugStats {
    fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(format!("FPS: {}", self.fps));
                ui.label(format!(
                    "Camera pos x:{:.1} y:{:.1} z:{:.1}",
                    self.cam_pos.x, self.cam_pos.y, self.cam_pos.z
                ));
                ui.label(format!(
                    "Camera yaw: {:.1}, pitch: {:.1}",
                    self.cam_yaw_pitch.0.to_degrees() % 360.0,
                    self.cam_yaw_pitch.1.to_degrees() % 360.0
                ));
                ui.end_row();
                ui.checkbox(&mut self.show_collision_wireframes, "Show collisions");
                ui.separator();
                egui::ComboBox::from_label("Select debug texture!")
                    .selected_text(format!("{:?}", &self.selected_texture_name))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.selected_texture_name,
                            TextureViewTypes::DeferredPosition,
                            "Deferred Position Texture View",
                        );
                        ui.selectable_value(
                            &mut self.selected_texture_name,
                            TextureViewTypes::DeferredNormals,
                            "Deferred Normals Texture View",
                        );
                        ui.selectable_value(
                            &mut self.selected_texture_name,
                            TextureViewTypes::DeferredAlbedo,
                            "Deferred Albedo Texture View",
                        );
                        ui.selectable_value(
                            &mut self.selected_texture_name,
                            TextureViewTypes::DeferredSpecular,
                            "Deferred Specular Texture View",
                        );
                        ui.selectable_value(
                            &mut self.selected_texture_name,
                            TextureViewTypes::Shadow,
                            "Shadow texture View",
                        );
                    });
                if let Some(tex_id) = self.texture_id {
                    ui.image(tex_id, egui::Vec2::new(480.0, 320.0));
                }
                ui.end_row();
            });
        });
    }
}
