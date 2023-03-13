use std::{io::Write, net::TcpStream};

fn main() {
    //const NEW_APP: &[u8] =
    //include_bytes!("/home/ronny/Documents/code/rust/esp_move_detect/ota_first_try/ota-app/app.bin");
    const NEW_APP: &[u8] =
    include_bytes!("/home/ronny/Documents/code/rust/esp_move_detect/rust-bin/app_ota.bin");
    println!("Total length of app: {:?}", NEW_APP.len());

    let mut stream = TcpStream::connect("192.168.1.37:5003").unwrap(); //esp ip
    for app_chunk in NEW_APP.chunks(4096) {
        println!("{:?}", app_chunk.len());
        stream.write(app_chunk).unwrap();
    }
}
