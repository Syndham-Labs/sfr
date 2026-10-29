use std::sync::Arc;

use eframe::egui;

use crate::app::root::{SFRAppState, UIComponentMut, global_state};

pub struct SettingsWindow {
    _state: &'static Arc<SFRAppState>,
}

impl SettingsWindow {
    pub fn new() -> Self {
        Self {
            _state: global_state(),
        }
    }
}

impl UIComponentMut for SettingsWindow {
    fn draw_mut(&mut self, ui: &mut eframe::egui::Ui, ctx: &eframe::egui::Context) -> () {
        let mut open = true;

        egui::Window::new("Settings")
            .open(&mut open)
            .default_size([300.0, 200.0])
            .default_pos([200.0, 300.0])
            .show(ctx, |ui| {
                ui.label("Audio Routing");
                ui.horizontal(|ui| {
                    ui.label("Physical Output: ");
                    egui::containers::ComboBox::from_label("Select device").show_ui(ui, |ui| {})
                });
                ui.separator();
            });

        if !open {
            self._state.toggle_settings_window();
        }
    }
}
