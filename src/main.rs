extern crate argmap;
extern crate http_request_parser;
extern crate flate2;

use std::collections::HashMap;
use std::{fs, vec};
use std::net::{ TcpListener, TcpStream };
use std::io::{Error, ErrorKind, Write};
use std::path::Path;
use std::sync::Arc;
use http_request_parser::{ Request, Response };
use flate2::{ Compression, write::GzEncoder };

fn main() -> Result<(), Error> {
  let (_args, flags) = argmap::parse(std::env::args());
  println!("Flags: {:?}", flags);
  let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

  let flags_ref = Arc::new(flags);
  for stream in listener.incoming() {
    match stream {
      Err(why) => println!("Error: {why}"),
      Ok(stream) => {
        println!("Accepted new connection...");
        let flags = flags_ref.clone();
        std::thread::spawn(|| handle_connection(stream, flags));
      }
    }
  }

  Ok(())
}

type Flags = Arc<HashMap<String, Vec<String>>>;

fn handle_connection(stream: TcpStream, flags: Flags) -> Result<(), Error> {
  let req = Request::from(&stream);
  if req.version == 0.0 {
    println!("Connection ended");
    return Ok(());
  }

  println!("Received request: {} {}", req.method, req.path);

  let path_text = req.path.to_lowercase();
  let path = parse_path(&path_text);

  let headers = parse_headers(&req.headers);

  let res = match req.method.as_str() {
    "GET" => handle_get(&stream, flags, req, path, &headers)?,
    "POST" => handle_post(&stream, flags, req, path, &headers)?,
    _ => {
      // Don't use Response::new() here, why would you want to
      // create a 200 OK response and then modify it?
      Response {
        version: 1.1,
        status: 400,
        status_message: "Bad Request".to_owned(),
        headers: vec!["Content-Type: text/plain".to_owned()],
        body: "".to_owned(),
      }
    }
  };
  if let Some(encodings) = headers.get("Accept-Encoding") {
    let encodings: Vec<&str> = encodings.split(',').map(|s| s.trim()).collect();
    // Ignore other encodings for now
    if encodings.contains(&"gzip") {
      let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
      encoder.write_all(res.body.as_bytes()).unwrap();
      let encoded = encoder.finish().unwrap();

      let mut encoded_res = EncodedResponse::from(res);
      encoded_res.body = encoded;
      encoded_res.headers.push("Content-Encoding: gzip".to_owned());
      encoded_res.headers.push(format!("Content-Length: {}", encoded_res.body.len()));
      encoded_res.send(&stream);
      return Ok(());
    }
  }
  res.send(&stream);
  Ok(())
}

fn handle_get(
  stream: &TcpStream,
  flags: Flags,
  _req: Request,
  path: Vec<&str>,
  headers: &HashMap<String, String>
) -> Result<Response, Error>
{
  let mut res = Response::new();

  if path.is_empty() {
    // GET "/", send a 200 OK response
    // In this case the 200 OK is the default Response, so no need to change anything
    return Ok(res);
  }

  match path[0] {
    "echo" => {
      res.headers.push(format!("Content-Length: {}", path[1].len()));
      res.body = path[1].to_owned();
    }
    "user-agent" => {
      let user_agent = match headers.get("User-Agent") {
        Some(ua) => ua.to_owned(),
        None => {
          res.status = 400;
          res.status_message = "Bad Request".to_owned();
          res.body = "Missing User-Agent header".to_owned();
          res.send(stream);
          return Err(Error::from(ErrorKind::InvalidInput));
        }
      };
      res.headers.push(format!("Content-Length: {}", user_agent.len()));
      res.body = user_agent;
    }
    "files" => {
      // The argmap crate returns a Vec<String> for each flag,
      // we are only interested in the last one
      let dir = flags
        .get("directory").expect("No directory specified")
        .last().expect("No directory specified");
      let file_name = path[1];
      let file_path = Path::new(dir).join(file_name);

      if !file_path.exists() {
        res.status = 404;
        res.status_message = "Not Found".to_owned();
        res.send(stream);
        return Err(Error::from(ErrorKind::NotFound));
      }

      // TODO: send Internal Server Error
      let contents = fs::read_to_string(&file_path)
        .unwrap_or_else(|why| panic!("Couldn't read file at {file_path:?}: {why}"));
      res.headers = vec![
        "Content-Type: application/octet-stream".to_owned(),
        format!("Content-Length: {}", contents.len()),
      ];
      res.body = contents;
    }
    _ => {
      res.status = 404;
      res.status_message = "Not Found".to_owned();
    }
  }
  Ok(res)
}

fn handle_post(
  stream: &TcpStream,
  flags: Flags,
  req: Request,
  path: Vec<&str>,
  _headers: &HashMap<String, String>
) -> Result<Response, Error>
{
  let mut res = Response::new();

  match path[0] {
    "files" => {
      let dir = flags
        .get("directory").expect("No directory specified")
        .last().expect("No directory specified");
      let file_name = path[1];
      let file_path = Path::new(dir).join(file_name);
      fs::write(file_path, req.body)?;
      res.status = 201;
      res.status_message = "Created".to_owned();
    }
    _ => {
      res.status = 404;
      res.status_message = "Not Found".to_owned();
      res.send(stream);
    }
  }
  Ok(res)
}

struct EncodedResponse {
  pub status: i32,
  pub status_message: String,
  pub headers: Vec<String>,
  pub body: Vec<u8>,
}

impl EncodedResponse {
  pub fn from(res: Response) -> EncodedResponse {
    EncodedResponse {
      status: res.status,
      status_message: res.status_message,
      headers: res.headers,
      body: Vec::new(),
    }
  }
  // Isn't this supposed to be an inmutable reference? I'm writing into it anyways
  pub fn send(&self, mut stream: &TcpStream) {
    let mut response = format!("HTTP/1.1 {} {}\r\n", self.status, self.status_message);
    for header in self.headers.clone() {
      response.push_str(format!("{}\r\n", header).as_str())
    }
    let mut bin_response = Vec::with_capacity(response.len() + self.body.len() + 4);
    bin_response.append(&mut response.as_bytes().to_vec());
    bin_response.push(b'\r');
    bin_response.push(b'\n');
    bin_response.append(&mut self.body.clone());
    bin_response.push(b'\r');
    bin_response.push(b'\n');
    stream.write_all(&bin_response).unwrap();
  }
}

fn parse_path(path: &str) -> Vec<&str> {
  path.split('/').filter(|s| !s.is_empty()).collect()
}

fn parse_headers(headers: &Vec<String>) -> HashMap<String, String> {
  let mut map = HashMap::new();
  for header in headers {
    let mut pair = header.split(':');
    let key = pair.next().unwrap().trim().to_owned();
    let value = pair.next().unwrap().trim().to_owned();
    map.insert(key, value);
  }
  map
}
