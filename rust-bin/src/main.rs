use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::gpio::{Level, PinDriver, Pull};
use esp_idf_hal::modem::Modem;
use esp_idf_hal::peripheral;
use esp_idf_hal::peripherals::Peripherals;

use esp_idf_svc::eventloop::{EspEventLoop, EspSystemEventLoop, System};
use esp_idf_svc::http::server::{Configuration, EspHttpServer};
use esp_idf_svc::netif::{EspNetif, EspNetifWait};
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::ping::EspPing;
use esp_idf_svc::wifi::{EspWifi, WifiWait};

use embedded_svc::http::Method;
use embedded_svc::io::Write;
use embedded_svc::ipv4::Ipv4Addr;
use embedded_svc::wifi::{
    AccessPointConfiguration, ClientConfiguration, Configuration as WifiConfig, Wifi,
};

use anyhow::Result;

mod wifi_info;
use wifi_info::*;
#[derive(Clone)]
struct Ip {
    own: Ipv4Addr,
    server: Ipv4Addr,
}
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();

    println!("---------------start---------------");
    let peripherals = Peripherals::take().unwrap();
    let mut move_input_pin =
        PinDriver::input(peripherals.pins.gpio17).expect("couldn't set gpio to input");
    move_input_pin
        .set_pull(Pull::Down)
        .expect("couldn't set input pin to pull down");

    println!("---------------set up wifi---------------");
    let sysloop = EspSystemEventLoop::take().unwrap();
    #[cfg(not(feature = "qemu"))]
    //let _wifi =  wifi_simple(peripherals.modem, sysloop).expect("couldn't connect to wifi");
    let wifi = wifi(peripherals.modem, sysloop).expect("couldn't connect to wifi");
    #[cfg(feature = "qemu")]
    let eth = eth_configure(
        &sysloop,
        Box::new(
            esp_idf_svc::eth::EspEth::wrap(
                esp_idf_svc::eth::EthDriver::new_openeth(peripherals.mac, sysloop.clone()).unwrap(),
            )
            .unwrap(),
        ),
    )
    .unwrap();
    let ip = Ip {
        own: wifi.sta_netif().get_ip_info().unwrap().ip,
        server: "192.168.1.38".parse::<Ipv4Addr>().unwrap(), //server ip loxone 192.168.1.222
    };
    ping(ip.server).unwrap();

    println!("---------------set up http_server---------------");
    let mut status;
    let (_http_server, status_main) = http_server(ip.clone()).unwrap();

    println!("---------------set up udp---------------");
    let socket =
        UdpSocket::bind(format!("{}:4002", ip.own)).expect("socket couldn't bind to address");
    socket
        .connect(format!("{}:4003", ip.server))
        .expect("socket connect function failed");

    println!("---------------start loop---------------");
    let mut toggle = 0;
    let mut move_input;
    loop {
        {
            let status_mutex = status_main.lock().unwrap();
            status = *status_mutex;
        }
        if status == true {
            FreeRtos::delay_ms(100);
        } else {
            // we are using thread::sleep here to make sure the watchdog isn't triggered
            FreeRtos::delay_ms(100);
            if let Level::High = move_input_pin.get_level() {
                move_input = 1;
            } else {
                move_input = 0;
            }
            println!("{}", move_input);
            if move_input == 1 && toggle == 0 {
                toggle = 1;
                println!("High");
                socket.send(&[1]).expect("couldn't send high message");
            } else if move_input == 0 && toggle == 1 {
                toggle = 0;
                println!("Low");
                socket.send(&[0]).expect("couldn't send low message");
            }
        }
    }
}

fn http_server(ip: Ip) -> Result<(EspHttpServer, Arc<Mutex<bool>>)> {
    let server_config = Configuration::default();
    let mut server = EspHttpServer::new(&server_config)?;
    server.fn_handler("/", Method::Get, move |request| {
        let html = index_html(format!(
            "own: {}, server: {}",
            ip.own.clone(),
            ip.server.clone()
        ));
        request.into_ok_response()?.write_all(html.as_bytes())?;
        Ok(())
    })?;

    let status_fn = Arc::new(Mutex::new(false));
    let status_thread1 = Arc::clone(&status_fn);
    let status_thread2 = Arc::clone(&status_fn);

    server
        .fn_handler("/stop", Method::Get, move |request| {
            let mut status = status_thread1.lock().unwrap();
            *status = true;
            let html = index_html(format!("status: {}", *status));
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok(())
        })?
        .fn_handler("/start", Method::Get, move |request| {
            let mut status = status_thread2.lock().unwrap();
            *status = false;
            let html = index_html(format!("status: {}", *status));
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok(())
        })?;
    let status_main = Arc::clone(&status_fn);

    Ok((server, status_main))
}
fn templated(content: impl AsRef<str>) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <title>esp-rs web server</title>
    </head>
    <body>
        {}
    </body>
</html>
"#,
        content.as_ref()
    )
}

fn index_html(content: String) -> String {
    templated(content)
}

//from https://medium.com/@rajeshpachaikani/connect-esp32-to-wifi-with-rust-7d12532f539b
fn wifi_simple(modem: Modem, sys_loop: EspEventLoop<System>) -> Result<EspWifi<'static>> {
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi_driver = EspWifi::new(modem, sys_loop, Some(nvs))?;

    wifi_driver.set_configuration(&WifiConfig::Client(ClientConfiguration {
        ssid: SSID.into(),
        password: PASS.into(),
        ..Default::default()
    }))?;

    wifi_driver.start()?;
    wifi_driver.connect()?;
    while !wifi_driver.is_connected()? {
        let config = wifi_driver.get_configuration()?;
        println!("Waiting for station {:?}", config);
    }
    println!("Should be connected now");
    for _ in 0..3 {
        println!("IP info: {:?}", wifi_driver.sta_netif().get_ip_info()?);
        FreeRtos::delay_ms(1000);
    }
    Ok(wifi_driver)
}
//from https://github.com/ivmarkov/rust-esp32-std-demo/blob/main/src/main.rs
#[cfg(not(feature = "qemu"))]
fn wifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    let mut wifi = Box::new(EspWifi::new(
        modem,
        sysloop.clone(),
        EspDefaultNvsPartition::take().ok(),
    )?);

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

    wifi.set_configuration(&WifiConfig::Mixed(
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
//from https://github.com/ivmarkov/rust-esp32-std-demo/blob/main/src/main.rs
fn ping(ip: Ipv4Addr) -> Result<()> {
    println!("About to do some pings for {:?}", ip);

    let ping_summary = EspPing::default().ping(ip, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        println!("Pinging IP {} resulted in timeouts", ip);
    }

    println!("Pinging done");

    Ok(())
}
//from https://github.com/ivmarkov/rust-esp32-std-demo/blob/main/src/main.rs
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
