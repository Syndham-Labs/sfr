/*
 * Implementation from the Audio EQ Cookbook
 * W3C Working Group Note, 08 June 2021
 * Issue Tracking: https://github.com/WebAudio/Audio-EQ-Cookbook/issues
 * Author: Robert Bristow-Johnson
 * Editor: Raymond Toy (Google) [toy.raymond@gmail.com]
 *
 * Latest Published Version: https://www.w3.org/TR/audio-eq-cookbook/
 * Copyright © 2021 W3C® (MIT, ERCIM, Keio, Beihang)
 */

use std::f32::consts::PI;

use crate::audio::filter::ParametricFilterOps;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BiquadMode {
    DirectFormI,
    DirectFormII,
    TransposedDirectFormII,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BiquadType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
    Peaking,
    FlatTopBell,
    LowShelf,
    HighShelf,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum BiquadPan {
    Left,
    Right,
    Centre,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BiquadState {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    s1: f32,
    s2: f32,
}

#[derive(Debug, Clone)]
pub struct BiquadFilter {
    pub id: u32,
    pub gain: f32,
    pub freq: u32,
    pub q: f32,
    pub band: BiquadType,
    pub sample_rate: usize,
    pub nchannels: usize,

    // TODO: Implementation
    pub pan: BiquadPan,

    pub mode: BiquadMode,
    pub enabled: bool,

    // One state per interleaved audio channel.
    pub states: Vec<BiquadState>,

    // Filter Coefficients
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl BiquadFilter {
    pub fn new_raw(
        id: u32,
        gain: f32,
        freq: u32,
        q: f32,
        band: BiquadType,
        pan: BiquadPan,
        mode: BiquadMode,
        sample_rate: usize,
        nchannels: usize,
        enabled: bool,
    ) -> Self {
        BiquadFilter {
            id,
            gain,
            freq,
            q,
            band,
            pan,
            mode,
            sample_rate,
            nchannels,
            enabled,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            states: Vec::new(),
        }
    }

    pub fn new(id: u32, freq: u32, sample_rate: usize, nchannels: usize) -> Self {
        BiquadFilter {
            id,
            gain: 0.0,
            freq,
            q: 0.70,
            band: BiquadType::Peaking,
            pan: BiquadPan::Centre,
            mode: BiquadMode::TransposedDirectFormII,
            sample_rate,
            nchannels,
            enabled: true,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
            states: Vec::new(),
        }
    }

    pub fn set_gain(&mut self, new: f32) {
        self.gain = new;
        self.update_coefficients();
    }
    pub fn set_freq(&mut self, new: u32) {
        self.freq = new;
        self.update_coefficients();
    }
    pub fn set_q(&mut self, new: f32) {
        self.q = new;
        self.update_coefficients();
    }
    pub fn set_band(&mut self, new: BiquadType) {
        self.band = new;
        self.update_coefficients();
    }
    // pub fn update_pan(&mut self, new: BiquadPan) {
    //     self.pan = new;
    // }
    pub fn set_mode(&mut self, new: BiquadMode) {
        self.mode = new;
        self.update_coefficients();
    }
    pub fn set_enabled(&mut self, new: bool) {
        self.enabled = new;
    }

    pub fn prepare_channels(&mut self) {
        // assert!(channels > 0, "a biquad requires at least one audio channel");
        self.states = vec![BiquadState::default(); self.nchannels];
    }

    pub fn update_coefficients(&mut self) {
        let fs = self.sample_rate as f32;
        let f0 = self.freq as f32;

        let db_gain = self.gain as f32;
        let a_gain = 10f32.powf(db_gain / 40.0);

        let omega = 2.0 * PI * f0 / fs;
        let (sin_omega, cos_omega) = omega.sin_cos();

        let q = self.q as f32;

        let alpha = sin_omega / (2.0 * q);

        let (b0_raw, b1_raw, b2_raw, a0_raw, a1_raw, a2_raw) = match self.band {
            BiquadType::LowPass => {
                let b0 = (1.0 - cos_omega) / 2.0;
                let b1 = 1.0 - cos_omega;
                let b2 = (1.0 - cos_omega) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;

                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::HighPass => {
                let b0 = (1.0 + cos_omega) / 2.0;
                let b1 = -(1.0 + cos_omega);
                let b2 = (1.0 + cos_omega) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;

                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::BandPass => {
                let b0 = alpha * q;
                let b1 = 0.0;
                let b2 = -alpha * q;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;

                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha;

                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::Peaking => {
                let b0 = 1.0 + alpha * a_gain;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0 - alpha * a_gain;
                let a0 = 1.0 + alpha / a_gain;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha / a_gain;

                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::FlatTopBell => {
                let flat_factor = 1.5;
                let alpha_num = alpha * flat_factor;

                let b0 = 1.0 + alpha_num * flat_factor;
                let b1 = -2.0 * cos_omega;
                let b2 = 1.0 - alpha_num * a_gain;
                let a0 = 1.0 + alpha / a_gain;
                let a1 = -2.0 * cos_omega;
                let a2 = 1.0 - alpha / a_gain;

                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::LowShelf => {
                let beta = 2.0 * a_gain.sqrt() * alpha;

                let b0 = a_gain * ((a_gain + 1.0) - (a_gain - 1.0) * cos_omega + beta);
                let b1 = 2.0 * a_gain * ((a_gain - 1.0) - (a_gain + 1.0) * cos_omega);
                let b2 = a_gain * ((a_gain + 1.0) - (a_gain - 1.0) * cos_omega - beta);
                let a0 = (a_gain + 1.0) + (a_gain - 1.0) * cos_omega + beta;
                let a1 = -2.0 * ((a_gain - 1.0) + (a_gain + 1.0) * cos_omega);
                let a2 = (a_gain + 1.0) + (a_gain - 1.0) * cos_omega - beta;

                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::HighShelf => {
                let beta = 2.0 * a_gain.sqrt() * alpha;

                let b0 = a_gain * ((a_gain + 1.0) + (a_gain - 1.0) * cos_omega + beta);
                let b1 = -2.0 * a_gain * ((a_gain - 1.0) + (a_gain + 1.0) * cos_omega);
                let b2 = a_gain * ((a_gain + 1.0) + (a_gain - 1.0) * cos_omega - beta);
                let a0 = (a_gain + 1.0) - (a_gain - 1.0) * cos_omega + beta;
                let a1 = 2.0 * ((a_gain - 1.0) - (a_gain + 1.0) * cos_omega);
                let a2 = (a_gain + 1.0) - (a_gain - 1.0) * cos_omega - beta;

                (b0, b1, b2, a0, a1, a2)
            }
        };

        self.b0 = b0_raw / a0_raw;
        self.b1 = b1_raw / a0_raw;
        self.b2 = b2_raw / a0_raw;
        self.a1 = a1_raw / a0_raw;
        self.a2 = a2_raw / a0_raw;
    }

    pub fn parameters_equal(&self, other: &BiquadFilter) -> bool {
        self.id == other.id
            && self.gain.to_bits() == other.gain.to_bits()
            && self.freq == other.freq
            && self.q.to_bits() == other.q.to_bits()
            && self.band == other.band
            && self.pan == other.pan
            && self.mode == other.mode
            && self.enabled == other.enabled
    }

    pub fn update_from(&mut self, other: &BiquadFilter) {
        if self.gain.to_bits() != other.gain.to_bits() {
            self.set_gain(other.gain);
        }
        if self.freq != other.freq {
            self.set_freq(other.freq);
        }
        if self.q.to_bits() != other.q.to_bits() {
            self.set_q(other.q);
        }
        if self.band != other.band {
            self.set_band(other.band.clone());
        }
        if self.pan != other.pan {
            self.pan = other.pan.clone();
        }
        if self.mode != other.mode {
            self.set_mode(other.mode.clone());
        }
        if self.enabled != other.enabled {
            self.set_enabled(other.enabled);
        }
    }
}

impl ParametricFilterOps for BiquadFilter {
    fn process(&mut self, buffer: &mut [f32], channels: usize) {
        if self.enabled {
            // assert_eq!(
            //     self.states.len(),
            //     channels,
            //     "biquad channel state must be prepared before processing"
            // );

            for frame in buffer.chunks_exact_mut(channels) {
                for (channel, sample) in frame.iter_mut().enumerate() {
                    let state = &mut self.states[channel];
                    let x = *sample;

                    let y = match self.mode {
                        BiquadMode::DirectFormI => {
                            let y = self.b0 * x + self.b1 * state.x1 + self.b2 * state.x2
                                - self.a1 * state.y1
                                - self.a2 * state.y2;

                            state.x2 = state.x1;
                            state.x1 = x;
                            state.y2 = state.y1;
                            state.y1 = y;
                            y
                        }
                        BiquadMode::DirectFormII => {
                            let w = x - self.a1 * state.s1 - self.a2 * state.s2;
                            let y = self.b0 * w + self.b1 * state.s1 + self.b2 * state.s2;

                            state.s2 = state.s1;
                            state.s1 = w;
                            y
                        }
                        BiquadMode::TransposedDirectFormII => {
                            let y = self.b0 * x + state.s1;

                            state.s1 = self.b1 * x - self.a1 * y + state.s2;
                            state.s2 = self.b2 * x - self.a2 * y;
                            y
                        }
                    };

                    *sample = y;
                }
            }
        }
    }
}
