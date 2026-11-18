# Filter synchronization review

## 1. Hades replaces the filter vector instead of synchronizing it

`ParametricEqualizationFilter::sync_filters` is never called by the DSP
pipeline. The router receives `UpdateFilters`, replaces the entire
`HadesAudioDigitalSignalProcessingPipeline::filters` vector, and then
reinitializes every filter's channel state:

```rust
// crates/hades/src/audio/engine.rs:71-74
HadesAudioDSPCommands::UpdateFilters(fs) => {
    worker_dsp._set_filters(fs);
    worker_dsp.prepare_channels(channels);
}
```

```rust
// crates/hades/src/audio/dsp.rs:69-71
pub fn _set_filters(&mut self, filters: Vec<AudioFilter>) {
    self.filters = filters;
}
```

This defeats `BiquadFilter::parameters_equal`, `BiquadFilter::update_from`,
and `ParametricEqualizationFilter::sync_filters`. It also discards the
running state of unchanged filters; the subsequent `prepare_channels` call
replaces every `states` vector with zeroed state.

**Recommended solution:** add a synchronization operation to the DSP
pipeline that matches incoming `AudioFilter` values by their stable IDs and
dispatches the `ParametricEqualizer` variant to
`ParametricEqualizationFilter::sync_filters`. Preserve existing filter
instances and only initialize state for newly added filters. Replace
`_set_filters` in the `UpdateFilters` handler with that operation, retaining
full replacement only as an explicitly named initialization/reset operation.

## 2. New filters can be initialized with zero channel state by `sync_filters`

When `sync_filters` adds a filter, it clones the incoming filter and calls
`prepare_channels` using the clone's current state length:

```rust
// crates/hades/src/audio/filter/parametric.rs:97-101
let mut new_filter = f1.clone();
new_filter.prepare_channels(new_filter.states.len());
self.filters.push(new_filter);
```

The control-side filters cloned by `commit_filter_changes` commonly have not
been prepared for the router's channel count, so `states.len()` can be zero.
The newly synchronized filter would then have no per-channel state and
`process` can index out of bounds when audio is processed.

**Recommended solution:** pass the active channel count into
`sync_filters` (or the DSP-level synchronization method) and call
`new_filter.prepare_channels(channels)`. Validate that the incoming filter
configuration is compatible with the active sample rate/channels before
publishing it.

## 3. `BiquadFilter::update_from` currently prevents compilation

The method contains a second nested declaration of itself:

```rust
// crates/hades/src/audio/filter/parametric/biquad.rs:269-271
pub fn update_from(&mut self, other: &BiquadFilter) {
    pub fn update_from(&mut self, other: &BiquadFilter) {
```

`cargo check -p hades` fails with "`self` parameter is only allowed in
associated functions" at the nested declaration. Consequently the
synchronization helper cannot currently be compiled or exercised.

**Recommended solution:** remove the inner duplicate declaration and retain
one method body under the `impl BiquadFilter` block, then add a focused test
covering coefficient updates while preserving `states`.
