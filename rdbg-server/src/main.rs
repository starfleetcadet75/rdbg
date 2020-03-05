use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 512];
    stream.read(&mut buffer).unwrap();
    println!("Request: {}", String::from_utf8_lossy(&buffer[..]));

    // Handle API functionality here
    // This server can be compiled and run on any target system
}

// TODO: This is just placeholder code for now. Eventually this will be a standalone server
// executable that wraps the sys crate.
fn main() {
    let listener = TcpListener::bind("127.0.0.1:2159").unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}
