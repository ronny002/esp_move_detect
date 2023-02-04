use std::{net::{UdpSocket, TcpStream}, thread, time::Duration, io::{Write, Read}};

fn main() {                                         //client
    let socket = UdpSocket::bind("10.22.22.14:4002").expect("socket couldn't bind to address");
    socket
        .connect("192.168.1.191:4003")   //server
        .expect("socket connect function failed");

    let mut tcp_stream = TcpStream::connect("10.22.22.14:14002").unwrap();
    println!("loop");
    let mut tcp_buf = [0; 128];
    loop {
        // we are using thread::sleep here to make sure the watchdog isn't triggered
        thread::sleep(Duration::from_millis(1000));

        socket.send(&[5]).expect("couldn't send low message");
        tcp_stream.write(&[7]).unwrap();
        //tcp_stream.read(&mut tcp_buf).unwrap();
        //println!("tcp: {:?}", tcp_buf);
    }
}
