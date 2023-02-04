use std::{net::UdpSocket, thread, time::Duration};

fn main() {                                         //client
    let socket = UdpSocket::bind("10.22.22.14:4002").expect("socket couldn't bind to address");
    socket
        .connect("192.168.1.191:4003")   //server
        .expect("socket connect function failed");
    println!("loop");
    loop {
        // we are using thread::sleep here to make sure the watchdog isn't triggered
        thread::sleep(Duration::from_millis(1000));

        socket.send(&[5]).expect("couldn't send low message");
    }
}
