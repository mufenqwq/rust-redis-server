#![allow(unused_imports)]

use std::arch::x86_64::_mm256_load_si256;
use std::net::*;
use std::io::*;
use std::ptr::read;
use tokio::runtime::{Builder, Runtime};


fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
        loop {
            let (mut socket, _) = listener.accept().unwrap();
            tokio::spawn(async move {
                let mut buf = [0; 512];
                loop {
                    let n = match socket.read(&mut buf) {
                        Ok(read_count) => {
                            if read_count == 0 {
                                return;
                            }
                            socket.write(b"+PONG\r\n").unwrap();
                        }
                        Err(e ) => {
                            println!("error: {}", e);
                        }
                    };
                }
            });
        }
    });
}
