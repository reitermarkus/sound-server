use std::sync::{Arc, Mutex};

use futures::{future, Future, Stream};
use gotham_derive::*;
use gotham;
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::router::builder::*;
use gotham::router::Router;
use gotham::state::{FromState, State};
use gotham::helpers::http::response::create_empty_response;
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::single::single_pipeline;
use gotham::pipeline::single_middleware;
use hyper::{Body, StatusCode};
use rppal::gpio::Gpio;

use garage::{RELAY_IN1_PIN, RELAY_IN2_PIN, RELAY_IN3_PIN};
use garage::GarageDoor;

#[derive(Clone, StateData)]
struct Door {
  pub inner: Arc<Mutex<GarageDoor>>,
}

fn door_handler(mut state: State) -> Box<HandlerFuture>  {
  let f = Body::take_from(&mut state)
      .concat2()
      .then(|full_body| match full_body {
          Ok(valid_body) => {
              let body_content = String::from_utf8(valid_body.to_vec()).unwrap();
              match body_content.as_str() {
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
          Err(e) => return future::err((state, e.into_handler_error())),
      });

  Box::new(f)
}

fn router() -> Router {
  let gpio = Gpio::new().unwrap();

  let door = GarageDoor::new(
    gpio.get(RELAY_IN2_PIN).unwrap().into_output(),
    gpio.get(RELAY_IN3_PIN).unwrap().into_output(),
    gpio.get(RELAY_IN1_PIN).unwrap().into_output(),
  );

  let middleware = StateMiddleware::new(Door { inner: Arc::new(Mutex::new(door)) });

  let pipeline = single_middleware(middleware);

  let (chain, pipelines) = single_pipeline(pipeline);

  build_router(chain, pipelines, |route| {
    route.post("/door")
         .to(door_handler);
  })
}

fn main() {
  let addr = "0.0.0.0:80";
  println!("Listening for requests at http://{}", addr);
  gotham::start(addr, router())
}
