use std::fs::File;
use std::io::{BufReader, BufRead};

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use esp_idf_hal::delay::FreeRtos;
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    println!("Hello, main!");
    FreeRtos::delay_ms(1000);

// This is a very unrealistic example. You usually don't store the new app in the
// old app. Instead you obtain it by downloading it from somewhere or similar.
const NEW_APP: &[u8] = include_bytes!("../app.bin");


// Finds the next suitable OTA partition and erases it
let mut ota = esp_ota::OtaUpdate::begin().unwrap();

// Write the app to flash. Normally you would download
// the app and call `ota.write` every time you have obtained
// a part of the app image. This example is not realistic,
// since it has the entire new app bundled.
for app_chunk in NEW_APP.chunks(4096) {
    ota.write(app_chunk).unwrap();
}
//drop(NEW_APP);

// Performs validation of the newly written app image and completes the OTA update.
let mut completed_ota = ota.finalize().unwrap();

// Sets the newly written to partition as the next partition to boot from.
completed_ota.set_as_boot_partition().unwrap();
// Restarts the CPU, booting into the newly written app.
completed_ota.restart();

}
