use std::{
    fs,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream, UdpSocket},
    thread,
    time::Duration,
};

fn main() {
    //client
    let socket = UdpSocket::bind("192.168.1.70:4002").expect("socket couldn't bind to address");
    socket
        .connect("192.168.1.38:4003") //server
        .expect("socket connect function failed");

    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let handle = thread::spawn(move || {
        for stream in listener.incoming() {
            let stream = stream.unwrap();
            println!("Connection established!");
            handle_connection(stream)
        }
    });

    println!("loop");
    loop {
        // we are using thread::sleep here to make sure the watchdog isn't triggered
        thread::sleep(Duration::from_millis(1000));
        println!("run");

        socket.send(&[5]).expect("couldn't send low message");
    }
    handle.join().unwrap();
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    println!("{}", request_line);
    let (status, filename) = match request_line.as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "hello.html"),
        "GET /ip HTTP/1.1" => ("HTTP/1.1 200 OK", "ip.html"),
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html"),
    };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status, length, contents
    );
    stream.write_all(response.as_bytes()).unwrap();
}
