use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;
use std::thread;
fn main() {
    let thread_1 = thread::spawn(move || {
        let socket = UdpSocket::bind("192.168.1.38:4003").unwrap();
        println!("Successfully bind to 192.168.1.38:4003");
        let mut buf = [0; 1];
    
        loop {
            sleep(Duration::from_millis(20));
            socket.recv_from(&mut buf).unwrap();
            println!("4003: {:?}", buf);
        }
    });
    let thread_2 = thread::spawn(move || {
        let socket = UdpSocket::bind("192.168.1.38:4004").unwrap();
        println!("Successfully bind to 192.168.1.38:4004");
        let mut buf = [0; 1];
    
        loop {
            sleep(Duration::from_millis(20));
            socket.recv_from(&mut buf).unwrap();
            println!("4004: {:?}", buf);
        }
    });
    thread_1.join().unwrap();
    thread_2.join().unwrap();

    
}