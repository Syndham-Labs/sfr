use std::{
    sync::{Arc, atomic::AtomicBool},
    thread::{JoinHandle, spawn},
};

use crossbeam_channel::{Receiver, Sender, unbounded};
use wasapi::initialize_mta;

use crate::{
    audio::dsp::{HadesAudioDSPCommands, HadesAudioDigitalSignalProcessingPipeline},
    device::HadesAudioDevice,
    error::HadesError,
    log_err, log_info,
    wasapi::{router::HadesAudioRouterBridge, utils::HadesAudioDeviceUtils},
};

#[derive(Debug)]
pub enum HadesEngineStatus {
    Starting,
    Stopped,
    Running,
    Failed(HadesError),
}

pub struct HadesEngine {
    pub capture_device: HadesAudioDevice,
    pub render_device: HadesAudioDevice,

    pub dsp: HadesAudioDigitalSignalProcessingPipeline,

    _router_running: Arc<AtomicBool>,
    _dsp_command_tx: Option<Sender<HadesAudioDSPCommands>>,
    _router_handle: Option<JoinHandle<()>>,
    _wasapi_com_mta_init: bool,
    _status_sender: Sender<HadesEngineStatus>,
}

impl HadesEngine {
    pub fn new(
        capture_device: HadesAudioDevice,
        render_device: HadesAudioDevice,
        dsp: HadesAudioDigitalSignalProcessingPipeline,
        sender: Sender<HadesEngineStatus>,
    ) -> Self {
        Self {
            capture_device,
            render_device,
            dsp,
            _router_running: Arc::new(AtomicBool::new(false)),

            _dsp_command_tx: None,
            _router_handle: None,
            _wasapi_com_mta_init: false,
            _status_sender: sender,
        }
    }

    pub fn initialize(&mut self) -> Result<(), HadesError> {
        let result = initialize_mta();

        if result.is_err() {
            return Err(HadesError::Win32APIError(result.0, result.message()));
        }

        Ok(())
    }

    pub fn get_router_status(&self) -> bool {
        self._router_running
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn start_router(&mut self) -> Result<(), HadesError> {
        if !self._wasapi_com_mta_init {
            return Err(HadesError::Win32COMWASAPINotInitialized);
        }

        log_info!("Attempting to start router...");

        self._status_sender
            .try_send(HadesEngineStatus::Starting)
            .map_err(|e| HadesError::CrossbeamChannelSendError(e.to_string()))?;

        let worker_running = Arc::clone(&self._router_running);
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
        self._router_running
            .store(true, std::sync::atomic::Ordering::Release);
        self._router_handle = Some(handle);
        self._status_sender
            .try_send(HadesEngineStatus::Running)
            .map_err(|e| HadesError::CrossbeamChannelSendError(e.to_string()))?;

        Ok(())
    }

    pub fn stop_router(&mut self) -> Result<(), HadesError> {
        log_info!("Attempting to stop router...");

        if let Some(h) = self._router_handle.take() {
            let _ = h.join();
        }

        self._router_running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self._status_sender
            .try_send(HadesEngineStatus::Stopped)
            .map_err(|e| HadesError::CrossbeamChannelSendError(e.to_string()))?;

        log_info!("Router stopped.");

        Ok(())
    }

    pub fn commit_filter_changes(&self) -> Result<(), HadesError> {
        let cloned = self.dsp.get_filters_cloned();

        if let Some(tx) = self._dsp_command_tx.as_ref() {
            tx.try_send(HadesAudioDSPCommands::UpdateFilters(cloned))
                .map_err(|e| HadesError::CrossbeamChannelSendError(e.to_string()))?;
        } else {
            return Err(HadesError::AudioDSPCommandChannelNotInitialized);
        }

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

    pub fn change_device(
        &mut self,
        new_capture: Option<HadesAudioDevice>,
        new_render: Option<HadesAudioDevice>,
    ) -> Result<(), HadesError> {
        let capture_to_set = match new_capture {
            Some(d) => d,
            None => self.capture_device.clone(),
        };
        let render_to_set = match new_render {
            Some(d) => d,
            None => self.render_device.clone(),
        };

        if self.get_router_status() {
            self.stop_router()?;
        }

        let sr = HadesAudioDeviceUtils::get_device_sample_rate(&capture_to_set)?;
        let nchannels = HadesAudioDeviceUtils::get_device_nchannels(&capture_to_set)?;

        let dsp = HadesAudioDigitalSignalProcessingPipeline::new(
            Some(self.dsp.filters.clone()),
            sr,
            nchannels,
        );

        self.dsp = dsp;
        self.capture_device = capture_to_set;
        self.render_device = render_to_set;

        self.start_router()?;
        self.commit_filter_changes()?;

        Ok(())
    }
}
