use crate::{
    app::root::{SFRFrontend, SFRFrontendOpts},
    asyncrt::TokioMultiThreadRuntime,
};
use eframe::{self, egui::ViewportBuilder, run_native};

pub mod app;
pub mod asyncrt;
pub mod types;

fn main() -> eframe::Result<(), String> {
    TokioMultiThreadRuntime::configure().map_err(|e| e.to_string())?;

    let app_version = env!("CARGO_PKG_VERSION");
    let ui_opts = SFRFrontendOpts::new(app_version.to_string());

    let eframe_native_opts = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([400.0, 200.0]),
        ..Default::default()
    };

    let _ = run_native(
        format!("Syndham Sound Frequency Rectification v{}", app_version).as_str(),
        eframe_native_opts,
        Box::new(|_cc| Ok(Box::new(SFRFrontend::new(ui_opts)))),
    );

    Ok(())
}
