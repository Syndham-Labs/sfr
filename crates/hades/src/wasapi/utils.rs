use wasapi::DeviceEnumerator;

use crate::{device::HadesAudioDevice, error::HadesError};

pub struct HadesAudioDeviceUtils;

impl HadesAudioDeviceUtils {
    pub fn find_render_device_by_partial_friendlyname(
        substring: String,
    ) -> Result<HadesAudioDevice, HadesError> {
        let enumerator = DeviceEnumerator::new()?;
        let collection = enumerator.get_device_collection(&wasapi::Direction::Render)?;
        let len = collection.get_nbr_devices()?;

        for i in 0..len {
            let device = collection.get_device_at_index(i)?;

            if device.get_friendlyname()?.contains(substring.as_str()) {
                return Ok(HadesAudioDevice::from_wasapi_device(device)?);
            }
        }

        Err(HadesError::DeviceNotFoundByFriendlyName(substring))
    }

    pub fn find_capture_device_by_partial_friendlyname(
        substring: String,
    ) -> Result<HadesAudioDevice, HadesError> {
        let enumerator = DeviceEnumerator::new()?;
        let collection = enumerator.get_device_collection(&wasapi::Direction::Capture)?;
        let len = collection.get_nbr_devices()?;

        for i in 0..len {
            let device = collection.get_device_at_index(i)?;

            if device.get_friendlyname()?.contains(substring.as_str()) {
                return Ok(HadesAudioDevice::from_wasapi_device(device)?);
            }
        }

        Err(HadesError::DeviceNotFoundByFriendlyName(substring))
    }
}
