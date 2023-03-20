use std::io::{Read, self};
use std::net::{TcpListener, UdpSocket};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::net::ToSocketAddrs;

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

use esp_idf_sys::esp_restart;

use embedded_svc::http::Method;
use embedded_svc::io::{Read as Http_Read, Write};
use embedded_svc::ipv4::Ipv4Addr;
use embedded_svc::wifi::{
    AccessPointConfiguration, ClientConfiguration, Configuration as WifiConfig, Wifi,
};

use anyhow::{Result, Error};

#[cfg(not(feature = "qemu"))]
mod wifi_info;
#[cfg(not(feature = "qemu"))]
use wifi_info::*;

#[derive(Clone)]
struct Network {
    ip_own_target: Ipv4Addr,
    ip_own_ap: Ipv4Addr,
    ip_server_udp: Ipv4Addr,
    port_server_udp: String,
}
#[derive(PartialEq, Debug, Clone)]
enum States {
    Run,
    Pause,
    Restart,
}
#[derive(PartialEq, Debug, Clone)]
enum DebugOutput {
    High,
    Low,
    Off,
}
#[derive(Debug, Clone)]
struct Commands {
    sensor_input: bool,
    esp_output: bool,
    status: States,
    const_output: DebugOutput,
    time: u64,
    ota: bool,
}
impl Default for Commands {
    fn default() -> Self {
        Commands {
            sensor_input: false,
            esp_output: false,
            status: States::Run,
            const_output: DebugOutput::Off,
            time: 10,
            ota: false,
        }
    }
}
fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    let version = String::from("version 1.0.4");
    println!("---------------start {}---------------", version);
    println!("---------------set up gpio---------------");
    let peripherals = Peripherals::take().unwrap();
    let mut move_input_pin =
        PinDriver::input(peripherals.pins.gpio17).expect("couldn't set gpio 17 to input");
    move_input_pin
        .set_pull(Pull::Down)
        .expect("couldn't set input pin to pull down");
    let mut move_output_pin =
        PinDriver::output(peripherals.pins.gpio18).expect("couldn't set gpio 18 to input");

    println!("---------------set up wifi---------------");
    println!("connecting to SSID: {}", SSID_TARGET);
    let sysloop = EspSystemEventLoop::take().unwrap();
    #[cfg(not(feature = "qemu"))]
    let mut wifi = wifi(peripherals.modem, sysloop).expect("couldn't connect to wifi");
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
    #[cfg(not(feature = "qemu"))]
    #[cfg(esp_idf_lwip_ipv4_napt)]
    enable_napt(&mut wifi).expect("couldn't enable napt");
    #[cfg(not(feature = "qemu"))]
    let network_info = Network {
        ip_own_target: wifi.sta_netif().get_ip_info().unwrap().ip,
        ip_own_ap: wifi.ap_netif().get_ip_info().unwrap().ip,
        ip_server_udp: UDP_SERVER_IP.parse::<Ipv4Addr>().unwrap(), //server ip loxone 192.168.1.222
        port_server_udp: UDP_SERVER_PORT.into()
    };
    #[cfg(feature = "qemu")]
    let network_info = Network {
        ip_own_target: eth.netif().get_ip_info().unwrap().ip,
        ip_own_ap: "0.0.0.0".parse::<Ipv4Addr>().unwrap(),
        ip_server_udp: UDP_SERVER_IP.parse::<Ipv4Addr>().unwrap(), //server ip loxone 192.168.1.222
        port_server_udp: UDP_SERVER_PORT.into()
    };
    #[cfg(not(feature = "qemu"))]
    ping(network_info.ip_server_udp).unwrap();

    println!("---------------set up http_server---------------");
    let mut command;
    let (http_server, command_main) = http_server(network_info.clone(), version).unwrap();

    println!("---------------set up udp---------------");
    let socket =
        UdpSocket::bind(format!("{}:4002", network_info.ip_own_target)).expect("socket couldn't bind to address");
    socket
        .connect(format!("{}:{}", network_info.ip_server_udp, network_info.port_server_udp)) 
        .expect("socket connect function failed");

    println!("---------------start loop---------------");
    let mut toggle_detect: bool = false;
    let mut move_input: bool;
    let mut high_time = Instant::now();
    loop {
        let command_mutex = command_main.lock().unwrap();
        command = command_mutex.clone();
        drop(command_mutex);
        if command.ota == true {
            println!("---------------start ota---------------");
            //drop(http_server);
            ota_flash(&network_info).expect("ota failed");
        }
        if command.status == States::Pause {
            FreeRtos::delay_ms(100);
        } else if command.status == States::Run {
            FreeRtos::delay_ms(100);
            if Level::High == move_input_pin.get_level() {
                move_input = true;
            } else {
                move_input = false;
            }
            if command.const_output == DebugOutput::High {
                move_input = true;
            } else if command.const_output == DebugOutput::Low {
                move_input = false;
            }
            command_main.lock().unwrap().sensor_input = move_input;
            // println!("{}", move_input);
            let now = Instant::now();
            if move_input == true {
                high_time = Instant::now();
                if toggle_detect == false {
                    toggle_detect = true;
                    println!("High");
                    command_main.lock().unwrap().esp_output = true;
                    move_output_pin
                        .set_high()
                        .expect("couldn't send pin high message");
                    socket.send(&[1]).expect("couldn't send udp high message");
                }
            } else if toggle_detect == true && move_input == false {
                if now.duration_since(high_time) > Duration::new(command.time, 0) {
                    toggle_detect = false;
                    println!("Low");
                    command_main.lock().unwrap().esp_output = false;
                    move_output_pin
                        .set_low()
                        .expect("couldn't send pin low message");
                    socket.send(&[0]).expect("couldn't send low message");
                }
            }
        } else if command.status == States::Restart {
            unsafe {
                esp_restart();
                unreachable!("esp_restart returned");
            }
        }
    }
}
fn http_server(network_info: Network, version: String) -> Result<(EspHttpServer, Arc<Mutex<Commands>>)> {
    let command_fn = Arc::new(Mutex::new(Commands::default()));
    let command_thread0 = Arc::clone(&command_fn);
    let command_thread1 = Arc::clone(&command_fn);
    let command_thread2 = Arc::clone(&command_fn);
    let command_thread3 = Arc::clone(&command_fn);
    let command_thread4 = Arc::clone(&command_fn);
    let command_thread5 = Arc::clone(&command_fn);
    let command_thread6 = Arc::clone(&command_fn);
    let command_thread7 = Arc::clone(&command_fn);
    let command_thread8 = Arc::clone(&command_fn);

    let server_config = Configuration::default();
    let mut server = EspHttpServer::new(&server_config)?;
    server
        .fn_handler("/", Method::Get, move |request| {
            let command = command_thread0.lock().unwrap();
            let html = index_html(format!(
                r#"
                Esp32 presence detection {} <br> <br>
                own ip in target network: {}, server ip: {} <br>
                own ip in ap network: {} <br>
                UDP port: 4002 -> {} <br>
                OTA TCP port: 5003 <br>
                HTTP port: 80 <br> <br>
                sensor input: {}, esp output: {} <br> <br>
                commands:<br> 
                <form action="/pause" method="post">
                <button type="submit">Pause</button>
                </form>
                <form action="/run" method="post">
                <button type="submit">Run</button>
                <label>status: {:?}</label>
                </form>
                <form action="/restart" method="post">
                <button type="submit">Restart</button>
                </form>
                <form action="/debughigh" method="post">
                <button type="submit">DebugHigh</button>
                </form>
                <form action="/debuglow" method="post">
                <button type="submit">DebugLow</button>
                <label>debug output: {:?}</label>
                </form>
                <form action="/debugoff" method="post">
                <button type="submit">DebugOff</button>
                </form>
                <form action="/ota" method="post">
                <button type="submit">Ota</button>
                <label> ota: {}</label>
                </form>
                <form action="/submit" method="post">
                <label for"number">Enter movement follow-up time [sec]:</label>
                <input type="number" min="1" max="1800" value="{}" id="n" name="n">
                <button type="submit">Submit</button>
                </form>
            "#,
                version,
                network_info.ip_own_target,
                network_info.ip_server_udp,
                network_info.ip_own_ap,
                network_info.port_server_udp,
                command.sensor_input,
                command.esp_output,
                command.status,
                command.const_output,
                command.ota,
                command.time,
            ));
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok(())
        })?
        .fn_handler("/pause", Method::Post, move |request| {
            let mut command = command_thread1.lock().unwrap();
            command.status = States::Pause;
            let html = action_html(format!("status: {:?}", command.status));
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok(())
        })?
        .fn_handler("/run", Method::Post, move |request| {
            let mut command = command_thread2.lock().unwrap();
            command.status = States::Run;
            let html = action_html(format!("status: {:?}", command.status));
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok(())
        })?
        .fn_handler("/restart", Method::Post, move |request| {
            let mut command = command_thread3.lock().unwrap();
            command.status = States::Restart;
            let html = action_html(format!("status: {:?}", command.status));
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok(())
        })?
        .fn_handler("/debughigh", Method::Post, move |request| {
            let mut command = command_thread4.lock().unwrap();
            command.const_output = DebugOutput::High;
            let html = action_html(format!("status: debug {:?}", command.const_output));
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok(())
        })?
        .fn_handler("/debuglow", Method::Post, move |request| {
            let mut command = command_thread5.lock().unwrap();
            command.const_output = DebugOutput::Low;
            let html = action_html(format!("status: debug {:?}", command.const_output));
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok(())
        })?
        .fn_handler("/debugoff", Method::Post, move |request| {
            let mut command = command_thread6.lock().unwrap();
            command.const_output = DebugOutput::Off;
            let html = action_html(format!("status: debug {:?}", command.const_output));
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok(())
        })?
        .fn_handler("/ota", Method::Post, move |request| {
            let mut command = command_thread7.lock().unwrap();
            command.ota = true;
            let html = action_html(format!("status: ota {:?}", command.ota));
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok(())
        })?
        .fn_handler("/submit", Method::Post, move |mut request| {
            let mut command = command_thread8.lock().unwrap();
            let mut buf = [0; 8];
            request.read(&mut buf)?;
            command.time = buf[2..]
                .iter()
                .filter(|&x| *x != 0)
                .map(|x| *x as char)
                .collect::<String>()
                .parse::<u64>()
                .unwrap();
            let html = action_html(format!("movement follow-up time = {:?}", command.time));
            request.into_ok_response()?.write_all(html.as_bytes())?;
            Ok(())
        })?
        .fn_handler("/favicon.ico", Method::Get, move |request| {
            const ICON: &[u8] = include_bytes!(
                "/home/ronny/Documents/code/rust/esp_move_detect/rust-bin/favicon.ico"
            );
            request.into_ok_response()?.write_all(ICON)?;
            Ok(())
        })?;

    let command_main = Arc::clone(&command_fn);

    Ok((server, command_main))
}

