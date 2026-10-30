use crate::audio::filter::parametric::ParametricEqualizationFilter;

pub mod parametric;

#[derive(Debug, Clone)]
pub enum AudioFilter {
    ParametricEqualizer(ParametricEqualizationFilter),
}

impl AudioFilterOps for AudioFilter {
    fn process(&mut self, buffer: &mut [f32], channels: usize) {
        match self {
            AudioFilter::ParametricEqualizer(f) => f.process(buffer, channels),
        }
    }
    fn get_id(&self) -> u32 {
        match self {
            AudioFilter::ParametricEqualizer(f) => f.get_id(),
        }
    }
}

impl AudioFilter {
    pub fn prepare_channels(&mut self) {
        match self {
            AudioFilter::ParametricEqualizer(f) => f.prepare_channels(),
        }
    }
}

pub trait ParametricFilterOps {
    fn process(&mut self, buffer: &mut [f32], channels: usize);
}

pub trait AudioFilterOps {
    fn process(&mut self, buffer: &mut [f32], channels: usize);
    fn get_id(&self) -> u32;
}

pub trait AudioFilterSync<T> {
    fn sync_filters(&mut self, other: Vec<T>);
}
