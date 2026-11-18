# Biquad filter review

## 1. Adding the default biquad is not transparent

`BiquadFilter::new` (and the example in `main.rs`) creates an enabled
`BandPass` filter. A band-pass filter necessarily attenuates frequencies
outside its pass band, so adding it will audibly reduce the signal even when
`gain` is `0.0`. The band-pass cookbook coefficients used here are the
constant-skirt-gain form and are valid; this is an expected response, not an
interleaving-channel bug.

```rust
// crates/hades/src/audio/filter/parametric/biquad.rs
band: BiquadType::BandPass,
enabled: true,
```

```rust
// crates/hades/src/main.rs
pef.new_raw_biquad_filter(
    0.4,
    1800,
    0.7,
    BiquadType::BandPass,
    ...
);
```

**Recommended solution:** require the caller to choose the filter type, or
make a newly created filter disabled until configured. For a transparent
default use a `Peaking` filter with `gain = 0.0`; its cookbook transfer
function is unity at every frequency. Also expose/use the intended
`Peaking`, low-pass, high-pass, or shelf type rather than treating `gain` as a
generic volume control.

## 2. Filter commits reset the state of existing filters

Every `UpdateFilters` command replaces the worker’s filter list and then calls
`prepare_channels`. `prepare_channels` unconditionally replaces each
biquad’s per-channel state with zeros. Therefore adding one filter also
restarts all previously running filters, causing a discontinuity/click and a
short transient. This is especially noticeable for resonant filters.

```rust
// crates/hades/src/audio/engine.rs
worker_dsp._set_filters(fs);
worker_dsp.prepare_channels(channels);
```

```rust
// crates/hades/src/audio/filter/parametric/biquad.rs
self.states = vec![BiquadState::default(); channels];
```

**Recommended solution:** preserve state for unchanged filters when applying
an update (match by stable filter ID), initialize only newly added filters,
and smooth coefficient changes or crossfade the old and new filter chains over
a short block interval. At minimum, do not reset existing states when only a
new filter is appended.

## 3. Capture and render formats are assumed to match

`bridge_loop` derives `block_align`, channel count, and DSP buffer sizing from
the capture format, then uses those values for render buffers and available
render bytes. It separately encodes with the render subformat. If the devices
have different channel counts, sample sizes, or sample rates, the output can
be truncated, mis-sized, or encoded with the wrong sample mapping. The DSP
also uses the capture sample rate for its coefficients while the callback
ignores the supplied rate.

```rust
let (block_align, max_bytes, mut capture_scratch_buffer) =
    self.init_scratch_buffer(buffer_size, n_channels, bits_per_sample);
let (_, _, mut render_scratch_buffer) =
    self.init_scratch_buffer(buffer_size, n_channels, bits_per_sample);
```

**Recommended solution:** validate that capture and render formats are
compatible before starting, or maintain separate capture/render frame sizes,
block alignments, channel counts, and sample-rate conversion. Construct the
DSP with the actual stream rate it processes and reject unsupported format
mismatches explicitly.

## Cookbook check

The current coefficient implementation uses `A = 10^(dBgain/40)` and the
published peaking/shelf equations. The `alpha` and normalization steps also
match the 2021 W3C Audio EQ Cookbook. No coefficient or interleaved-state
ordering error was found in the reviewed implementation.
