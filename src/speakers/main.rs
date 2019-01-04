use std::env;
use std::process::exit;

use rppal::gpio::{Gpio, Mode, Level};

use garage::RELAY_PIN_4;

fn main() {
  let gpio = Gpio::new().unwrap();

  let args: Option<String> = env::args().nth(1);

  let mut pin14 = gpio.get(RELAY_PIN_4).unwrap();

  match args.as_ref().map(|s| s.as_str()) {
    Some("on") => {
      let mut relay = pin14.into_output();
      relay.set_reset_on_drop(false);
      relay.set_low();
    },
    Some("off") => {
      let mut relay = pin14.into_output();
      relay.set_reset_on_drop(false);
      relay.set_high();
    },
    Some("status") => {
      if pin14.mode() == Mode::Output {
        match pin14.into_output().read() {
          Level::Low => println!("on"),
          Level::High => println!("off"),
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
