use std::sync::Arc;

use eframe::egui::{self, ViewportCommand};

use crate::app::root::{SFRAppState, UIComponentMut, global_state};

pub struct UITopbar {
    _state: &'static Arc<SFRAppState>,
    _app_version: String,
}

impl UITopbar {
    pub fn new(app_version: String) -> Self {
        Self {
            _state: global_state(),
            _app_version: app_version,
        }
    }
}

impl UIComponentMut for UITopbar {
    fn draw_mut(&mut self, ui: &mut eframe::egui::Ui, ctx: &eframe::egui::Context) {
        egui::Panel::top("top_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("SFR v{}", self._app_version));
                ui.separator();
                egui::MenuBar::default().ui(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Settings").clicked() {
                            self._state.toggle_settings_window();
                        }
                        ui.separator();
                        if ui.button("Exit").clicked() {
                            ctx.send_viewport_cmd(ViewportCommand::Close);
                        }
                    });
                });
            })
        });
    }
}
