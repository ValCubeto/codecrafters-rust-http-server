extern crate flate2;

use std::collections::HashMap;
use std::io::{ Error, Write };
use std::net::TcpStream;
use crate::Flags;
use crate::get::handle_get;
use crate::post::handle_post;
use crate::bin_response::BinResponse;
use flate2::{ Compression, write::GzEncoder };
use http_request_parser::{ Request, Response };

pub fn handle_connection(stream: TcpStream, flags: Flags) -> Result<(), Error> {
  // Panics if the request data is empty...
  let req = Request::from(&stream);

  println!("Received request: {} {}", req.method, req.path);

  let path_text = req.path.to_lowercase();
  let path = path_text.split('/').filter(|s| !s.is_empty()).collect();

  let headers = parse_headers(&req.headers);

  let mut res = match req.method.as_str() {
    "GET" => handle_get(&stream, flags, req, path, &headers)?,
    "POST" => handle_post(&stream, flags, req, path, &headers)?,
    _ => {
      // Prevent creating instantly-dropped Strings
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

      res.headers.push(format!("Content-Length: {}", encoded.len()));
      let mut bin_res = BinResponse::from(res);
      bin_res.body = encoded;

      bin_res.headers.push("Content-Encoding: gzip".to_owned());
      bin_res.send(&stream);
      return Ok(());
    }
  }
  res.headers.push(format!("Content-Length: {}", res.body.len()));
  res.send(&stream);
  Ok(())
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
