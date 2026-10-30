use std::io::stdin;

use hades::{
    audio::{
        dsp::HadesAudioDigitalSignalProcessingPipeline,
        engine::HadesEngine,
        filter::{
            AudioFilter::ParametricEqualizer,
            parametric::{
                ParametricEqualizationFilter,
                biquad::{BiquadMode, BiquadPan, BiquadType},
            },
        },
    },
    error::HadesError,
    wasapi::utils::HadesAudioDeviceUtils,
};
use wasapi::initialize_mta;

fn loopback_test() -> Result<(), HadesError> {
    let _ = initialize_mta();

    let capture_device = HadesAudioDeviceUtils::find_capture_device_by_partial_friendlyname(
        "CABLE Output".to_string(),
    )?;
    let render_device = HadesAudioDeviceUtils::find_render_device_by_partial_friendlyname(
        "Bluebass Elite Hybrid".to_string(),
    )?;

    println!(
        "Found Capture Device: {} - {}",
        capture_device.get_name(),
        capture_device.get_desc()
    );
    println!(
        "Found Render Device: {} - {}",
        render_device.get_name(),
        render_device.get_desc()
    );

    let cap_sr = HadesAudioDeviceUtils::get_device_sample_rate(&capture_device)?;
    let cap_nchannels = HadesAudioDeviceUtils::get_device_nchannels(&capture_device)?;

    let mut engine = HadesEngine::new(
        capture_device,
        render_device,
        HadesAudioDigitalSignalProcessingPipeline::new(None, cap_sr, cap_nchannels),
    );

    engine.start_router()?;
    engine.commit_filter_changes()?;

    engine
        .dsp
        .add_filter(ParametricEqualizer(ParametricEqualizationFilter::new(
            244,
            None,
            cap_nchannels,
            true,
        )));
    engine.dsp.update_filter(244, |f, sr| match f {
        ParametricEqualizer(pef) => {
            pef.new_raw_biquad_filter(
                3.5,
                35,
                0.5,
                BiquadType::FlatTopBell,
                BiquadPan::Centre,
                BiquadMode::TransposedDirectFormII,
                sr,
                true,
            );
        }
    });
    engine.commit_filter_changes()?;

    let mut input = String::new();

    stdin().read_line(&mut input)?;

    println!("Input received, stopping bridge...");

    engine.stop_router();

    Ok(())
}

fn main() -> Result<(), HadesError> {
    loopback_test()?;

    Ok(())
}
