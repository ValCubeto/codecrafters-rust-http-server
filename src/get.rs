use std::fs;
use std::path::Path;
use std::collections::HashMap;
use std::io::{ Error, ErrorKind };
use std::net::TcpStream;
use crate::Flags;
use http_request_parser::{ Request, Response };

pub fn handle_get(
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
      res.body = path[1].to_owned();
    }
    "user-agent" => {
      let user_agent = match headers.get("User-Agent") {
        Some(ua) => ua.to_owned(),
        None => {
          res.status = 400;
          res.status_message = "Bad Request".to_owned();
          res.send(stream);
          return Err(ErrorKind::InvalidInput.into());
        }
      };
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
        return Err(ErrorKind::NotFound.into());
      }

      // TODO: send Internal Server Error
      let contents = fs::read_to_string(&file_path)
        .unwrap_or_else(|why| panic!("Couldn't read file at {file_path:?}: {why}"));
      res.headers = vec![
        "Content-Type: application/octet-stream".to_owned()
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
