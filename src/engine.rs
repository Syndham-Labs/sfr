use std::{
    sync::{Arc, RwLock},
    thread::{JoinHandle, spawn},
};

use crossbeam_channel::{Receiver, Sender, unbounded};
use hades::{HadesEngine, HadesEngineStatus, device::HadesAudioDevice, error::HadesError};

use crate::engine::HadesEngineControllerCommands::{
    ChangeDevice, DisableDSP, EnableDSP, StartEngineRouter, StopEngineRouter,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HadesEngineControllerCommands {
    StartEngineRouter,
    StopEngineRouter,
    DisableDSP,
    EnableDSP,
    ChangeDevice(Option<HadesAudioDevice>, Option<HadesAudioDevice>),
}

#[derive(Debug, Clone)]
pub struct HadesEngineController {
    status: Arc<RwLock<Option<HadesEngineStatus>>>,
    status_rx: Receiver<HadesEngineStatus>,
    command_tx: Sender<HadesEngineControllerCommands>,
}

impl HadesEngineController {
    pub fn new() -> (
        Self,
        Sender<HadesEngineStatus>,
        Receiver<HadesEngineControllerCommands>,
    ) {
        let (status_tx, status_rx) = unbounded::<HadesEngineStatus>();
        let (command_tx, command_rx) = unbounded::<HadesEngineControllerCommands>();

        (
            Self {
                status: Arc::new(RwLock::new(Some(HadesEngineStatus::Stopped))),
                status_rx,
                command_tx,
            },
            status_tx,
            command_rx,
        )
    }

    pub fn spawn_command_watchdog(
        command_rx: Receiver<HadesEngineControllerCommands>,
        mut engine: HadesEngine,
    ) -> JoinHandle<()> {
        spawn(move || {
            let result: Result<(), HadesError> = (|| {
                while let Ok(cmd) = command_rx.try_recv() {
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
        })
    }

    pub fn watch_for_status_change(&mut self) {
        while let Ok(new_status) = self.status_rx.try_recv() {
            let mut guard = self.status.write().unwrap();

            *guard = Some(new_status);
        }
    }

    pub fn get_engine_status(&self) -> Arc<RwLock<Option<HadesEngineStatus>>> {
        self.status.clone()
    }
}