fn templated_action(content: impl AsRef<str>) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
        <meta http-equiv="refresh" content="1;url=/" />
    </head>
    <body>
        {}
    </body>
</html>
"#,
        content.as_ref()
    )
}
fn templated_index(content: impl AsRef<str>) -> String {
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
    templated_index(content)
}
fn action_html(content: String) -> String {
    templated_action(content)
}
//from https://github.com/ivmarkov/rust-esp32-std-demo/blob/main/src/main.rs
#[cfg(not(feature = "qemu"))]
fn wifi(
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    use embedded_svc::wifi::AuthMethod;

    let mut wifi = Box::new(EspWifi::new(
        modem,
        sysloop.clone(),
        EspDefaultNvsPartition::take().ok(),
    )?);

    println!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == SSID_TARGET);

    let channel = if let Some(ours) = ours {
        println!(
            "Found configured access point {} on channel {}",
            SSID_TARGET, ours.channel
        );
        Some(ours.channel)
    } else {
        println!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            SSID_TARGET
        );
        None
    };
    #[cfg(not(feature = "qemu"))]
    #[cfg(esp_idf_lwip_ipv4_napt)]
    wifi.set_configuration(&WifiConfig::Mixed(
        ClientConfiguration {
            ssid: SSID_TARGET.into(),
            password: PASS_TARGET.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: SSID_AP.into(),
            password: PASS_AP.into(),
            channel: channel.unwrap_or(1),
            auth_method: AuthMethod::WPA2Personal,
            ..Default::default()
        },
    ))?;

    #[cfg (not(esp_idf_lwip_ipv4_napt))]
    wifi.set_configuration(&WifiConfig::Client(
        ClientConfiguration {
            ssid: SSID_TARGET.into(),
            password: PASS_TARGET.into(),
            channel,
            ..Default::default()
        }
    ))?;

    wifi.start()?;

    println!("Starting wifi...");

    if !WifiWait::new(&sysloop)?
        .wait_with_timeout(Duration::from_secs(20), || wifi.is_started().unwrap())
    {
        println!("Wifi did not start");
    }

    println!("Connecting wifi...");
    println!("AP Info: {:?}", wifi.ap_netif().get_ip_info().unwrap());
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

    Ok(wifi)
}
//from https://github.com/ivmarkov/rust-esp32-std-demo/blob/main/src/main.rs
#[cfg(not(feature = "qemu"))]
fn ping(ip: Ipv4Addr) -> Result<()> {
    println!("About to do some pings for {:?}", ip);

    let ping_summary = EspPing::default().ping(ip, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        println!("Pinging IP {} resulted in timeouts", ip);
    }

    println!("Pinging done");

    Ok(())
}
#[cfg(not(feature = "qemu"))]
#[cfg(esp_idf_lwip_ipv4_napt)]
fn enable_napt(wifi: &mut EspWifi) -> Result<()> {
    wifi.ap_netif_mut().enable_napt(true);

    println!("NAPT enabled on the WiFi SoftAP!");

    Ok(())
}
//from https://github.com/ivmarkov/rust-esp32-std-demo/blob/main/src/main.rs
#[cfg(any(feature = "qemu"))]
fn eth_configure(
    sysloop: &EspSystemEventLoop,
    mut eth: Box<esp_idf_svc::eth::EspEth<'static>>,
) -> Result<Box<esp_idf_svc::eth::EspEth<'static>>> {
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
//from https://github.com/faern/esp-ota/tree/e73cf6f3959ab41ecdb459851e878946ebbb7363/
fn ota_flash(network_info: &Network) -> Result<()> {
    // Finds the next suitable OTA partition and erases it
    let mut ota = esp_ota::OtaUpdate::begin()?;
    //download new app
    let listener_target = TcpListener::bind(format!("{}:5003", network_info.ip_own_target))?;
    listener_target.set_nonblocking(true).expect("Cannot set non-blocking for target network");
    let listener_ap = TcpListener::bind(format!("{}:5003", network_info.ip_own_ap))?;
    listener_ap.set_nonblocking(true).expect("Cannot set non-blocking for ap network");
    let mut app_chunk = [0; 4096];
    let mut eof = 1;
    let mut downloaded_bytes = 0;
    let mut try_count = 0;
//maybe use threads???
    for stream in listener_target.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Connection established: {:?}", stream);
                while eof != 0 {
                    FreeRtos::delay_ms(11);
                    eof = stream.read(&mut app_chunk[..])?;
                    if eof != 0 {
                        downloaded_bytes += app_chunk[0..eof].len();
                        println!("{}", downloaded_bytes);
                        ota.write(&app_chunk[0..eof])?;
                    }
                }
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
    FreeRtos::delay_ms(150);
                try_count += 1;
                if try_count > 80 {
                    break;
                }
                else{
                    continue;
                }
            },
            Err(e) => panic!("encountered IO error: {}", e),
        }
    }
    for stream in listener_ap.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("Connection established: {:?}", stream);
                while eof != 0 {
                    FreeRtos::delay_ms(11);
                    eof = stream.read(&mut app_chunk[..])?;
                    if eof != 0 {
                        downloaded_bytes += app_chunk[0..eof].len();
                        println!("{}", downloaded_bytes);
                        ota.write(&app_chunk[0..eof])?;
                    }
                }
            },
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
    FreeRtos::delay_ms(150);
                try_count += 1;
                if try_count > 80 {
                    break;
                }
                else{
                    continue;
                }
            },
            Err(e) => panic!("encountered IO error: {}", e),
        }
    }

    FreeRtos::delay_ms(11);
    // Performs validation of the newly written app image and completes the OTA update.
    let mut completed_ota = ota.finalize()?;
    FreeRtos::delay_ms(11);
    // Sets the newly written to partition as the next partition to boot from.
    completed_ota.set_as_boot_partition()?;
    // Restarts the CPU, booting into the newly written app.
    println!("----------Restart----------");
    completed_ota.restart();
}
