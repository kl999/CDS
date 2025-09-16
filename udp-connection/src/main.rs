use std::time::Duration;
use std::{io, thread};

use crate::message::Message;
use crate::socket_worker_handshake::{receive_handshake, send_handshake};

mod socket_worker;
mod message;
mod socket_worker_handshake;
mod control_message;

#[cfg(test)]
mod tests;

fn main() {
    //_message_test();

    /*let socket1 = UdpSocket::bind("127.0.0.1:0").unwrap();
    let socket2 = UdpSocket::bind("127.0.0.1:0").unwrap();

    print!("{:?} | {:?}", socket1.local_addr(), socket2.local_addr());*/

    _socket_test();

    println!("End!");
}

fn _socket_test() {
    println!("Server or client?");
    
    let mut cmd = String::new();
    
    io::stdin().read_line(&mut cmd).expect("Error read_line");
    
    match cmd.to_lowercase().trim() {
        "server" => run_server(),
        "client" => run_client(),
        _ => { println!("Wrong command!") }
    }
}

fn run_server() {
    let mut worker = receive_handshake(
        "127.0.0.1:8080".to_string(),
        |msg| println!("{}", msg)).unwrap();

    println!("Made worker {:?}", worker);

    loop {
        //println!("work");
        worker.work();

        thread::sleep(Duration::from_millis(100));
    }
}

fn run_client() {
    let mut worker = send_handshake(
        "127.0.0.1:8080".to_string(),
        |msg| println!("{}", msg)).unwrap();

    println!("Made worker {:?}", worker);
    
    worker.send_message("Aaa it worked!".as_bytes().to_vec().into_boxed_slice());

    loop {
        //println!("work");
        worker.work();

        thread::sleep(Duration::from_millis(100));
    }
}

fn _message_test() {
    let m1 = Message::new(1, format!("hi!!!").as_bytes().to_vec().into_boxed_slice());

    println!("{}", m1);
}
