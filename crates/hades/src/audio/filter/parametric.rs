use crate::audio::filter::{
    AudioFilterOps, AudioFilterSync, ParametricFilterOps,
    parametric::biquad::{BiquadFilter, BiquadMode, BiquadPan, BiquadType},
};

pub mod biquad;

#[derive(Debug, Clone)]
pub struct ParametricEqualizationFilter {
    pub id: u32,
    pub filters: Vec<BiquadFilter>,
    pub nchannels: usize,
    pub enabled: bool,
}

impl ParametricEqualizationFilter {
    pub fn new(
        id: u32,
        filters: Option<Vec<BiquadFilter>>,
        nchannels: usize,
        enabled: bool,
    ) -> Self {
        let filter_vec = match filters {
            Some(f) => f,
            None => vec![],
        };

        Self {
            id,
            filters: filter_vec,
            nchannels,
            enabled,
        }
    }

    pub fn set_enabled(&mut self, new: bool) {
        self.enabled = new;
    }

    pub fn new_biquad_filter(&mut self, freq: u32, sample_rate: usize) {
        let id = match self.filters.last() {
            Some(f) => f.id + 1,
            None => 0,
        };

        let mut bfilter = BiquadFilter::new(id, freq, sample_rate, self.nchannels);
        bfilter.update_coefficients();

        self.filters.push(bfilter);
    }
    pub fn new_raw_biquad_filter(
        &mut self,
        gain: f32,
        freq: u32,
        q: f32,
        band: BiquadType,
        pan: BiquadPan,
        mode: BiquadMode,
        sample_rate: usize,
        enabled: bool,
    ) {
        let id = match self.filters.last() {
            Some(f) => f.id + 1,
            None => 0,
        };

        let mut filter = BiquadFilter::new_raw(
            id,
            gain,
            freq,
            q,
            band,
            pan,
            mode,
            sample_rate,
            self.nchannels,
            enabled,
        );
        filter.update_coefficients();

        self.filters.push(filter);
    }

    pub fn remove_filter(&mut self, id: u32) -> bool {
        let search = self.filters.iter().position(|f| f.id == id);

        match search {
            Some(idx) => {
                self.filters.remove(idx);
                return true;
            }
            None => return false,
        }
    }

    pub fn update_filter(&mut self, id: u32, update_callback: impl FnOnce(&BiquadFilter)) -> bool {
        if let Some(filter) = self.filters.iter().find(|f| f.id == id) {
            update_callback(filter);
            true
        } else {
            false
        }
    }

    pub fn prepare_channels(&mut self) {
        for filter in &mut self.filters {
            filter.prepare_channels();
        }
    }
}

impl AudioFilterOps for ParametricEqualizationFilter {
    fn process(&mut self, buffer: &mut [f32], channels: usize) {
        if self.enabled {
            for filter in self.filters.iter_mut() {
                filter.process(buffer, channels);
            }
        }
    }

    fn get_id(&self) -> u32 {
        self.id
    }
}

impl AudioFilterSync<BiquadFilter> for ParametricEqualizationFilter {
    fn sync_filters(&mut self, filters: Vec<BiquadFilter>) {
        for f1 in filters.iter() {
            if let Some(f2) = self.filters.iter_mut().find(|f| f.id == f1.id) {
                if !f2.parameters_equal(&f1) {
                    f2.update_from(&f1);
                }
            } else {
                let mut new_filter = f1.clone();
                new_filter.prepare_channels();
                self.filters.push(new_filter);
            }
        }

        self.filters
            .retain(|f2| filters.iter().any(|f1| f1.id == f2.id));
    }
}
