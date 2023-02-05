use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, UdpSocket};
use std::thread::sleep;
use std::time::Duration;
fn main() {
    //server
    let udp_socket = UdpSocket::bind("192.168.1.70:4003").unwrap();
    println!("Successfully bind to 192.168.1.70:4003");

    let mut udp_buf = [0; 1];

    loop {
        sleep(Duration::from_millis(100));

        udp_socket.recv_from(&mut udp_buf).unwrap();
        println!("udp: {:?}", udp_buf);
    }
}
