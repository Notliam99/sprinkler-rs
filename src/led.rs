use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal,
};

use esp_hal::gpio::{AnyPin, Level, Output, OutputConfig};

#[derive(serde::Deserialize, serde::Serialize, defmt::Format, Clone, Copy)]
pub enum ZoneState {
    On,
    Off,
}

impl From<ZoneState> for bool {
    fn from(value: ZoneState) -> Self {
        match value {
            ZoneState::On => true,
            ZoneState::Off => false,
        }
    }
}

impl ZoneState {
    fn set_output(self, output_driver: &mut Output) {
        match self {
            Self::On => output_driver.set_high(),
            Self::Off => output_driver.set_low(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, defmt::Format, Clone, Copy)]
pub struct Zones {
    #[cfg(feature = "debug-led")]
    debug_led: ZoneState,
    #[cfg(feature = "zone-1")]
    zone_1: ZoneState,
    #[cfg(feature = "zone-2")]
    zone_2: ZoneState,
    #[cfg(feature = "zone-3")]
    zone_3: ZoneState,
    #[cfg(feature = "zone-4")]
    zone_4: ZoneState,
    #[cfg(feature = "zone-5")]
    zone_5: ZoneState,
    #[cfg(feature = "zone-6")]
    zone_6: ZoneState,
}

impl Default for Zones {
    fn default() -> Self {
        Self {
            #[cfg(feature = "debug-led")]
            debug_led: ZoneState::Off,
            #[cfg(feature = "zone-1")]
            zone_1: ZoneState::Off,
            #[cfg(feature = "zone-2")]
            zone_2: ZoneState::Off,
            #[cfg(feature = "zone-3")]
            zone_3: ZoneState::Off,
            #[cfg(feature = "zone-4")]
            zone_4: ZoneState::Off,
            #[cfg(feature = "zone-5")]
            zone_5: ZoneState::Off,
            #[cfg(feature = "zone-6")]
            zone_6: ZoneState::Off,
        }
    }
}
pub static LED_STATE: Signal<CriticalSectionRawMutex, Zones> = Signal::new();

#[embassy_executor::task]
pub async fn zone_driver(
    #[cfg(feature = "debug-led")] debug_led_pin: AnyPin<'static>,
    #[cfg(feature = "zone-1")] zone_1_pin: AnyPin<'static>,
    #[cfg(feature = "zone-2")] zone_2_pin: AnyPin<'static>,
    #[cfg(feature = "zone-3")] zone_3_pin: AnyPin<'static>,
    #[cfg(feature = "zone-4")] zone_4_pin: AnyPin<'static>,
    #[cfg(feature = "zone-5")] zone_5_pin: AnyPin<'static>,
    #[cfg(feature = "zone-6")] zone_6_pin: AnyPin<'static>,
) {
    #[cfg(feature = "debug-led")]
    let mut debug_led_driver =
        Output::new(debug_led_pin, Level::Low, OutputConfig::default());

    #[cfg(feature = "zone-1")]
    let mut zone_1_driver =
        Output::new(zone_1_pin, Level::Low, OutputConfig::default());

    #[cfg(feature = "zone-2")]
    let mut zone_2_driver =
        Output::new(zone_2_pin, Level::Low, OutputConfig::default());

    #[cfg(feature = "zone-3")]
    let mut zone_3_driver =
        Output::new(zone_3_pin, Level::Low, OutputConfig::default());

    #[cfg(feature = "zone-4")]
    let mut zone_4_driver =
        Output::new(zone_4_pin, Level::Low, OutputConfig::default());

    #[cfg(feature = "zone-5")]
    let mut zone_5_driver =
        Output::new(zone_5_pin, Level::Low, OutputConfig::default());

    #[cfg(feature = "zone-6")]
    let mut zone_6_driver =
        Output::new(zone_6_pin, Level::Low, OutputConfig::default());

    // TODO: fix blocking nature of this function or implement a check if changed.
    loop {
        let signal = LED_STATE.wait().await;
        defmt::info!("signaled in main loop");

        #[cfg(feature = "debug-led")]
        signal.debug_led.set_output(&mut debug_led_driver);

        #[cfg(feature = "zone-1")]
        signal.zone_1.set_output(&mut zone_1_driver);

        #[cfg(feature = "zone-2")]
        signal.zone_2.set_output(&mut zone_2_driver);

        #[cfg(feature = "zone-3")]
        signal.zone_3.set_output(&mut zone_3_driver);

        #[cfg(feature = "zone-4")]
        signal.zone_4.set_output(&mut zone_4_driver);

        #[cfg(feature = "zone-5")]
        signal.zone_5.set_output(&mut zone_5_driver);

        #[cfg(feature = "zone-6")]
        signal.zone_6.set_output(&mut zone_6_driver);
    }
}
