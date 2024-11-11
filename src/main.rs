#![allow(unused_imports)]
use std::net::*;
use std::io::*;
use std::ptr::read;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");


    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                let mut buf = [0; 512];
                stream.read(&mut buf).unwrap();
                loop {
                    let read_count = stream.read(&mut buf).unwrap();
                    if read_count == 0 {
                        break;
                    }
                    stream.write(b"+PONG\r\n").unwrap();
                }


            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
