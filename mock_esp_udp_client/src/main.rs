use std::{net::UdpSocket, thread, time::Duration};

fn main() {
    let socket = UdpSocket::bind("127.0.0.1:1234").expect("socket couldn't bind to address");
    socket
        .connect("127.0.0.1:1235")
        .expect("socket connect function failed");
    println!("loop");
    loop {
        // we are using thread::sleep here to make sure the watchdog isn't triggered
        thread::sleep(Duration::from_millis(1000));


            socket.send(&[5]).expect("couldn't send low message");
        
    }
}

