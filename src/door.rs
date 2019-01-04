use std::time::Duration;
use std::thread::sleep;

use rppal::gpio::OutputPin;

pub struct GarageDoor<'a> {
  s0: &'a mut OutputPin, // S0 - Taster HALT (normally closed)
  s2: &'a mut OutputPin, // S2 - Taster AUF (normally open)
  s4: &'a mut OutputPin, // S4 - Taster ZU (normally open)
}

impl<'a> GarageDoor<'a> {
  pub fn new(s0: &'a mut OutputPin, s2: &'a mut OutputPin, s4: &'a mut OutputPin) -> GarageDoor<'a> {
    GarageDoor { s0, s2, s4 }
  }

  pub fn open(&mut self) {
    self.s0.set_high();
    self.s4.set_high();

    self.s2.set_low();
    sleep(Duration::from_millis(500));
    self.s2.set_high();
  }

  pub fn stop(&mut self) {
    self.s2.set_high();
    self.s4.set_high();

    self.s0.set_low();
    sleep(Duration::from_millis(500));
    self.s0.set_high();
  }

  pub fn close(&mut self) {
    self.s0.set_high();
    self.s2.set_high();

    self.s4.set_low();
    sleep(Duration::from_millis(500));
    self.s4.set_high();
  }
}
