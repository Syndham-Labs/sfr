use std::{
    cmp::min,
    io::stdin,
    sync::{Arc, atomic::AtomicBool},
    thread::spawn,
};

use hades::{
    error::HadesError,
    wasapi::{router::HadesAudioRouterBridge, utils::HadesAudioDeviceUtils},
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

    let running = Arc::new(AtomicBool::new(true));
    let worker_running = Arc::clone(&running);

    let handle = spawn(move || {
        let result: Result<(), HadesError> = (|| {
            let mut bridge = HadesAudioRouterBridge::new(capture_device, render_device)?;

            bridge.init_loopback_capture()?;

            bridge.bridge_loop(
                |input, output| {
                    let len = min(input.len(), output.len());
                    output[..len].copy_from_slice(&input[..len]);
                },
                worker_running,
            )?;

            Ok(())
        })();

        if let Err(e) = result {
            eprintln!("Loopback Worker Thread failed: {}", e);
        }
    });

    println!("Loopback bridge running on capture device");

    let mut input = String::new();

    stdin().read_line(&mut input)?;

    println!("Input received, stopping bridge...");

    running.store(false, std::sync::atomic::Ordering::SeqCst);
    let _ = handle.join();

    Ok(())
}

fn main() -> Result<(), HadesError> {
    loopback_test()?;

    Ok(())
}
