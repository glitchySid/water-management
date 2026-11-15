use std::thread::sleep;

use esp_idf_hal::gpio::{Input, PinDriver};

use crate::config;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Off,
    Auto,
    On,
}

pub struct Switch {
    on: PinDriver<'static, Input>,
    off: PinDriver<'static, Input>,
    stable: Mode,
}

impl Switch {
    pub fn new(on: PinDriver<'static, Input>, off: PinDriver<'static, Input>) -> Self {
        Self {
            on,
            off,
            stable: Mode::Off,
        }
    }

    /// Reads twice 30 ms apart; a reading that does not agree is ignored.
    pub fn read(&mut self) -> Mode {
        let first = self.raw();
        sleep(config::SWITCH_DEBOUNCE);
        let second = self.raw();

        if first == second {
            self.stable = second;
        }

        self.stable
    }

    fn raw(&self) -> Mode {
        match (self.on.is_low(), self.off.is_low()) {
            (false, false) => Mode::Auto,
            (true, false) => Mode::On,
            _ => Mode::Off, // OFF wins if both contacts are closed.
        }
    }
}
