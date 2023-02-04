use std::io::{Write, Read};
use std::net::{UdpSocket, TcpStream, TcpListener};
use std::thread::sleep;
use std::time::Duration;
fn main() {                                         //server
    let udp_socket = UdpSocket::bind("10.22.22.14:4003").unwrap();
    println!("Successfully bind to 10.22.22.14:4003");
    let mut tcp_stream = TcpListener::bind("10.22.22.14:14002").unwrap();
    
    let mut udp_buf = [0; 1];
    let mut tcp_buf = String::from("");

    loop {
        sleep(Duration::from_millis(100));
        
        udp_socket.recv_from(&mut udp_buf).unwrap();
        println!("udp: {:?}", udp_buf);
        
        for stream in tcp_stream.incoming(){
            stream.unwrap().read_to_string(&mut tcp_buf).unwrap();
        }
        println!("tcp: {}", tcp_buf);

    }
}
