use std::{net::UdpSocket, thread, time::Duration};

fn main() {
    let socket = UdpSocket::bind("192.168.1.38:4002").expect("socket couldn't bind to address");
    socket
        .connect("192.168.1.38:4003")
        .expect("socket connect function failed");
    println!("loop");
    loop {
        // we are using thread::sleep here to make sure the watchdog isn't triggered
        thread::sleep(Duration::from_millis(1000));

        socket.send(&[5]).expect("couldn't send low message");
    }
}
