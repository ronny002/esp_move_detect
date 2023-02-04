use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;
fn main() {                                         //server
    let socket = UdpSocket::bind("10.22.22.14:4003").unwrap();
    println!("Successfully bind to 10.22.22.14:4003");

    let mut buf = [0; 1];
    loop {
        sleep(Duration::from_millis(100));
        socket.recv_from(&mut buf).unwrap();
        println!("{:?}", buf);
    }
}
