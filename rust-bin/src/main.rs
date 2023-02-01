use esp_idf_hal::modem::Modem;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_sys::{self as _}; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use anyhow:: Result;


use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{PinDriver, Pull};
use esp_idf_hal::peripheral;
use esp_idf_hal::peripherals::Peripherals;

use esp_idf_svc::eventloop::{EspSystemEventLoop, EspEventLoop, System};
use esp_idf_svc::netif::{EspNetif, EspNetifWait};
use esp_idf_svc::ping::EspPing;
use esp_idf_svc::wifi::{EspWifi, WifiWait};

use embedded_svc::ipv4::Ipv4Addr;
use embedded_svc::wifi::ClientConfiguration;
use embedded_svc::wifi::Configuration;
use embedded_svc::wifi::{AccessPointConfiguration, Wifi};

use std::net::UdpSocket;
use std::time::Duration;

mod wifi_info;
use wifi_info::*;
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // unsafe{
    //     if ESP_OK != nvs_flash_init(){panic!("nvs_flash_init failed")}
    // }
    println!("start");

    let peripherals = Peripherals::take().unwrap();
    let mut move_input =
        PinDriver::input(peripherals.pins.gpio12).expect("couldn't set gpio to input");
    move_input
        .set_pull(Pull::Down)
        .expect("couldn't set input pin to pull down");

    let sysloop = EspSystemEventLoop::take().unwrap();
    #[cfg(not(feature = "qemu"))]
    //let _wifi =  wifi_simple(peripherals.modem, sysloop).expect("couldn't connect to wifi");
    let _wifi = wifi(peripherals.modem, sysloop).expect("couldn't connect to wifi");
    #[cfg(feature = "qemu")]
    let eth = eth_configure(
        &sysloop,
        Box::new(esp_idf_svc::eth::EspEth::wrap(
            esp_idf_svc::eth::EthDriver::new_openeth(peripherals.mac, sysloop.clone()).unwrap(),
        ).unwrap()),
    ).unwrap();
    ping(Ipv4Addr::new(192, 168, 1, 59)).unwrap();
    let socket = UdpSocket::bind("192.168.1.59:4002").expect("socket couldn't bind to address");
    socket
        .connect("192.168.1.59:4003")
        .expect("socket connect function failed");
    println!("loop");
    loop {
        // we are using thread::sleep here to make sure the watchdog isn't triggered
        FreeRtos::delay_ms(10);

        if move_input.is_high() {
            socket.send(&[1]).expect("couldn't send high message");
        } else {
            socket.send(&[0]).expect("couldn't send low message");
        }
    }
}


fn wifi_simple(modem: Modem, sys_loop: EspEventLoop<System>) -> Result<EspWifi<'static>>{
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi_driver = EspWifi::new(
        modem,
        sys_loop,
        Some(nvs)
    )?;

    wifi_driver.set_configuration(&Configuration::Client(ClientConfiguration{
        ssid: SSID.into(),
        password: PASS.into(),
        ..Default::default()
    }))?;

    wifi_driver.start()?;
    wifi_driver.connect()?;
    while !wifi_driver.is_connected()?{
        let config = wifi_driver.get_configuration()?;
        println!("Waiting for station {:?}", config);
    }
    println!("Should be connected now");
    for _ in 0..3{
        println!("IP info: {:?}", wifi_driver.sta_netif().get_ip_info()?);
        FreeRtos::delay_ms(1000);
    }
    Ok(wifi_driver)
}
#[cfg(not(feature = "qemu"))]
fn wifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    let mut wifi = Box::new(EspWifi::new(modem, sysloop.clone(), EspDefaultNvsPartition::take().ok())?);

    println!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID);

    let channel = if let Some(ours) = ours {
        println!(
            "Found configured access point {} on channel {}",
            SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        println!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: SSID.into(),
            password: PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    wifi.start()?;

    println!("Starting wifi...");

    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        println!("Wifi did not start");
    }

    println!("Connecting wifi...");

    wifi.connect()?;

    if !EspNetifWait::new::<EspNetif>(wifi.sta_netif(), &sysloop)?.wait_with_timeout(
        Duration::from_secs(20),
        || {
            wifi.is_connected().unwrap()
                && wifi.sta_netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        },
    ) {
        println!("Wifi did not connect or did not receive a DHCP lease");
    }

    let ip_info = wifi.sta_netif().get_ip_info()?;

    println!("Wifi DHCP info: {:?}", ip_info);

    ping(ip_info.subnet.gateway)?;

    Ok(wifi)
}

fn ping(ip: Ipv4Addr) -> Result<()> {
    println!("About to do some pings for {:?}", ip);

    let ping_summary = EspPing::default().ping(ip, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        println!("Pinging IP {} resulted in timeouts", ip);
    }

    println!("Pinging done");

    Ok(())
}

#[cfg(any(feature = "qemu"))]
fn eth_configure(
    sysloop: &EspSystemEventLoop,
    mut eth: Box<esp_idf_svc::eth::EspEth<'static>>,
) -> Result<Box<esp_idf_svc::eth::EspEth<'static>>> {
    use std::net::Ipv4Addr;

    println!("Eth created");

    eth.start()?;

    println!("Starting eth...");

    if !esp_idf_svc::eth::EthWait::new(eth.driver(), sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || eth.is_started().unwrap())
    {
        println!("Eth did not start");
    }

    if !EspNetifWait::new::<EspNetif>(eth.netif(), &sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || {
            eth.netif().get_ip_info().unwrap().ip != Ipv4Addr::new(0, 0, 0, 0)
        })
    {
        println!("Eth did not receive a DHCP lease");
    }

    let ip_info = eth.netif().get_ip_info()?;

    println!("Eth DHCP info: {:?}", ip_info);

    //ping(ip_info.subnet.gateway)?;

    Ok(eth)
}
