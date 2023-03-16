//fill out and rename file to wifi_info.rs
pub const SSID: &str = "SSID";

pub const PASS: &str = "PASSWORD";

#[cfg(not(feature = "qemu"))]
#[cfg(esp_idf_lwip_ipv4_napt)]
pub const PASS_AP: &str = "PASSWORD";