//fill out and rename to wifi_info.rs
//target wifi
pub const SSID: &str = "SSID";

pub const PASS: &str = "PASSWORD";

//own wifi
#[cfg(not(feature = "qemu"))]
#[cfg(esp_idf_lwip_ipv4_napt)]
pub const SSID_AP: &str = "SSID_AP";

#[cfg(not(feature = "qemu"))]
#[cfg(esp_idf_lwip_ipv4_napt)]
pub const PASS_AP: &str = "PASSWORD_AP";