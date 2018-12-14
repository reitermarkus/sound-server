use std::env;
use std::process::exit;

use rppal::gpio::Gpio;

use garage::{RELAY_PIN_1, RELAY_PIN_2, RELAY_PIN_3};
use garage::door::GarageDoor;

fn main() {
  let gpio = Gpio::new().unwrap();

  let mut pin1 = gpio.get(RELAY_PIN_1).unwrap();
  let mut pin2 = gpio.get(RELAY_PIN_2).unwrap();
  let mut pin3 = gpio.get(RELAY_PIN_3).unwrap();

  let mut output_pin1 = pin1.as_output();
  let mut output_pin2 = pin2.as_output();
  let mut output_pin3 = pin3.as_output();

  let mut door = GarageDoor::new(&mut output_pin1, &mut output_pin2, &mut output_pin3);

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
