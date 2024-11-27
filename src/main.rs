extern crate http_request_parser;

use std::net::{ TcpListener, TcpStream };
use std::io::{Error, Read};
use http_request_parser::{ Request, Response };

fn main() -> Result<(), Error> {
  let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

  for stream in listener.incoming() {
    match stream {
      Err(why) => println!("Error: {why}"),
      Ok(mut stream) => {
        println!("Accepted new connection...");
        handle_connection(&mut stream)?;
      }
    }
  }

  Ok(())
}

fn handle_connection(stream: &mut TcpStream) -> Result<(), Error> {
  let req = Request::from(stream);
  if req.version == 0.0 {
    return Ok(());
  }
  let mut res = Response::new();

  println!("Received request: {:?}", req.path);

  if req.path == "/" {
    res.status = 200;
    res.status_message = "OK".to_owned();
  } else {
    res.status = 404;
    res.status_message = "Not Found".to_owned();
    println!("{res:?}");
  }
  res.send(stream);
  Ok(())
}
