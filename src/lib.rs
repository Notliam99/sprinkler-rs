#![no_std]
#![feature(impl_trait_in_assoc_type)]

pub mod led;
pub mod web;
pub mod wifi;

const WIFI_CHANNEL: u16 = 11;
const WIFI_CHANNEL_SECONDARY: u16 = 1;
const GATEWAY_IP: &str = "192.168.2.1";
const WIFI_HIDDEN: bool = false;
const WIFI_SSID: &str = "Test_Wifi";
const WIFI_PASSWORD: &str = "";

#[macro_export]
macro_rules! mk_static {
    ($t:ty,$val:expr) => {{
        static STATIC_CELL: static_cell::StaticCell<$t> = static_cell::StaticCell::new();
        #[deny(unused_attributes)]
        let x = STATIC_CELL.uninit().write(($val));
        x
    }};
}
