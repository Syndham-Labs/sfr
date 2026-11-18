use wasapi::{Device, DeviceEnumerator, DeviceState, Direction, WasapiError};

use crate::error::HadesError;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HadesDeviceDirection {
    Render,
    Capture,
}

impl From<Direction> for HadesDeviceDirection {
    fn from(value: Direction) -> Self {
        match value {
            Direction::Render => Self::Render,
            Direction::Capture => Self::Capture,
        }
    }
}

impl Into<Direction> for HadesDeviceDirection {
    fn into(self) -> Direction {
        match self {
            Self::Render => Direction::Render,
            Self::Capture => Direction::Capture,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum HadesDeviceState {
    Active,
    Disabled,
    NotPresent,
    Unplugged,
}

impl From<DeviceState> for HadesDeviceState {
    fn from(value: DeviceState) -> Self {
        match value {
            DeviceState::Active => Self::Active,
            DeviceState::Disabled => Self::Disabled,
            DeviceState::NotPresent => Self::NotPresent,
            DeviceState::Unplugged => Self::Unplugged,
        }
    }
}

impl Into<DeviceState> for HadesDeviceState {
    fn into(self) -> DeviceState {
        match self {
            Self::Active => DeviceState::Active,
            Self::Disabled => DeviceState::Disabled,
            Self::NotPresent => DeviceState::NotPresent,
            Self::Unplugged => DeviceState::Unplugged,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct HadesAudioDevice {
    id: String,
    direction: HadesDeviceDirection,
    state: HadesDeviceState,
    name: String,
    description: String,
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
            direction: HadesDeviceDirection::from(direction),
            name,
            description,
            state: HadesDeviceState::from(state),
        }
    }
    pub fn from_wasapi_device(d: Device) -> Result<Self, WasapiError> {
        Ok(Self {
            id: d.get_id()?,
            direction: HadesDeviceDirection::from(d.get_direction()),
            state: HadesDeviceState::from(d.get_state()?),
            name: d.get_friendlyname()?,
            description: d.get_description()?,
        })
    }
    pub fn to_wasapi_device(&self) -> Result<Device, HadesError> {
        let enumerator = DeviceEnumerator::new()?;
        let collection = enumerator.get_device_collection(&self.direction.clone().into())?;

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
    pub fn get_direction(&self) -> HadesDeviceDirection {
        self.direction.clone()
    }
    pub fn get_wasapi_direction(&self) -> Direction {
        self.direction.clone().into()
    }
    pub fn get_state(&self) -> HadesDeviceState {
        self.state.clone()
    }
    pub fn get_wasapi_state(&self) -> DeviceState {
        self.state.clone().into()
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_desc(&self) -> String {
        self.description.clone()
    }
}
