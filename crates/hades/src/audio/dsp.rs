use crate::audio::filter::{AudioFilter, AudioFilterOps, AudioFilterSync};

pub enum HadesAudioDSPCommands {
    UpdateFilters(Vec<AudioFilter>),
    SetEnabled(bool),
}

#[derive(Debug, Clone)]
pub struct HadesAudioDigitalSignalProcessingPipeline {
    pub filters: Vec<AudioFilter>,
    pub sample_rate: usize,
    pub nchannels: usize,
    pub enabled: bool,
}

impl HadesAudioDigitalSignalProcessingPipeline {
    pub fn new(filters: Option<Vec<AudioFilter>>, sample_rate: usize, nchannels: usize) -> Self {
        if let Some(f) = filters {
            Self {
                filters: f,
                sample_rate,
                nchannels,
                enabled: true,
            }
        } else {
            Self {
                filters: vec![],
                sample_rate,
                nchannels,
                enabled: true,
            }
        }
    }

    pub fn get_sample_rate(&self) -> usize {
        self.sample_rate
    }

    pub fn set_enabled(&mut self, new: bool) {
        self.enabled = new;
    }

    pub fn add_filter(&mut self, filter: AudioFilter) {
        self.filters.push(filter);
    }

    pub fn remove_filter(&mut self, id: u32) -> bool {
        let search = self.filters.iter().position(|f| f.get_id() == id);

        match search {
            Some(idx) => {
                self.filters.remove(idx);
                return true;
            }
            None => return false,
        }
    }

    pub fn update_filter(
        &mut self,
        id: u32,
        update_callback: impl FnOnce(&mut AudioFilter, usize),
    ) -> bool {
        if let Some(filter) = self.filters.iter_mut().find(|f| (*f).get_id() == id) {
            update_callback(filter, self.sample_rate);
            true
        } else {
            false
        }
    }

    pub fn sync_filters(&mut self, filters: Vec<AudioFilter>) {
        let mut current_filters = std::mem::take(&mut self.filters);
        let mut synced_filters = Vec::with_capacity(filters.len());

        for incoming_filter in filters {
            let current_filter = current_filters
                .iter()
                .position(|filter| filter.get_id() == incoming_filter.get_id())
                .map(|index| current_filters.remove(index));

            let Some(mut current_filter) = current_filter else {
                synced_filters.push(incoming_filter);
                continue;
            };

            match (&mut current_filter, &incoming_filter) {
                (
                    AudioFilter::ParametricEqualizer(current),
                    AudioFilter::ParametricEqualizer(incoming),
                ) => {
                    current.enabled = incoming.enabled;
                    current.sync_filters(incoming.filters.clone());
                }
            }

            synced_filters.push(current_filter);
        }

        self.filters = synced_filters;
    }

    pub fn _set_filters(&mut self, filters: Vec<AudioFilter>) {
        self.filters = filters;
    }
    pub fn get_filters_cloned(&self) -> Vec<AudioFilter> {
        self.filters.clone()
    }

    pub fn prepare_channels(&mut self) {
        for filter in &mut self.filters {
            filter.prepare_channels();
        }
    }

    pub fn process_audio(&mut self, buffer: &mut [f32], channels: usize) {
        if self.enabled {
            for filter in self.filters.iter_mut() {
                filter.process(buffer, channels);
            }
        }
    }
}
