use std::env;
use std::mem::drop;
use std::sync::{mpsc::channel, Arc, Mutex, RwLock};
use std::thread;

use cistern::Cistern;
use futures::{future, Future, Stream};
use gotham_derive::*;
use gotham;
use gotham::handler::{HandlerFuture, IntoResponse, IntoHandlerError};
use gotham::router::builder::*;
use gotham::state::{FromState, State};
use gotham::helpers::http::response::{create_response, create_empty_response};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::{new_pipeline, single::single_pipeline};
use hyper::{Body, StatusCode, Response};
use linux_embedded_hal::I2cdev;
use rppal::gpio::Gpio;
use serde_json::json;
use simple_signal::Signal;

use garage::{INPUT_PIN, RELAY_IN1_PIN, RELAY_IN2_PIN, RELAY_IN3_PIN};
use garage::GarageDoor;

#[derive(Clone, StateData)]
struct Door {
  pub inner: Arc<Mutex<GarageDoor>>,
}

#[derive(Clone, StateData)]
struct CisternState {
  pub inner: Arc<RwLock<Cistern<I2cdev>>>,
}

fn door_control_handler(mut state: State) -> Box<HandlerFuture>  {
  let f = Body::take_from(&mut state)
    .concat2()
    .then(|res| match res {
      Ok(body) => {
        match String::from_utf8(body.to_vec()) {
          Ok(content) => future::ok(content),
          Err(err) => future::err(err.into_handler_error()),
        }
      },
      Err(err) => future::err(err.into_handler_error()),
    })
    .then(|res| match res {
      Ok(body) => {
        match body.as_str() {
          "OPEN" => {
            println!("Opening door …");

            let mut door = Door::borrow_from(&state).inner.lock().unwrap();
            door.open();
          },
          "STOP" => {
            println!("Stopping door …");
            let mut door = Door::borrow_from(&state).inner.lock().unwrap();
            door.stop();
          },
          "CLOSE" => {
            println!("Closing door …");

            let mut door = Door::borrow_from(&state).inner.lock().unwrap();
            door.close();
          },
          _ => {
            let res = create_empty_response(&state, StatusCode::NOT_IMPLEMENTED);
            return future::ok((state, res))
          },
        }

        let res = create_empty_response(&state, StatusCode::OK);
        future::ok((state, res))
      }
      Err(e) => future::err((state, e)),
    });

  Box::new(f)
}

fn door_status_handler(state: State) -> Box<HandlerFuture> {
  let mut door = Door::borrow_from(&state).inner.lock().unwrap();

  let status = if door.is_closed() { "CLOSED" } else { "OPEN" };
  let response = status.into_response(&state);

  drop(door);

  Box::new(future::ok((state, response)))
}

fn cistern_level_handler(state: State) -> (State, Response<Body>) {
  let cistern = CisternState::borrow_from(&state).inner.read().unwrap();

  let json = cistern.level().map(|(height, percent, volume)| {
    json!({
      "fill_height": height,
      "percentage": percent * 100.0,
      "volume": volume,
    })
  });

  let response = create_response(
    &state,
    StatusCode::OK,
    mime::APPLICATION_JSON,
    serde_json::to_vec(&json).unwrap(),
  );

  drop(cistern);

  (state, response)
}

fn main() {
  let dev = I2cdev::new(env::var("I2C_DEVICE").expect("I2C_DEVICE is not set")).expect("Failed to open I2C device");

  let cistern = Arc::new(RwLock::new(Cistern::new(dev)));

  let gpio = Gpio::new().unwrap();

  let door = GarageDoor::new(
    gpio.get(RELAY_IN2_PIN).unwrap().into_output(),
    gpio.get(RELAY_IN1_PIN).unwrap().into_output(),
    gpio.get(RELAY_IN3_PIN).unwrap().into_output(),
    gpio.get(INPUT_PIN).unwrap().into_input_pullup(),
  );

  let door_middleware = StateMiddleware::new(Door { inner: Arc::new(Mutex::new(door)) });
  let cistern_middleware = StateMiddleware::new(CisternState { inner: cistern.clone() });

  let pipeline = new_pipeline()
    .add(door_middleware)
    .add(cistern_middleware)
    .build();

  let (chain, pipelines) = single_pipeline(pipeline);

  thread::spawn(move || {
    let (sig_tx, sig_rx) = channel();

    simple_signal::set_handler(&[Signal::Int], move |_| {
      sig_tx.send(true).unwrap();
    });

    loop {
      if sig_rx.try_recv().unwrap_or(false) {
        break
      }

      if let Err(e) = cistern.write().unwrap().measure() {
        eprintln!("Failed to measure: {:?}", e);
      }
    }

    std::process::exit(1);
  });

  let addr = "0.0.0.0:80";
  println!("Listening for requests at http://{}", addr);
  gotham::start(addr, build_router(chain, pipelines, |route| {
    route.post("/door").to(door_control_handler);
    route.get("/door").to(door_status_handler);
    route.get("/cistern").to(cistern_level_handler);
  }))
}
