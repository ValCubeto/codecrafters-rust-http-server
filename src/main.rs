extern crate argmap;
extern crate http_request_parser;

mod connect;
mod bin_response;
mod get;
mod post;

use std::collections::HashMap;
use std::net::TcpListener;
use std::io::Error;
use std::sync::Arc;
use crate::connect::handle_connection;

pub type Flags = Arc<HashMap<String, Vec<String>>>;

fn main() -> Result<(), Error> {
  let (_args, flags) = argmap::parse(std::env::args());
  println!("Flags: {:?}", flags);
  let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

  let flags_ref = Arc::new(flags);
  for stream in listener.incoming() {
    match stream {
      Err(why) => eprintln!("Connection failed: {why}"),
      Ok(stream) => {
        println!("Accepted new connection...");
        let flags = Arc::clone(&flags_ref);
        std::thread::spawn(|| handle_connection(stream, flags));
      }
    }
  }

  Ok(())
}
