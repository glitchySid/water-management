mod config;
mod mode;
mod motor;
mod probe;
mod pump;

use std::thread;

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::sys::EspError;
use esp_idf_svc::log::EspLogger;

use mode::Switch;
use motor::Motor;
use probe::Probe;
use pump::Pump;

fn main() -> Result<(), EspError> {
    esp_idf_hal::sys::link_patches();
    EspLogger::initialize_default();
    log::info!("Water pump controller starting");

    let peripherals = Peripherals::take()?;
    let pins = config::init(peripherals)?;

    let motor = Motor::new(pins.relay, config::RELAY_ACTIVE_HIGH)?;
    let probe = Probe::new(pins.probe_drive, pins.probe_read)?;
    let switch = Switch::new(pins.switch_on, pins.switch_off);
    let mut pump = Pump::new(motor, probe, switch);

    log::info!("Motor OFF (boot)");

    loop {
        if let Err(e) = pump.update() {
            log::error!("Pump update failed: {e}");
        }
        thread::sleep(config::LOOP_INTERVAL);
    }
}
