#![allow(unused)]

use core::{
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4},
    str::FromStr,
};

use defmt::{info, trace};
use edge_dhcp::io;
use edge_nal::{UdpBind, UdpSocket};
use edge_nal_embassy::{Udp, UdpBuffers};
use embassy_executor;
use embassy_net::Ipv4Cidr;
use embassy_time::{Duration, Timer};
use esp_radio::wifi::{
    AccessPointConfig, ModeConfig, WifiApState, WifiController, WifiDevice,
    WifiEvent, ap_state,
};

use crate::{GATEWAY_IP, WIFI_SSID, mk_static};

pub async fn setup(
    esp_radio_controler: &'static esp_radio::Controller<'static>,
    esp_wifi_perph: esp_hal::peripherals::WIFI<'static>,
    spawner: &embassy_executor::Spawner,
    rng: esp_hal::rng::Rng,
) -> anyhow::Result<embassy_net::Stack<'static>> {
    let net_seed: u64 = rng.random() as u64 | (rng.random() as u64) << 32;
    let (wifi_controler, wifi_interfaces) = esp_radio::wifi::new(
        esp_radio_controler,
        esp_wifi_perph,
        Default::default(),
    )
    .expect("Failed To Create Wifi Interfaces, Controller");
    let interface = wifi_interfaces.ap;
    let gateway_ip =
        Ipv4Addr::from_str(GATEWAY_IP).expect("Unable To Parse Ip");
    let device_config =
        embassy_net::Config::ipv4_static(embassy_net::StaticConfigV4 {
            address: Ipv4Cidr::new(gateway_ip, 24),
            gateway: Some(gateway_ip),
            dns_servers: Default::default(),
        });

    // INFO: Init Embassy_net
    let (net_stack, net_runner) = embassy_net::new(
        interface,
        device_config,
        mk_static!(
            embassy_net::StackResources::<5>,
            embassy_net::StackResources::<5>::new()
        ),
        net_seed,
    );

    spawner.spawn(connection_task(wifi_controler)).ok();
    spawner.spawn(run_captive(net_stack, gateway_ip)).ok();
    spawner.spawn(run_dhcp(net_stack, gateway_ip)).ok();
    spawner.spawn(net_task(net_runner)).ok();

    wait_for_connection(net_stack).await;

    Ok(net_stack)
}

async fn wait_for_connection(stack: embassy_net::Stack<'_>) {
    defmt::info!("Waiting for link to be up");
    loop {
        if stack.is_link_up() {
            break;
        }
        Timer::after(Duration::from_millis(500)).await;
    }

    defmt::info!(
        "Connect to the AP `esp-wifi` and point your browser to http://{:?}/",
        GATEWAY_IP
    );
    while !stack.is_config_up() {
        Timer::after(Duration::from_millis(100)).await
    }
    stack
        .config_v4()
        .inspect(|c| defmt::info!("ipv4 config: {:?}", c));
}

#[embassy_executor::task]
async fn run_captive(
    net_stack: embassy_net::Stack<'static>,
    gateway_ip: Ipv4Addr,
) {
    let udp_buffers: edge_nal_embassy::UdpBuffers<5, 1024, 1024, 5> =
        edge_nal_embassy::UdpBuffers::new();
    let udp_socket = edge_nal_embassy::Udp::new(net_stack, &udp_buffers);

    let mut tx_buf = [0; 1500];
    let mut rx_buf = [0; 1500];

    edge_captive::io::run(
        &udp_socket,
        SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 53),
        &mut tx_buf,
        &mut rx_buf,
        gateway_ip,
        core::time::Duration::from_secs(60),
    )
    .await
    .unwrap();
}

#[embassy_executor::task]
async fn run_dhcp(
    net_stack: embassy_net::Stack<'static>,
    gateway_ip: Ipv4Addr,
) {
    let mut buffer = [0u8; 1500];
    let mut gateway_buffer = [Ipv4Addr::UNSPECIFIED];
    let udp_buffers = UdpBuffers::<3, 1024, 1024, 10>::new();

    let udp_socket = Udp::new(net_stack, &udp_buffers);
    let mut udp_socket = udp_socket
        .bind(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::UNSPECIFIED,
            edge_dhcp::io::DEFAULT_SERVER_PORT,
        )))
        .await
        .expect("Socket Couldn't Be Bound");

    let mut dhcp_config = edge_dhcp::server::ServerOptions::new(
        gateway_ip,
        Some(&mut gateway_buffer),
    );

    let dns_servers = [gateway_ip];
    let captive_url = "http://192.168.2.1:80";
    dhcp_config.dns = &dns_servers;
    dhcp_config.captive_url = Some(captive_url);

    defmt::info!("{:?}", dhcp_config);

    loop {
        _ = io::server::run(
            &mut edge_dhcp::server::Server::<_, 64>::new_with_et(gateway_ip),
            &dhcp_config,
            &mut udp_socket,
            &mut buffer,
        )
        .await
        .inspect_err(|e| defmt::warn!("DHCP server error: {:?}", e));
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task]
async fn connection_task(mut controller: WifiController<'static>) {
    defmt::info!("start connection task");
    defmt::info!("Device capabilities: {:?}", controller.capabilities());
    loop {
        match esp_radio::wifi::ap_state() {
            WifiApState::Started => {
                // wait until we're no longer connected
                controller.wait_for_event(WifiEvent::ApStop).await;
                Timer::after(Duration::from_millis(5000)).await
            }
            _ => {}
        }
        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = ModeConfig::AccessPoint(
                AccessPointConfig::default().with_ssid(WIFI_SSID.into()),
            );
            controller.set_config(&client_config).unwrap();
            defmt::info!("Starting wifi");
            controller.start_async().await.unwrap();
            defmt::info!("Wifi started!");
        }
    }
}

#[embassy_executor::task]
async fn net_task(
    mut net_runner: embassy_net::Runner<'static, WifiDevice<'static>>,
) {
    net_runner.run().await
}
