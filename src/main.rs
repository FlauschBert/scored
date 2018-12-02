extern crate threadpool;

use threadpool::ThreadPool;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::str;
use std::{thread, time};

fn handle_connection (mut stream: TcpStream)
{
  thread::sleep (time::Duration::from_millis(500));

  // Read up to 256 bytes
  let mut buffer = [0; 512];
  let len = stream.read (&mut buffer).unwrap ();

  println!("Got {} with {} bytes", str::from_utf8 (&buffer).unwrap (), len);

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
  let max_threads = 10;
  let pool = ThreadPool::new (max_threads);

  let listener = TcpListener::bind ("127.0.0.1:7878").unwrap ();
  for stream in listener.incoming ()
  {
    let stream = stream.unwrap ();
    let count = pool.queued_count ();
    println! ("Queue: {}", count);
    if count > 1 {
      continue;
    }

    pool.execute (|| {
      handle_connection (stream);
    });
  }
}
