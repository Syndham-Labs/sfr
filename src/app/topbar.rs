use std::sync::Arc;

use eframe::egui::{self, ViewportCommand};
use hades::HadesEngineStatus;

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
        let status_result = self._state.engine_controller.get_engine_status();
        let status = match status_result.read() {
            Ok(status_guard) => status_guard.clone().unwrap_or(HadesEngineStatus::Unknown),
            Err(_) => {
                *self._state.error_message.write().unwrap() = Some(Arc::new(
                    "Unable to retrieve engine status, defaulting to Unknown".to_string(),
                ));
                HadesEngineStatus::Unknown
            }
        };

        egui::Panel::top("top_panel").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(format!("SFR v{}", self._app_version));
                ui.separator();
                ui.label(format!("Engine: {}", status));
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
