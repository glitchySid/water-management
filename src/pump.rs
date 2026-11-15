use std::time::Instant;

use esp_idf_hal::sys::EspError;

use crate::config;
use crate::mode::{Mode, Switch};
use crate::motor::Motor;
use crate::probe::Probe;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Fault {
    Dry,
    MaxRun,
}

pub struct Pump {
    motor: Motor,
    probe: Probe,
    switch: Switch,
    run_start: Option<Instant>,
    last_stop: Option<Instant>,
    last_water_check: Option<Instant>,
    // shortcut: fault and cooldown are RAM-only; move them to NVS if a reboot must stay latched.
    fault: Option<Fault>,
    last_mode: Mode,
}

impl Pump {
    pub fn new(motor: Motor, probe: Probe, switch: Switch) -> Self {
        Self {
            motor,
            probe,
            switch,
            run_start: None,
            last_stop: None,
            last_water_check: None,
            fault: None,
            last_mode: Mode::Off,
        }
    }

    pub fn update(&mut self) -> Result<(), EspError> {
        let mode = self.switch.read();
        if mode != self.last_mode {
            log::info!("Mode: {mode:?}");
            self.last_mode = mode;
        }

        self.check_safety()?;

        match mode {
            Mode::Off => self.handle_off(),
            Mode::On => self.handle_on(),
            Mode::Auto => self.handle_auto(),
        }
    }

    fn check_safety(&mut self) -> Result<(), EspError> {
        let Some(start) = self.run_start else {
            return Ok(());
        };

        // Check the hard limit first so a probe error cannot bypass it.
        if start.elapsed() > config::MAX_RUN_TIME {
            return self.trip(Fault::MaxRun, "max run time exceeded");
        }

        if start.elapsed() > config::WATER_CHECK_DELAY && !self.probe.water_present()? {
            return self.trip(Fault::Dry, "no water at probe");
        }

        Ok(())
    }

    fn handle_off(&mut self) -> Result<(), EspError> {
        if self.motor.is_on() {
            self.stop("switch in OFF position")?;
        }

        if let Some(fault) = self.fault.take() {
            log::info!("Fault cleared: {fault:?}");
        }

        Ok(())
    }

    fn handle_on(&mut self) -> Result<(), EspError> {
        self.try_start(false)
    }

    fn handle_auto(&mut self) -> Result<(), EspError> {
        self.try_start(true)
    }

    /// Manual mode starts regardless of the probe; Auto waits for water. A dry
    /// trip clears itself once the probe sees water again.
    fn try_start(&mut self, require_water: bool) -> Result<(), EspError> {
        if self.motor.is_on() || !self.cooldown_passed() {
            return Ok(());
        }
        if self.fault == Some(Fault::MaxRun) {
            return Ok(());
        }

        let dry = self.fault == Some(Fault::Dry);
        if require_water || dry {
            // Probe at the water-check rate, not every loop, to limit corrosion.
            let due = self
                .last_water_check
                .is_none_or(|check| check.elapsed() >= config::WATER_CHECK_DELAY);
            if !due {
                return Ok(());
            }
            self.last_water_check = Some(Instant::now());
            if !self.probe.water_present()? {
                return Ok(());
            }
        }
        if dry {
            self.fault = None;
            log::info!("Water detected, dry fault cleared");
        }

        self.start()
    }

    fn start(&mut self) -> Result<(), EspError> {
        self.motor.on()?;
        self.run_start = Some(Instant::now());
        log::info!("Motor ON");
        Ok(())
    }

    fn trip(&mut self, fault: Fault, reason: &str) -> Result<(), EspError> {
        self.fault = Some(fault);
        log::warn!("Fault ({fault:?}): {reason}");
        self.stop(reason)
    }

    fn stop(&mut self, reason: &str) -> Result<(), EspError> {
        self.motor.off()?;
        self.run_start = None;
        self.last_stop = Some(Instant::now());
        log::info!("Motor OFF: {reason}");
        Ok(())
    }

    fn cooldown_passed(&self) -> bool {
        self.last_stop
            .is_none_or(|stop| stop.elapsed() >= config::COOLDOWN)
    }
}
