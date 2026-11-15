//! Board wiring and timings. This is the only file that knows about the
//! simulation/real hardware difference.

use std::time::Duration;

use esp_idf_hal::gpio::{Input, Output, PinDriver, Pull};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::sys::EspError;

#[cfg(feature = "simulation")]
pub const RELAY_ACTIVE_HIGH: bool = true;
#[cfg(not(feature = "simulation"))]
pub const RELAY_ACTIVE_HIGH: bool = false;

#[cfg(feature = "simulation")]
pub const WATER_CHECK_DELAY: Duration = Duration::from_secs(5);
#[cfg(not(feature = "simulation"))]
pub const WATER_CHECK_DELAY: Duration = Duration::from_secs(10);

#[cfg(feature = "simulation")]
pub const MAX_RUN_TIME: Duration = Duration::from_secs(60);
#[cfg(not(feature = "simulation"))]
pub const MAX_RUN_TIME: Duration = Duration::from_secs(30 * 60);

#[cfg(feature = "simulation")]
pub const COOLDOWN: Duration = Duration::from_secs(10);
#[cfg(not(feature = "simulation"))]
pub const COOLDOWN: Duration = Duration::from_secs(5 * 60);

pub const PROBE_SETTLE: Duration = Duration::from_millis(5);
pub const PROBE_SAMPLE_GAP: Duration = Duration::from_millis(20);
pub const PROBE_SAMPLES: usize = 5;
pub const SWITCH_DEBOUNCE: Duration = Duration::from_millis(30);
pub const LOOP_INTERVAL: Duration = Duration::from_millis(200);

pub struct Pins {
    pub relay: PinDriver<'static, Output>,
    pub probe_drive: PinDriver<'static, Output>,
    pub probe_read: PinDriver<'static, Input>,
    pub switch_on: PinDriver<'static, Input>,
    pub switch_off: PinDriver<'static, Input>,
}

pub fn init(peripherals: Peripherals) -> Result<Pins, EspError> {
    let gpio = peripherals.pins;

    #[cfg(feature = "simulation")]
    let relay = PinDriver::output(gpio.gpio2)?;
    #[cfg(not(feature = "simulation"))]
    let relay = PinDriver::output(gpio.gpio26)?;

    Ok(Pins {
        relay,
        probe_drive: PinDriver::output(gpio.gpio25)?,
        probe_read: PinDriver::input(gpio.gpio27, Pull::Down)?,
        switch_on: PinDriver::input(gpio.gpio32, Pull::Up)?,
        switch_off: PinDriver::input(gpio.gpio33, Pull::Up)?,
    })
}
