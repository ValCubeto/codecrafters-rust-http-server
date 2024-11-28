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
    println!("Connection ended");
    return Ok(());
  }
  let mut res = Response::new();

  println!("Received request: {:?}", req.path);

  
  let path: Vec<&str> = req.path.split('/').filter(|s| !s.is_empty()).collect();
  if path.is_empty() {
    res.send(stream);
    return Ok(());
  }

  match path[0] {
    "echo" => {
      res.headers.push(format!("Content-Length: {}", path[1].len()));
      res.body = path[1].to_owned();
    }
    _ => {
      res.status = 404;
      res.status_message = "Not Found".to_owned();
    }
  }
  res.send(stream);
  Ok(())
}
