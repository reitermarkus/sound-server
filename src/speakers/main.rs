use std::env;
use std::process::exit;

use rppal::gpio::{Gpio, Mode, Level};

use garage::RELAY_IN4_PIN;

fn main() {
  let gpio = Gpio::new().unwrap();

  let args: Option<String> = env::args().nth(1);

  let pin = gpio.get(RELAY_IN4_PIN).unwrap();

  match args.as_ref().map(|s| s.as_str()) {
    Some("on") => {
      let mut relay = pin.into_output();
      relay.set_reset_on_drop(false);
      relay.set_low();
    },
    Some("off") => {
      let mut relay = pin.into_output();
      relay.set_reset_on_drop(false);
      relay.set_high();
    },
    Some("status") => {
      if pin.mode() == Mode::Output {
        if pin.into_output().is_set_low() {
          println!("on") ;
        } else {
          println!("off");
        }
      } else {
        println!("unknown");
      }
    },
    _ => {
      eprintln!("Argument must be exactly one of on/off.");
      exit(1);
    },
  }
}
