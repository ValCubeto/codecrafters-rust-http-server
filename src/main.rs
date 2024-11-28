extern crate http_request_parser;

use std::collections::HashMap;
use std::net::{ TcpListener, TcpStream };
use std::io::Error;
use http_request_parser::{ Request, Response };

fn main() -> Result<(), Error> {
  let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

  for stream in listener.incoming() {
    match stream {
      Err(why) => println!("Error: {why}"),
      Ok(stream) => {
        println!("Accepted new connection...");
        std::thread::spawn(|| handle_connection(stream));
      }
    }
  }

  Ok(())
}

fn handle_connection(stream: TcpStream) -> Result<(), Error> {
  let req = Request::from(&stream);
  if req.version == 0.0 {
    println!("Connection ended");
    return Ok(());
  }
  let mut res = Response::new();

  println!("Received request: {:?}", req.path);

  let path_text = req.path.to_lowercase();
  let path = parse_path(&path_text);
  if path.is_empty() {
    // GET "/", send a 200 OK response
    // In this case the 200 OK is the default Response, so no need to change anything
    res.send(&stream);
    return Ok(());
  }

  match path[0] {
    "echo" => {
      res.headers.push(format!("Content-Length: {}", path[1].len()));
      res.body = path[1].to_owned();
    }
    "user-agent" => {
      let headers = parse_headers(req.headers);
      let user_agent = match headers.get("User-Agent") {
        Some(ua) => ua.to_owned(),
        None => {
          res.status = 400;
          res.status_message = "Bad Request".to_owned();
          res.body = "Missing User-Agent header".to_owned();
          res.send(&stream);
          return Ok(());
        }
      };
      res.headers.push(format!("Content-Length: {}", user_agent.len()));
      res.body = user_agent;
    }
    _ => {
      res.status = 404;
      res.status_message = "Not Found".to_owned();
    }
  }
  res.send(&stream);
  Ok(())
}

fn parse_path(path: &str) -> Vec<&str> {
  path.split('/').filter(|s| !s.is_empty()).collect()
}

fn parse_headers(headers: Vec<String>) -> HashMap<String, String> {
  let mut map = HashMap::new();
  for header in headers {
    let mut pair = header.split(':');
    let key = pair.next().unwrap().trim().to_owned();
    let value = pair.next().unwrap().trim().to_owned();
    map.insert(key, value);
  }
  map
}
