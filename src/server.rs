use std::io::Read;
use std::net::TcpListener;

use crate::messages::Message;

pub fn server<Handler: Fn(Message)>(port: u32, handler: Handler) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).expect("Failed to initialize server");
    println!("[INFO] Coolttviz started, awaiting connections");
    for stream in listener.incoming() {
        let mut stream = stream.expect("Failed to accept");
        println!("[INFO] Connected");
        let mut str = String::new();
        match stream.read_to_string(&mut str) {
            Result::Ok(0) => (),
            Result::Ok(_) => {
                match serde_json::from_str(&str) {
                    Result::Ok(msg) => handler(msg),
                    Result::Err(err) => println!("Deserialization Error: {:?}", err)
                }
            },
            Result::Err(err) => println!("Read Error: {:?}", err)
        }
    }
}
