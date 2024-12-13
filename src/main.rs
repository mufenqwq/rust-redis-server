#![allow(unused_imports)]

mod resp;

use std::io::*;
use std::net::*;
use tokio::net::{TcpListener, TcpStream};
use anyhow::Result;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::macros::support::maybe_done;
use crate::resp::Value;

#[tokio::main]
async fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // todo resp 格式
    // SimpleString "+abc/r/n"
    //Integer ":0/r/n"
    // BulkString "$5/r/nHello/r/n"
    // Errors "-xxx/r/n"
    // Arrays "*2/r/n$5/r/nHello/r/n$5/r/nWorld/r/n"
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    loop {
        let stream = listener.accept().await;
        match stream {
            Ok((mut  stream, _)) => {
                println!("accepted new connection");
                
                tokio::spawn(async move {
                    let mut buf = [0; 512];
                    loop {
                        let read_count = stream.read(&mut buf).await.unwrap();
                        if read_count == 0 {
                            break;
                        }
                        stream.write(b"+PONG\r\n").await.unwrap();
                    }
                    handle_connection(stream).await
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
        
    }

    /*    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
        loop {
            let (mut socket, _) = listener.accept().unwrap();
            //todo accept method can block thread until it comes a connection!!! Then tokio::spawn
            tokio::spawn(async move {
                let mut buf = [0; 512];
                loop {
                    let n = match socket.read(&mut buf) {
                        Ok(read_count) => {
                            if read_count == 0 {
                                return;
                            }
                            read_count
                        }
                        Err(e ) => {
                            println!("error: {}", e);
                            0
                        }
                    };
                    let message = &buf[0..n];
                    let message = std::str::from_utf8(message).unwrap();
                    if message
                }
            });
        }
    });*/
}

async fn handle_connection(stream: TcpStream) {
    let mut handler = resp::RespHandler::new(stream);
    
    println!("Starting read loop");
    loop {
        let value = handler.read_value().await.unwrap();
        println!("Got value {:?}", value);
        
        let response = if let Some(v) = value {
            let (command, args) = extract_command(v).unwrap();
            match command.as_str() { 
                "ping" => Value::SimpleString("PONG".to_string()),
                "echo" => args.first().unwrap().clone(),
                c => panic!("Cannot handle command {}", c),
            }
        } else {
            break;
        };
        
        println!("Sending value {:?}", response);
        
        handler.write_value(response).await.unwrap();
    }
}


fn extract_command(value: Value) -> Result<(String, Vec<Value>)> {
    match value {
        Value::Array(a ) => {
            Ok((
                unpack_bulk_string(a.first().unwrap().clone())?,
                a.into_iter().skip(1).collect(),
                ))
        },
        _ => Err(anyhow::anyhow!("Unexpected command format")),
    }
}

fn unpack_bulk_string(value: Value) -> Result<String> {
    match value {
        Value::BulkString(s) => {Ok(s)}
        _ => Err(anyhow::anyhow!("Expected to be a bulk string"))
    }
}
