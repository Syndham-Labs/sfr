use bytemuck::{cast_slice, pod_align_to};
use wasapi::{SampleType, WaveFormat};

use crate::error::HadesError;

#[derive(Debug, Clone)]
pub enum AudioSubformat {
    F32,
    I16,
    I32,
}

impl AudioSubformat {
    fn format_is_float(subformat: &SampleType) -> bool {
        matches!(subformat, SampleType::Float)
    }

    pub fn from_wave_format(wave_fmt: &WaveFormat) -> Result<Self, HadesError> {
        let bits = wave_fmt.get_bitspersample();
        let subformat = wave_fmt.get_subformat()?;

        match (bits, AudioSubformat::format_is_float(&subformat)) {
            (32, true) => Ok(AudioSubformat::F32),
            (16, false) => Ok(AudioSubformat::I16),
            (32, false) => Ok(AudioSubformat::I32),
            _ => Err(HadesError::UnsupportedWaveFormatBitFormat(bits.to_string())),
        }
    }

    #[inline(always)]
    pub fn decode_to_f32(&self, raw_bytes: &[u8], float_scratch: &mut [f32]) {
        match self {
            AudioSubformat::F32 => {
                let (_prefix, floats, _suffix) = pod_align_to::<u8, f32>(raw_bytes);
                let len = floats.len().min(float_scratch.len());
                float_scratch[..len].copy_from_slice(&floats[..len]);
            }
            AudioSubformat::I16 => {
                for (i, chunk) in raw_bytes.chunks_exact(2).enumerate() {
                    if i >= float_scratch.len() {
                        break;
                    }
                    let sample = i16::from_le_bytes([chunk[0], chunk[1]]);
                    float_scratch[i] = sample as f32 / 32768.0;
                }
            }
            AudioSubformat::I32 => {
                for (i, chunk) in raw_bytes.chunks_exact(4).enumerate() {
                    if i >= float_scratch.len() {
                        break;
                    }
                    let sample = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                    float_scratch[i] = sample as f32 / 2147483648.0;
                }
            }
        }
    }

    #[inline(always)]
    pub fn encode_from_f32(&self, float_samples: &[f32], raw_bytes: &mut [u8]) {
        match self {
            AudioSubformat::F32 => {
                let bytes = cast_slice(float_samples);
                let len = bytes.len().min(raw_bytes.len());
                raw_bytes[..len].copy_from_slice(&bytes[..len]);
            }
            AudioSubformat::I16 => {
                for (i, sample) in float_samples.iter().enumerate() {
                    let byte_start = i * 2;
                    if byte_start + 2 > raw_bytes.len() {
                        break;
                    }

                    let sample = (sample.clamp(-1.0, 1.0) * 32768.0) as i16;
                    raw_bytes[byte_start..byte_start + 2].copy_from_slice(&sample.to_le_bytes());
                }
            }
            AudioSubformat::I32 => {
                for (i, sample) in float_samples.iter().enumerate() {
                    let byte_start = i * 4;
                    if byte_start + 4 > raw_bytes.len() {
                        break;
                    }

                    let sample = (sample.clamp(-1.0, 1.0) * 2147483648.0) as i32;
                    raw_bytes[byte_start..byte_start + 4].copy_from_slice(&sample.to_le_bytes());
                }
            }
        }
    }
}
