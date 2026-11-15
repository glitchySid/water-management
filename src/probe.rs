use std::thread::sleep;

use esp_idf_hal::gpio::{Input, Output, PinDriver};
use esp_idf_hal::sys::EspError;

use crate::config;

pub struct Probe {
    drive: PinDriver<'static, Output>,
    read: PinDriver<'static, Input>,
}

impl Probe {
    pub fn new(
        drive: PinDriver<'static, Output>,
        read: PinDriver<'static, Input>,
    ) -> Result<Self, EspError> {
        let mut probe = Self { drive, read };
        probe.drive.set_low()?; // Leave the probe unpowered between readings.
        Ok(probe)
    }

    /// Powers the probe briefly and takes a majority vote to ignore splashing.
    pub fn water_present(&mut self) -> Result<bool, EspError> {
        self.drive.set_high()?;
        sleep(config::PROBE_SETTLE);

        let mut wet = 0;
        for sample in 0..config::PROBE_SAMPLES {
            if self.read.is_high() {
                wet += 1;
            }
            if sample + 1 < config::PROBE_SAMPLES {
                sleep(config::PROBE_SAMPLE_GAP);
            }
        }

        self.drive.set_low()?; // Depower immediately after reading.
        Ok(wet * 2 > config::PROBE_SAMPLES)
    }
}
