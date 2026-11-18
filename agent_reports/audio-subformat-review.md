# Audio subformat review

## 1. Capture format is reused for rendering

`bridge_loop` derives `block_align`, channel count, and `AudioSubformat` from the capture format, then encodes those samples into the render buffer:

```rust
let (buffer_size, n_channels, bits_per_sample, wfm) = { /* capture client */ };
let audio_subformat = AudioSubformat::from_wave_format(&wfm)?;
// ...
let output_slice = &mut render_scratch_buffer[..n_bytes];
audio_subformat.encode_from_f32(output_floats, output_slice);
```

The render client is initialized from its own mix format in `init_loopback_capture`. If the devices differ in channels, sample size, or subformat, the render data has the wrong layout and may be truncated or misinterpreted.

**Recommended solution:** retain the render `WaveFormat`, derive separate capture/render layouts and subformats, and convert the callback output using the render layout.

> Developer Response [2026-10-29@1400h]: Instantiated a separate `render_subformat`, used `render_subformat` for conversion from `f32` back to `u8`

## 2. Float decoding silently drops unaligned bytes

```rust
let (_prefix, floats, _suffix) = pod_align_to::<u8, f32>(raw_bytes);
let len = floats.len().min(float_scratch.len());
float_scratch[..len].copy_from_slice(&floats[..len]);
```

`pod_align_to` can return a non-empty prefix or suffix, but both are ignored. This can leave samples missing or stale without an error.

**Recommended solution:** validate that the input is properly aligned and a multiple of `size_of::<f32>()`, or decode each 4-byte sample explicitly with `f32::from_le_bytes`.

> Developer Response [2026-10-29@1407h]: `HadesAudioRouterBridge` has multiple checks to ensure input and output slices are of equal lengths. Samples that would be dropped would occur quite rarely due to the usage of two separate ring buffers to ensure clock synchronization. Can be disregarded.
