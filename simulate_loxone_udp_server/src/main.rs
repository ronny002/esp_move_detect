use std::net::UdpSocket;
use std::thread::sleep;
use std::time::Duration;
fn main() {
    //server
    let udp_socket_1 = UdpSocket::bind("192.168.1.70:4003").unwrap();
    println!("Successfully bind to 192.168.1.70:4003");
    let udp_socket_2 = UdpSocket::bind("192.168.1.70:4004").unwrap();
    println!("Successfully bind to 192.168.1.70:4004");
    let mut udp_buf = [0; 1];

    loop {
        sleep(Duration::from_millis(100));

        udp_socket_1.recv_from(&mut udp_buf).unwrap();
        println!("udp from port 4003: {:?}", udp_buf);

        udp_socket_2.recv_from(&mut udp_buf).unwrap();
        println!("udp from port 4004: {:?}", udp_buf);
    }
}
