extern crate rayon;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn handle_connection (mut stream: TcpStream)
{
  // Read up to 512 bytes
  let mut buffer = [0; 512];
  let len = stream.read (&mut buffer).unwrap ();

  let get = b"GET / HTTP/1.1\r\n";
  let (status_line, contents) = if buffer.starts_with (get)
  {
    ("HTTP/1.1 200 OK\r\n\r\n", format! ("invoked from browser: got {} bytes", len))
  }
  else
  {
    ("", format! ("direct access: got {} bytes", len))
  };
  
  let response = format! ("{}{}", status_line, contents);
  stream.write (response.as_bytes ()).unwrap ();
  stream.flush ().unwrap ();
}

fn main ()
{
  let pool = rayon::ThreadPoolBuilder::new ()
    .num_threads (8)
    .build ()
    .unwrap ();

  let listener = TcpListener::bind ("127.0.0.1:7878").unwrap ();
  for stream in listener.incoming ()
  {
    let stream = stream.unwrap ();
    pool.install(|| {
      handle_connection (stream);
    });
  }
}
