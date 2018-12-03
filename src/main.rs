extern crate threadpool;
extern crate log;
extern crate simple_logging;

// logging
use log::{info, warn, error};
use log::LevelFilter;

use threadpool::ThreadPool;

use std::process;
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

fn activate_logging (log_file: &String, log_level: LevelFilter)
{
  match simple_logging::log_to_file (log_file, log_level)
  {
    Ok (_) => info! ("Activated logging to {}.", log_file),
    Err (err) => {
      simple_logging::log_to_stderr (log_level);
      warn! ("Could not log to {}. Error: {}.", err, log_file);
      info! ("Logging to stderr instead.");
    }
  };
}

fn bind_to (ip_address: &String, port: i32) -> Result<TcpListener,i32>
{
  let dst = format! ("{}:{}", ip_address, port);
  return match TcpListener::bind (&dst)
  {
    Ok (listener) => {
      info! ("Bound to {}.", dst);
      Ok (listener)
    },
    Err (err) => {
      error! ("Could not bind to {}: {}.", dst, err);
      Err(1)
    }
  };
}

fn peer_address (stream : &TcpStream) -> String
{
  return match stream.peer_addr () 
  {
    Ok (addr) => format! ("{}", addr),
    Err (_) => String::from ("unknown")
  };
}

fn main ()
{
  let max_threads = 4;
  let max_queue_depth = 10;
  let ip = String::from ("127.0.0.1");
  let port = 7878;
  let log_file = String::from ("scored.log");
  let log_level = LevelFilter::Info;

  activate_logging (&log_file, log_level);

  info! ("Starting scored with thread pool size {}.", max_threads);

  // Bind to address and port
  // Exit on error
  let listener = match bind_to (&ip, port)
  {
    Ok (listener) => listener,
    Err (error_code) => process::exit (error_code)
  };

  let pool = ThreadPool::new (max_threads);
  for stream in listener.incoming ()
  {
    // Check stream for error
    // Drop on error
    let stream = match stream
    {
      Ok (stream) => stream,
      Err (err) => {
        warn! ("Stream error: {}. Ignoring ...", err);
        continue;
      }
    };

    let peer_address = peer_address (&stream);

    // Queue up to max queue depth: Drop if depth too deep
    let queued_count = pool.queued_count ();
    if queued_count > max_queue_depth
    {
      warn! ("Too many streams queued: {}. Dropping peer {}...",
        queued_count, peer_address
      );
      continue;
    }

    info! ("Processing incoming stream of peer {}.", peer_address);

    pool.execute (|| {
      handle_connection (stream);
    });
  }
}
