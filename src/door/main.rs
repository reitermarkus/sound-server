use std::sync::{Arc, Mutex};
use std::mem::drop;

use futures::{future, Future, Stream};
use gotham_derive::*;
use gotham;
use gotham::handler::{HandlerFuture, IntoResponse, IntoHandlerError};
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};
use gotham::helpers::http::response::create_empty_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use hyper::{Body, StatusCode};
use rppal::gpio::Gpio;

use garage::{INPUT_PIN, RELAY_IN1_PIN, RELAY_IN2_PIN, RELAY_IN3_PIN};
use garage::GarageDoor;

#[derive(Clone, StateData)]
struct Door {
  pub inner: Arc<Mutex<GarageDoor>>,
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

fn router() -> Router {
  let gpio = Gpio::new().unwrap();

  let door = GarageDoor::new(
    gpio.get(RELAY_IN2_PIN).unwrap().into_output(),
    gpio.get(RELAY_IN1_PIN).unwrap().into_output(),
    gpio.get(RELAY_IN3_PIN).unwrap().into_output(),
    gpio.get(INPUT_PIN).unwrap().into_input_pullup(),
  );

  let middleware = StateMiddleware::new(Door { inner: Arc::new(Mutex::new(door)) });

  let pipeline = single_middleware(middleware);

  let (chain, pipelines) = single_pipeline(pipeline);

  build_router(chain, pipelines, |route| {
    route.post("/door")
         .to(door_control_handler);

    route.get("/door").to(door_status_handler)
  })
}

fn main() {
  let addr = "0.0.0.0:80";
  println!("Listening for requests at http://{}", addr);
  gotham::start(addr, router())
}
