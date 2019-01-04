use std::env;
use std::process::exit;

use rppal::gpio::Gpio;

use garage::{RELAY_PIN_1, RELAY_PIN_2, RELAY_PIN_3};
use garage::door::GarageDoor;

fn main() {
  let gpio = Gpio::new().unwrap();

  let mut pin1 = gpio.get(RELAY_PIN_1).unwrap().into_output();
  let mut pin2 = gpio.get(RELAY_PIN_2).unwrap().into_output();
  let mut pin3 = gpio.get(RELAY_PIN_3).unwrap().into_output();

  let mut door = GarageDoor::new(&mut pin1, &mut pin2, &mut pin3);

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
