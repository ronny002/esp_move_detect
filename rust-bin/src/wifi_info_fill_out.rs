//fill out and rename to wifi_info.rs
//target wifi
pub const SSID_TARGET: &str = "SSID to connect to";
pub const PASS_TARGET: &str = "PASSWORD";

//own (ap) wifi
#[cfg(not(feature = "qemu"))]
#[cfg(esp_idf_lwip_ipv4_napt)]
pub const SSID_AP: &str = "SSID to host";
#[cfg(not(feature = "qemu"))]
#[cfg(esp_idf_lwip_ipv4_napt)]
pub const PASS_AP: &str = "PASSWORD";

//udp settings
pub const UDP_SERVER_IP: &str = "IP to send sensor data to";  //loxone server 192.168.1.222
pub const UDP_SERVER_PORT: &str = "PORT to send sensor data to";         //4003, 4004 ...