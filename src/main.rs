use std::net::TcpListener;
use std::io;
use std::io::Write;

const RES_200: &[u8] = b"HTTP/1.1 200 OK\r\n\r\n";

fn main() -> io::Result<()> {
  println!("Logs from your program will appear here!");

  let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

  for stream in listener.incoming() {
    match stream {
      Err(why) => println!("Error: {why}"),
      Ok(mut stream) => {
        println!("Accepted new connection");
        stream.write_all(RES_200)?;
        stream.flush()?;
      }
    }
  }

  Ok(())
}
