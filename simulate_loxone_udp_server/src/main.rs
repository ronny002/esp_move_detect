use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;
fn main() {                                         //server
    let socket = UdpSocket::bind("192.168.1.59:4003").unwrap();
    println!("Successfully bind to 192.168.1.59:4003");

    let mut buf = [0; 1];
    loop {
        sleep(Duration::from_millis(100));
        socket.recv_from(&mut buf).unwrap();
        println!("{:?}", buf);
    }
}
