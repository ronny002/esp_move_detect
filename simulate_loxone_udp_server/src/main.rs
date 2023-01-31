use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;
fn main() {
    let socket = UdpSocket::bind("127.0.0.1:1235").unwrap();
    println!("Successfully bind to 127.0.0.1:1235");

    let mut buf = [0; 1];
    loop {
        sleep(Duration::from_secs(1));
        println!("{:?}", socket.recv_from(&mut buf).unwrap());
    }
}
