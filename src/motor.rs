use esp_idf_hal::gpio::{Level, Output, PinDriver};
use esp_idf_hal::sys::EspError;

pub struct Motor {
    pin: PinDriver<'static, Output>,
    active_high: bool,
    is_on: bool,
}

impl Motor {
    pub fn new(pin: PinDriver<'static, Output>, active_high: bool) -> Result<Self, EspError> {
        let mut pin = pin;
        pin.set_level(Self::level(active_high, false))?;

        Ok(Self {
            pin,
            active_high,
            is_on: false,
        })
    }

    pub fn on(&mut self) -> Result<(), EspError> {
        self.pin.set_level(Self::level(self.active_high, true))?;
        self.is_on = true;
        Ok(())
    }

    pub fn off(&mut self) -> Result<(), EspError> {
        self.pin.set_level(Self::level(self.active_high, false))?;
        self.is_on = false;
        Ok(())
    }

    pub fn is_on(&self) -> bool {
        self.is_on
    }

    fn level(active_high: bool, on: bool) -> Level {
        if on == active_high {
            Level::High
        } else {
            Level::Low
        }
    }
}
