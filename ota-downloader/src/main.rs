use std::{io::Write, net::TcpStream, fs::read};

fn main() {
    let ip_esp = std::env::args().nth(1).expect("no esp ip given");
    let path_bin: String = std::env::args().nth(2).expect("no bin path given");

   // const NEW_APP: &[u8] =
   //     include_bytes!(bin_ota);
    let app = read(path_bin).unwrap();
    println!("Total length of app: {:?}", app.len());

    let mut stream = TcpStream::connect(format!("{}:5003", ip_esp)).unwrap(); //esp ip
    for app_chunk in app.chunks(4096) {
        println!("{:?}", app_chunk.len());
        stream.write(app_chunk).unwrap();
    }
}
