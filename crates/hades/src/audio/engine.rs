use std::{
    sync::{Arc, atomic::AtomicBool},
    thread::{JoinHandle, spawn},
};

use crossbeam_channel::{Sender, unbounded};

use crate::{
    audio::dsp::{HadesAudioDSPCommands, HadesAudioDigitalSignalProcessingPipeline},
    device::HadesAudioDevice,
    error::HadesError,
    log_err, log_info,
    wasapi::router::HadesAudioRouterBridge,
};

pub struct HadesEngine {
    pub capture_device: HadesAudioDevice,
    pub render_device: HadesAudioDevice,

    pub dsp: HadesAudioDigitalSignalProcessingPipeline,
    pub router_running: Arc<AtomicBool>,

    _dsp_command_tx: Option<Sender<HadesAudioDSPCommands>>,
    _router_handle: Option<JoinHandle<()>>,
}

impl HadesEngine {
    pub fn new(
        capture_device: HadesAudioDevice,
        render_device: HadesAudioDevice,
        dsp: HadesAudioDigitalSignalProcessingPipeline,
    ) -> Self {
        Self {
            capture_device,
            render_device,
            dsp,
            router_running: Arc::new(AtomicBool::new(false)),

            _dsp_command_tx: None,
            _router_handle: None,
        }
    }

    pub fn start_router(&mut self) -> Result<(), HadesError> {
        log_info!("Attempting to start router...");

        self.router_running
            .store(true, std::sync::atomic::Ordering::Release);

        let worker_running = Arc::clone(&self.router_running);
        let worker_capture = self.capture_device.clone();
        let worker_render = self.render_device.clone();

        let mut worker_dsp = self.dsp.clone();

        let (filter_tx, filter_rx) = unbounded::<HadesAudioDSPCommands>();
        self._dsp_command_tx = Some(filter_tx);

        let handle = spawn(move || {
            let result: Result<(), HadesError> = (|| {
                let mut bridge = HadesAudioRouterBridge::new(worker_capture, worker_render)?;

                bridge.init_loopback_capture()?;
                worker_dsp.prepare_channels();

                bridge.bridge_loop(
                    |floats, _, buffer_channels| {
                        while let Ok(cmd) = filter_rx.try_recv() {
                            match cmd {
                                HadesAudioDSPCommands::UpdateFilters(fs) => {
                                    worker_dsp.sync_filters(fs);
                                    worker_dsp.prepare_channels();
                                }
                                HadesAudioDSPCommands::SetEnabled(b) => worker_dsp.set_enabled(b),
                            }
                        }

                        worker_dsp.process_audio(floats, buffer_channels);
                    },
                    worker_running,
                )?;

                Ok(())
            })();

            if let Err(e) = result {
                log_err!("Router Bridge Worker Thread failed: {}", e);
            }
        });

        log_info!("Router started.");
        self._router_handle = Some(handle);

        Ok(())
    }

    pub fn stop_router(&mut self) {
        log_info!("Attempting to stop router...");

        self.router_running
            .store(false, std::sync::atomic::Ordering::SeqCst);

        if let Some(h) = self._router_handle.take() {
            let _ = h.join();
        }

        log_info!("Router stopped.");
    }

    pub fn commit_filter_changes(&self) -> Result<(), HadesError> {
        let cloned = self.dsp.get_filters_cloned();
        let len = cloned.len();

        if let Some(tx) = self._dsp_command_tx.as_ref() {
            tx.try_send(HadesAudioDSPCommands::UpdateFilters(cloned))
                .map_err(|e| HadesError::CrossbeamChannelSendError(e.to_string()))?;
        } else {
            return Err(HadesError::AudioDSPCommandChannelNotInitialized);
        }

        log_info!("Commited {} filters to DSP shadow copy.", len);

        Ok(())
    }

    pub fn set_dsp_enabled_state(&mut self, new: bool) -> Result<(), HadesError> {
        if let Some(tx) = self._dsp_command_tx.as_ref() {
            self.dsp.set_enabled(new);
            tx.try_send(HadesAudioDSPCommands::SetEnabled(new))
                .map_err(|e| HadesError::CrossbeamChannelSendError(e.to_string()))?;
        } else {
            return Err(HadesError::AudioDSPCommandChannelNotInitialized);
        }

        Ok(())
    }
}
