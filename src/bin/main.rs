#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;

use esp_hal::clock::CpuClock;

use esp_hal::rng::Rng;
use esp_hal::timer::timg::TimerGroup;
use panic_rtt_target as _;

use trying_esp_hal_again as lib;
use trying_esp_hal_again::led::zone_driver;
use trying_esp_hal_again::web::{WEB_TASK_POOL_SIZE, WebApp};

extern crate alloc;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: Spawner) {
    // generator version: 0.6.0

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[unsafe(link_section = ".dram2_uninit")] size: 65536);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let sw_interrupt =
        esp_hal::interrupt::software::SoftwareInterruptControl::new(
            peripherals.SW_INTERRUPT,
        );
    esp_rtos::start(timg0.timer0, sw_interrupt.software_interrupt0);

    info!("Embassy initialized!");

    let radio_controler =
        esp_radio::init().expect("Failed to initialize Wi-Fi/BLE controller");
    let rng = Rng::new();

    let net_stack = lib::wifi::setup(
        picoserve::make_static!(
            esp_radio::Controller<'static>,
            radio_controler
        ),
        peripherals.WIFI,
        &spawner,
        rng,
    )
    .await
    .unwrap();

    spawner.must_spawn(zone_driver(
        peripherals.GPIO15.into(),
        peripherals.GPIO2.into(),
        peripherals.GPIO21.into(),
        peripherals.GPIO1.into(),
        peripherals.GPIO18.into(),
        peripherals.GPIO19.into(),
        peripherals.GPIO0.into(),
    ));

    let web_app = WebApp::default();

    for task_id in 0..WEB_TASK_POOL_SIZE {
        spawner.must_spawn(lib::web::web_task(
            task_id,
            net_stack,
            web_app.router,
            web_app.config,
        ));
    }

    defmt::info!("Web Server Stated");
    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.0.0-rc.1/examples/src/bin
}
