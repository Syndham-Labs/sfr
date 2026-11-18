use std::sync::Arc;

use crate::{
    app::root::{GLOBAL_APP_STATE, SFRAppState, SFRFrontend, SFRFrontendOpts},
    asyncrt::TokioMultiThreadRuntime,
    engine::HadesEngineController,
};
use eframe::{self, egui::ViewportBuilder, run_native};
use hades::{
    HadesAudioDigitalSignalProcessingPipeline, HadesEngine, wasapi::utils::HadesAudioDeviceUtils,
};

pub mod app;
pub mod asyncrt;
pub mod engine;
pub mod types;

fn main() -> eframe::Result<(), String> {
    TokioMultiThreadRuntime::configure().map_err(|e| e.to_string())?;

    let device = HadesAudioDeviceUtils::get_default_render_device().map_err(|e| e.to_string())?;
    let sr = HadesAudioDeviceUtils::get_device_sample_rate(&device).map_err(|e| e.to_string())?;
    let nchannels =
        HadesAudioDeviceUtils::get_device_nchannels(&device).map_err(|e| e.to_string())?;

    let dsp = HadesAudioDigitalSignalProcessingPipeline::new(None, sr, nchannels);
    let (controller, status_sender) = HadesEngineController::new();

    let engine = HadesEngine::new(device.clone(), device, dsp, status_sender);

    GLOBAL_APP_STATE
        .set(Arc::new(SFRAppState::new(Arc::new(controller))))
        .map_err(|_| "UNABLE TO INITIALIZE APP STATE".to_string())?;

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
