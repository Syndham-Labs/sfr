use std::sync::{
    Arc, OnceLock, RwLock,
    atomic::{AtomicBool, Ordering},
};

use eframe::{
    App,
    egui::{self, FontData, FontDefinitions, FontFamily, FontTweak, ViewportCommand},
};

use crate::app::{SettingsWindow, UITopbar};

// * App State and UI Definitions

pub trait UIComponent {
    fn draw(&self, ui: &mut eframe::egui::Ui, ctx: &eframe::egui::Context) -> ();
}
pub trait UIComponentMut {
    fn draw_mut(&mut self, ui: &mut eframe::egui::Ui, ctx: &eframe::egui::Context) -> ();
}

pub struct SFRAppState {
    pub show_settings_window: AtomicBool,
    pub error_message: RwLock<Option<Arc<String>>>,
}

impl SFRAppState {
    pub fn is_open_settings_window(&self) -> bool {
        self.show_settings_window.load(Ordering::Relaxed)
    }

    pub fn toggle_settings_window(&self) {
        self.show_settings_window.fetch_xor(true, Ordering::Relaxed);
    }
}

impl Default for SFRAppState {
    fn default() -> Self {
        Self {
            show_settings_window: AtomicBool::new(false),
            error_message: RwLock::new(None),
        }
    }
}

pub static GLOBAL_APP_STATE: OnceLock<Arc<SFRAppState>> = OnceLock::new();

pub fn global_state() -> &'static Arc<SFRAppState> {
    GLOBAL_APP_STATE.get_or_init(|| Arc::new(SFRAppState::default()))
}

// * Root App Component

pub struct SFRFrontend {
    pub name: String,
    pub first_frame: bool,

    _state: &'static Arc<SFRAppState>,
    _app_version: String,
}

pub struct SFRFrontendOpts {
    pub app_version: String,
}

impl SFRFrontendOpts {
    pub fn new(app_version: String) -> Self {
        SFRFrontendOpts { app_version }
    }
}

impl SFRFrontend {
    pub fn new(opts: SFRFrontendOpts) -> Self {
        Self {
            name: "Syndham Sound Frequency Rectification".to_string(),
            first_frame: true,

            _state: global_state(),
            _app_version: opts.app_version,
        }
    }

    fn replace_fonts(&self, ctx: &eframe::egui::Context) {
        let mut fonts = FontDefinitions::default();

        fonts.font_data.insert(
            "apple_sf_pro_text_regular".to_string(),
            Arc::new(
                FontData::from_static(include_bytes!(
                    "../../fonts/AppleSF/SF-Pro-Text-Regular.otf"
                ))
                .tweak(FontTweak {
                    y_offset_factor: 0.0,
                    scale: 1.0,
                    ..Default::default()
                }),
            ),
        );

        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "apple_sf_pro_text_regular".to_string());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("apple_sf_pro_text_regular".to_string());

        ctx.set_fonts(fonts);
    }

    fn setup_window(&mut self, ctx: &eframe::egui::Context) {
        egui_material_icons::initialize(ctx);
        self.replace_fonts(ctx);

        if self.first_frame {
            ctx.send_viewport_cmd(ViewportCommand::Maximized(true));
            ctx.request_repaint();
            self.first_frame = false;
        }
    }

    fn render_warning_banner(&mut self, ui: &mut eframe::egui::Ui) {
        if let Some(err_guard) = &self._state.error_message.read().ok() {
            let err = (*err_guard).clone();

            if let Some(msg) = err {
                let bg_colour = egui::Color32::from_rgb(254, 243, 199);
                let text_colour = egui::Color32::from_rgb(146, 64, 14);

                egui::Panel::top("warning_banner")
                    .frame(
                        egui::Frame::new()
                            .fill(bg_colour)
                            .inner_margin(egui::Margin::symmetric(12, 6))
                            .stroke(egui::Stroke::new(
                                1.0,
                                egui::Color32::from_rgb(252, 211, 77),
                            )),
                    )
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Warning:").color(text_colour).strong());
                            ui.label(
                                egui::RichText::new(msg.to_string())
                                    .color(text_colour)
                                    .text_style(egui::TextStyle::Body),
                            );

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui
                                        .button(
                                            egui::RichText::new(
                                                egui_material_icons::icons::ICON_CLOSE,
                                            )
                                            .color(text_colour)
                                            .size(12.0),
                                        )
                                        .clicked()
                                    {
                                        *self._state.error_message.write().unwrap() = None;
                                    }
                                },
                            );
                        });
                    });
            }
        }
    }
}

impl App for SFRFrontend {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, frame: &mut eframe::Frame) {
        let ctx = &ui.ctx().clone();

        self.setup_window(ctx);

        self.render_warning_banner(ui);

        UITopbar::new(self._app_version.clone()).draw_mut(ui, ctx);

        egui::CentralPanel::default().show(ui, |_| {});

        if self._state.is_open_settings_window() {
            SettingsWindow::new().draw_mut(ui, ctx);
        }
    }
}
