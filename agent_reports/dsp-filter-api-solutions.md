# Hades DSP Filter API: Possible Solutions

## Current difficulty

SFR currently crosses two command boundaries before a filter change reaches
the audio thread:

1. `src/engine.rs` sends `HadesEngineControllerCommands` to the engine
   watchdog.
2. `HadesEngine::commit_filter_changes` sends a cloned `Vec<AudioFilter>` to
   `HadesAudioDSPCommands::UpdateFilters`.

The filter data model also exposes mutation through callbacks:

- `HadesAudioDigitalSignalProcessingPipeline::update_filter` accepts
  `impl FnOnce(&mut AudioFilter, usize)`.
- `ParametricEqualizationFilter::update_filter` accepts
  `impl FnOnce(&BiquadFilter)`, but its callback argument is immutable.

This makes even one edit require application code to know nested enums,
control-side clone/sync behavior, stable IDs, and manual commit/channel
calls. It also makes it possible to mutate SFR state and forget to commit it,
or to clone state that has not been prepared for the router's channel count.

## Ranked solutions

Solutions are ordered from simplest to most involved.

### 1. Add concrete, ID-based DSP methods and remove filter callbacks

Replace callback-based mutation with small methods that describe exactly what
the caller wants. Keep `commit_filter_changes` but hide it behind the concrete
operation so SFR does not have to remember to call it.

```rust
#[derive(Debug, Clone)]
pub enum ParametricFilterUpdate {
    Gain(f32),
    Frequency(u32),
    Q(f32),
    Band(BiquadType),
    Mode(BiquadMode),
    Enabled(bool),
}

impl HadesEngine {
    pub fn add_parametric_filter(
        &mut self,
        filter: ParametricEqualizationFilter,
    ) -> Result<(), HadesError> {
        self.dsp.add_filter(AudioFilter::ParametricEqualizer(filter));
        self.commit_filter_changes()
    }

    pub fn remove_filter(&mut self, id: u32) -> Result<(), HadesError> {
        if self.dsp.remove_filter(id) {
            self.commit_filter_changes()
        } else {
            Ok(())
        }
    }

    pub fn update_biquad(
        &mut self,
        parametric_id: u32,
        biquad_id: u32,
        update: ParametricFilterUpdate,
    ) -> Result<(), HadesError> {
        self.dsp.update_filter(parametric_id, |filter, _sample_rate| {
            if let AudioFilter::ParametricEqualizer(parametric) = filter {
                if let Some(biquad) =
                    parametric.filters.iter_mut().find(|b| b.id == biquad_id)
                {
                    match update {
                        ParametricFilterUpdate::Gain(v) => biquad.set_gain(v),
                        ParametricFilterUpdate::Frequency(v) => biquad.set_freq(v),
                        ParametricFilterUpdate::Q(v) => biquad.set_q(v),
                        ParametricFilterUpdate::Band(v) => biquad.set_band(v),
                        ParametricFilterUpdate::Mode(v) => biquad.set_mode(v),
                        ParametricFilterUpdate::Enabled(v) => biquad.set_enabled(v),
                    }
                }
            }
        });
        self.commit_filter_changes()
    }
}
```

This is the lowest-risk change because it preserves the existing DSP and
channel architecture. It removes nested callback code from SFR and makes
commit behavior consistent. The top-level `HadesEngineController` should
delegate these operations to `HadesEngine` rather than maintaining an
unrelated parallel command API.

### 2. Send typed filter deltas instead of the entire filter vector

Change `HadesAudioDSPCommands` from snapshots to operations. This avoids
cloning and permits the audio thread to synchronize only the requested
state. It should batch multiple operations for UI gestures or preset loads.

```rust
#[derive(Debug, Clone)]
pub enum HadesAudioDSPCommands {
    SetEnabled(bool),
    AddFilter(AudioFilter),
    RemoveFilter(u32),
    UpdateFilter {
        filter_id: u32,
        update: DSPFilterUpdate,
    },
    Batch(Vec<HadesAudioDSPCommands>),
}

#[derive(Debug, Clone)]
pub enum DSPFilterUpdate {
    Parametric {
        enabled: Option<bool>,
        biquad: Option<BiquadUpdate>,
    },
}

#[derive(Debug, Clone)]
pub enum BiquadUpdate {
    Gain { id: u32, value: f32 },
    Frequency { id: u32, value: u32 },
    Q { id: u32, value: f32 },
    Band { id: u32, value: BiquadType },
    Enabled { id: u32, value: bool },
}
```

In the router callback, drain and apply a batch before processing the next
audio block:

