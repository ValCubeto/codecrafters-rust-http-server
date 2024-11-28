use std::io::Write;
use std::net::TcpStream;
use http_request_parser::Response;

pub struct BinResponse {
  pub status: i32,
  pub status_message: String,
  pub headers: Vec<String>,
  pub body: Vec<u8>,
}

impl BinResponse {
  pub fn from(res: Response) -> BinResponse {
    BinResponse {
      status: res.status,
      status_message: res.status_message,
      headers: res.headers,
      body: Vec::new(),
    }
  }
  // Isn't this supposed to be an inmutable reference? I'm writing into it anyways
  pub fn send(&mut self, mut stream: &TcpStream) {
    let mut res = format!("HTTP/1.1 {} {}\r\n", self.status, self.status_message);
    for header in self.headers.iter() {
      res.push_str(format!("{}\r\n", header).as_str())
    }
    let mut bytes = Vec::with_capacity(res.len() + self.body.len() + 4);
    bytes.append(&mut res.as_bytes().to_vec());
    bytes.push(b'\r');
    bytes.push(b'\n');
    bytes.append(&mut self.body);
    bytes.push(b'\r');
    bytes.push(b'\n');
    stream.write_all(&bytes).unwrap();
  }
}
