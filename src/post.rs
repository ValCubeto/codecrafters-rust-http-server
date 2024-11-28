use std::collections::HashMap;
use std::io::Error;
use std::fs;
use std::net::TcpStream;
use std::path::Path;
use crate::Flags;
use http_request_parser::{ Request, Response };

pub fn handle_post(
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
