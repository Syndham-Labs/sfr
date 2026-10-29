use wasapi::{Device, DeviceEnumerator, DeviceState, Direction, WasapiError};

use crate::error::HadesError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HadesAudioDevice {
    id: String,
    direction: Direction,
    state: DeviceState,
    name: String,
    description: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceChangeType {
    Render(HadesAudioDevice),
    Capture(HadesAudioDevice),
}

impl HadesAudioDevice {
    pub fn new(
        id: String,
        direction: Direction,
        state: DeviceState,
        name: String,
        description: String,
    ) -> Self {
        Self {
            id,
            direction,
            name,
            description,
            state,
        }
    }
    pub fn from_wasapi_device(d: Device) -> Result<Self, WasapiError> {
        Ok(Self {
            id: d.get_id()?,
            direction: d.get_direction(),
            state: d.get_state()?,
            name: d.get_friendlyname()?,
            description: d.get_description()?,
        })
    }
    pub fn to_wasapi_device(&self) -> Result<Device, HadesError> {
        let enumerator = DeviceEnumerator::new()?;
        let collection = enumerator.get_device_collection(&self.direction)?;

        let count = collection.get_nbr_devices()?;
        for i in 0..count {
            let endpoint = collection.get_device_at_index(i)?;
            if let Ok(endpoint_id) = endpoint.get_id() {
                if endpoint_id == self.id {
                    return Ok(endpoint);
                }
            }
        }

        Err(HadesError::DeviceNotFoundByID(self.id.clone()))
    }

    pub fn get_id(&self) -> String {
        self.id.clone()
    }
    pub fn get_direction(&self) -> Direction {
        self.direction.clone()
    }
    pub fn get_state(&self) -> DeviceState {
        self.state.clone()
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_desc(&self) -> String {
        self.description.clone()
    }
}