```rust
while let Ok(command) = filter_rx.try_recv() {
    worker_dsp.apply_command(command);
}
worker_dsp.process_audio(floats, buffer_channels);
```

The worker should use `sync_filters` for any snapshot/batch merge rather than
`_set_filters`, preserving filter IDs and per-channel state. New biquads
must be prepared for the active channel count, and changed coefficients must
recompute them through `set_*` instead of cloning over running state.

### 3. Expose one high-level DSP command to SFR

Add filter operations directly to `HadesEngineControllerCommands`. This gives
SFR a single API and lets the watchdog translate user intent into an engine
operation. The filter-specific channel in `HadesEngine` can remain internal.

```rust
#[derive(Debug, Clone)]
pub enum HadesEngineControllerCommands {
    StartEngineRouter,
    StopEngineRouter,
    SetDSPEnabled(bool),
    ChangeDevice(
        Option<HadesAudioDevice>,
        Option<HadesAudioDevice>,
    ),
    DSP(DSPFilterCommand),
}

#[derive(Debug, Clone)]
pub enum DSPFilterCommand {
    AddParametric(ParametricEqualizationFilter),
    RemoveFilter(u32),
    UpdateParametricFilter {
        filter_id: u32,
        biquad_id: u32,
        update: ParametricFilterUpdate,
    },
}

impl HadesEngineController {
    pub fn send(&self, command: HadesEngineControllerCommands) {
        let _ = self.command_tx.send(command);
    }
}
```

This hides the fact that Hades needs an engine watchdog and an inner DSP
channel. It is more complete than solution 1, but requires changing the
controller API and enum used by SFR. It is also a good boundary for a future
UI, persistence, or preset layer.

### 4. Publish immutable DSP snapshots for lock-free audio-thread reads

Replace the DSP update channel with an atomic shared snapshot or
copy-on-write handle. SFR mutates a control-side configuration, then publishes
a whole new configuration once.

```rust
use std::sync::Arc;

pub struct HadesEngine {
    pub dsp: HadesAudioDigitalSignalProcessingPipeline,
    dsp_snapshot: Arc<ArcSwap<HadesAudioDSPSnapshot>>,
}

pub struct HadesAudioDSPSnapshot {
    pub enabled: bool,
    pub filters: Vec<AudioFilter>,
}

impl HadesEngine {
    pub fn publish_dsp(
        &self,
        snapshot: HadesAudioDSPSnapshot,
    ) -> Result<(), HadesError> {
        self.dsp_snapshot.store(Arc::new(snapshot));
        Ok(())
    }
}
```

The audio callback loads the newest snapshot and synchronizes it into the
worker-owned DSP instance before processing. This removes send/receive
bookkeeping and is easier to introspect than a queue, but it requires a
snapshot library or an equivalent atomic pointer and does not eliminate the
need for state-preserving synchronization.

### 5. Add a declarative filter preset/builder API

Model intended filter configuration separately from DSP state. A preset can
be built without callbacks and applied as one operation.

```rust
#[derive(Debug, Clone)]
pub struct ParametricPreset {
    pub enabled: bool,
    pub bands: Vec<BiquadPreset>,
}

#[derive(Debug, Clone)]
pub struct BiquadPreset {
    pub id: u32,
    pub gain_db: f32,
    pub frequency_hz: u32,
    pub q: f32,
    pub band: BiquadType,
    pub mode: BiquadMode,
    pub enabled: bool,
}

impl HadesEngine {
    pub fn apply_parametric_preset(
        &mut self,
        filter_id: u32,
        preset: ParametricPreset,
    ) -> Result<(), HadesError> {
        let filter = parametric_from_preset(filter_id, preset);
        self.dsp.replace_or_add_filter(AudioFilter::ParametricEqualizer(filter));
        self.commit_filter_changes()
    }
}
```

This is strongest for preset import/export and UI models, but it is not the
smallest fix: it requires converting presets to runtime filters, defining
validation rules, and deciding whether omitted fields mean defaults or
preserve existing values. It should therefore build on solutions 1 or 2,
not replace them.

## Recommended implementation order

Implement solutions 1 and 2 first:

1. Replace callback mutations with typed parameter updates.
2. Send typed/filter delta commands instead of cloning full vectors.
3. Make the router always synchronize instead of replacing `_set_filters`.
4. Preserve existing Biquad states and initialize only new or resized filters.
5. Optionally add solution 3 once SFR needs a public controller facade.
6. Add presets or snapshot publication only if persistence or lock-free
   configuration becomes an actual requirement.
