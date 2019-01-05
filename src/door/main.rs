use std::env;
use std::process::exit;

use rppal::gpio::Gpio;

use garage::{RELAY_IN1_PIN, RELAY_IN2_PIN, RELAY_IN3_PIN};
use garage::GarageDoor;

fn main() {
  let gpio = Gpio::new().unwrap();

  let mut door = GarageDoor::new(
    gpio.get(RELAY_IN2_PIN).unwrap().into_output(),
    gpio.get(RELAY_IN3_PIN).unwrap().into_output(),
    gpio.get(RELAY_IN1_PIN).unwrap().into_output(),
  );

  let args: Option<String> = env::args().nth(1);

  match args.as_ref().map(|s| s.as_str()) {
    Some("open") => door.open(),
    Some("stop") => door.stop(),
    Some("close") => door.close(),
    _ => {
      eprintln!("Argument must be exactly one of open/stop/close.");
      exit(1);
    },
  }
}
