use std::sync::{Arc, atomic::AtomicBool};

use rtrb::RingBuffer;
use wasapi::{AudioCaptureClient, AudioClient, AudioRenderClient, StreamMode};

use crate::{device::HadesAudioDevice, error::HadesError};

pub struct HadesAudioRouterBridge {
    pub capture_device: HadesAudioDevice,
    pub render_device: HadesAudioDevice,

    _capture_audio_client: Option<AudioClient>,
    _capture_audio_cap: Option<AudioCaptureClient>,

    _render_audio_client: Option<AudioClient>,
    _render_audio_rend: Option<AudioRenderClient>,
}

impl HadesAudioRouterBridge {
    pub fn new(
        capture_device: HadesAudioDevice,
        render_device: HadesAudioDevice,
    ) -> Result<Self, HadesError> {
        Ok(Self {
            capture_device,
            render_device,

            _capture_audio_client: None,
            _capture_audio_cap: None,

            _render_audio_client: None,
            _render_audio_rend: None,
        })
    }

    pub fn set_capture_audio_client(&mut self, c: Option<AudioClient>) {
        self._capture_audio_client = c;
    }
    pub fn set_capture_audio_cap(&mut self, c: Option<AudioCaptureClient>) {
        self._capture_audio_cap = c;
    }
    pub fn set_render_audio_client(&mut self, c: Option<AudioClient>) {
        self._render_audio_client = c;
    }
    pub fn set_render_audio_rend(&mut self, c: Option<AudioRenderClient>) {
        self._render_audio_rend = c;
    }

    pub fn init_scratch_buffer(
        &self,
        buffer_size: usize,
        n_channels: usize,
        bits_per_sample: usize,
    ) -> (usize, usize, Vec<u8>) {
        let block_align = n_channels * (bits_per_sample / 8);
        let max_bytes = buffer_size * block_align;

        (block_align, max_bytes, vec![0u8; max_bytes])
    }

    pub fn init_loopback_capture(&mut self) -> Result<(), HadesError> {
        let mut capture_client = self.capture_device.to_wasapi_device()?.get_iaudioclient()?;
        let mut render_client = self.render_device.to_wasapi_device()?.get_iaudioclient()?;

        let capture_wave_format = capture_client.get_mixformat()?;
        let (capture_def_period, _capture_min) = capture_client.get_device_period()?;

        let render_wave_format = render_client.get_mixformat()?;
        let (render_def_period, _render_min) = render_client.get_device_period()?;

        capture_client.initialize_client(
            &capture_wave_format,
            &wasapi::Direction::Capture,
            &StreamMode::PollingShared {
                autoconvert: false,
                buffer_duration_hns: capture_def_period,
            },
        )?;

        render_client.initialize_client(
            &render_wave_format,
            &wasapi::Direction::Render,
            &StreamMode::PollingShared {
                autoconvert: false,
                buffer_duration_hns: render_def_period,
            },
        )?;

        let capture_cap = capture_client.get_audiocaptureclient()?;
        let render_rend = render_client.get_audiorenderclient()?;

        capture_client.start_stream()?;

        let prefill = render_client.get_available_space_in_frames()? as usize;
        let (block_align, _, silence) = self.init_scratch_buffer(
            render_client.get_buffer_size()? as usize,
            render_wave_format.get_nchannels() as usize,
            render_wave_format.get_bitspersample() as usize,
        );

        render_rend.write_to_device(prefill, &silence[..prefill * block_align], None)?;
        render_client.start_stream()?;

        self.set_capture_audio_client(Some(capture_client));
        self.set_capture_audio_cap(Some(capture_cap));

        self.set_render_audio_client(Some(render_client));
        self.set_render_audio_rend(Some(render_rend));

        Ok(())
    }

    pub fn bridge_loop(
        &mut self,
        mut callback: impl FnMut(&[u8], &mut [u8]),
        running: Arc<AtomicBool>,
    ) -> Result<(), HadesError> {
        let (buffer_size, n_channels, bits_per_sample) = {
            let client = self
                ._capture_audio_client
                .as_ref()
                .ok_or(HadesError::LoopbackBridgeCaptureAudioClientNotInitialized)?;

            let wfm = client.get_mixformat()?;

            (
                client.get_buffer_size()? as usize,
                wfm.get_nchannels() as usize,
                wfm.get_bitspersample() as usize,
            )
        };

        let (block_align, max_bytes, mut capture_scratch_buffer) =
            self.init_scratch_buffer(buffer_size, n_channels, bits_per_sample);

        let (_, _, mut dsp_scratch_buffer) =
            self.init_scratch_buffer(buffer_size, n_channels, bits_per_sample);
        let (_, _, mut dsp_scratch_output) =
            self.init_scratch_buffer(buffer_size, n_channels, bits_per_sample);

        let (_, _, mut render_scratch_buffer) =
            self.init_scratch_buffer(buffer_size, n_channels, bits_per_sample);

        let (mut capture_producer, mut capture_consumer) = RingBuffer::new(max_bytes);
        let (mut render_producer, mut render_consumer) = RingBuffer::new(max_bytes);

        let capture = self
            ._capture_audio_cap
            .as_ref()
            .ok_or(HadesError::LoopbackBridgeCaptureAudioCaptureClientNotInitialized)?;

        let rend_client = self
            ._render_audio_client
            .as_ref()
            .ok_or(HadesError::LoopbackBridgeRenderAudioClientNotInitialized)?;
        let render = self
            ._render_audio_rend
            .as_ref()
            .ok_or(HadesError::LoopbackBridgeRenderAudioRenderClientNotInitialized)?;

        while running.load(std::sync::atomic::Ordering::Acquire) {
            if let Some(packet_frames) = capture.get_next_packet_size()? {
                let packet_bytes = packet_frames as usize * block_align;
                let slice = &mut capture_scratch_buffer[..packet_bytes];

                capture.read_from_device(slice)?;
                let _ = capture_producer.push_entire_slice(slice)?;
            }

            let (input_slice, _remainder) =
                capture_consumer.pop_partial_slice(&mut dsp_scratch_buffer);

            if !input_slice.is_empty() {
                let n_bytes = input_slice.len();
                let output_slice = &mut dsp_scratch_output[..n_bytes];

                callback(input_slice, output_slice);

                let _ = render_producer.push_partial_slice(output_slice);
            }

            let available_frames = rend_client.get_available_space_in_frames()? as usize;
            if available_frames > 0 {
                let available_bytes = available_frames * block_align;
                let render_slice = &mut render_scratch_buffer[..available_bytes];

                let (pop_slice, _remainder) = render_consumer.pop_partial_slice(render_slice);

                if !pop_slice.is_empty() {
                    let frames_to_write = pop_slice.len() / block_align;
                    render.write_to_device(frames_to_write, pop_slice, None)?;
                }
            }
        }

        let cap_client = self
            ._capture_audio_client
            .as_ref()
            .ok_or(HadesError::LoopbackBridgeCaptureAudioClientNotInitialized)?;

        cap_client.stop_stream()?;
        rend_client.stop_stream()?;

        Ok(())
    }
}
