use std::time::Duration;
use std::thread::sleep;

use rppal::gpio::OutputPin;

pub struct GarageDoor {
  s0: OutputPin, // S0 - Taster HALT (normally closed)
  s2: OutputPin, // S2 - Taster AUF (normally open)
  s4: OutputPin, // S4 - Taster ZU (normally open)
}

impl GarageDoor {
  pub fn new(mut s0:  OutputPin, mut s2: OutputPin, mut s4: OutputPin) -> GarageDoor {
    s0.set_high();
    s2.set_high();
    s4.set_high();

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
