use std::{
    sync::{Arc, RwLock},
    thread::{JoinHandle, spawn},
};

use crossbeam_channel::{Receiver, Sender, unbounded};
use hades::{
    HadesAudioDigitalSignalProcessingPipeline, HadesEngine, HadesEngineStatus,
    audio::filter::parametric::{ParametricEqualizationFilter, ParametricFilterUpdate},
    device::HadesAudioDevice,
    error::HadesError,
    wasapi::utils::HadesAudioDeviceUtils,
};

use crate::engine::HadesEngineControllerCommands::{
    AddParametricFilter, ChangeDevice, DisableDSP, EnableDSP, RemoveFilter, StartEngineRouter,
    StopEngineRouter, UpdateBiquad,
};

#[derive(Debug, Clone, PartialEq)]
pub enum HadesEngineControllerCommands {
    StartEngineRouter,
    StopEngineRouter,
    DisableDSP,
    EnableDSP,
    ChangeDevice(Option<HadesAudioDevice>, Option<HadesAudioDevice>),
    AddParametricFilter(ParametricEqualizationFilter),
    RemoveFilter(u32),
    UpdateBiquad {
        parametric_id: u32,
        biquad_id: u32,
        update: ParametricFilterUpdate,
    },
}

#[derive(Debug)]
pub struct HadesEngineController {
    status: Arc<RwLock<Option<HadesEngineStatus>>>,
    command_tx: Sender<HadesEngineControllerCommands>,

    _engine_handle: Option<JoinHandle<()>>,
    _status_handle: Option<JoinHandle<()>>,
}

impl HadesEngineController {
    pub fn new() -> (
        Self,
        Sender<HadesEngineStatus>,
        Receiver<HadesEngineControllerCommands>,
        Receiver<HadesEngineStatus>,
    ) {
        let (status_tx, status_rx) = unbounded::<HadesEngineStatus>();
        let (command_tx, command_rx) = unbounded::<HadesEngineControllerCommands>();

        (
            Self {
                status: Arc::new(RwLock::new(Some(HadesEngineStatus::Stopped))),
                command_tx,
                _engine_handle: None,
                _status_handle: None,
            },
            status_tx,
            command_rx,
            status_rx,
        )
    }

    pub fn initialize_engine_and_watchdog(
        &mut self,
        command_rx: Receiver<HadesEngineControllerCommands>,
        status_tx: Sender<HadesEngineStatus>,
    ) {
        let handle = spawn(move || {
            let result: Result<(), HadesError> = (|| {
                HadesEngine::initialize()?;

                let device = HadesAudioDeviceUtils::get_default_render_device()?;
                let sr = HadesAudioDeviceUtils::get_device_sample_rate(&device)?;
                let nchannels = HadesAudioDeviceUtils::get_device_nchannels(&device)?;

                let dsp = HadesAudioDigitalSignalProcessingPipeline::new(None, sr, nchannels);

                let mut engine = HadesEngine::new(device.clone(), device, dsp, status_tx);

                while let Ok(cmd) = command_rx.recv() {
                    match cmd {
                        StartEngineRouter => {
                            engine.start_router()?;
                            engine.commit_filter_changes()?;
                        }
                        StopEngineRouter => {
                            engine.stop_router()?;
                        }
                        DisableDSP => {
                            engine.set_dsp_enabled_state(false)?;
                        }
                        EnableDSP => {
                            engine.set_dsp_enabled_state(true)?;
                        }
                        ChangeDevice(capture, render) => {
                            engine.change_device(capture, render)?;
                        }
                        AddParametricFilter(filter) => {
                            engine.add_parametric_filter(filter)?;
                        }
                        RemoveFilter(id) => {
                            engine.remove_filter(id)?;
                        }
                        UpdateBiquad {
                            parametric_id,
                            biquad_id,
                            update,
                        } => {
                            engine.update_biquad(parametric_id, biquad_id, update)?;
                        }
                    }
                }

                Ok(())
            })();

            if let Err(e) = result {
                eprintln!(
                    "An error occurred with Hades command watchdog: {}",
                    e.to_string()
                )
            }
        });

        self._engine_handle = Some(handle);
    }

    pub fn send(
        &self,
        command: HadesEngineControllerCommands,
    ) -> Result<(), crossbeam_channel::SendError<HadesEngineControllerCommands>> {
        self.command_tx.send(command)
    }

    pub fn watch_for_status_change(&mut self, status_rx: Receiver<HadesEngineStatus>) {
        let status = self.status.clone();
        let handle = spawn(move || {
            while let Ok(new_status) = status_rx.recv() {
                *status.write().unwrap() = Some(new_status);
            }
        });

        self._status_handle = Some(handle);
    }

    pub fn get_engine_status(&self) -> Arc<RwLock<Option<HadesEngineStatus>>> {
        self.status.clone()
    }
}
