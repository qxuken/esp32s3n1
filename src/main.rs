use anyhow::{bail, Result};
use app_state::AppState;
use colors_transform::Rgb;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{delay::FreeRtos, prelude::*},
};
use rgb_led::WS2812RMT;

mod app_state;
mod rgb_led;
pub mod utils;
mod web;
mod wifi;

#[derive(Debug)]
#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    log::info!("{:?}", CONFIG);

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;

    let rgb_led = WS2812RMT::new(peripherals.pins.gpio48, peripherals.rmt.channel0)?;
    let app_state = AppState::try_new(rgb_led, Rgb::from(0.0, 2.0, 10.0))?.shared();

    log::info!("{:?}", app_state);

    let _wifi = match wifi::wifi(
        CONFIG.wifi_ssid,
        CONFIG.wifi_psk,
        peripherals.modem,
        sysloop,
    ) {
        Ok(inner) => inner,
        Err(err) => {
            app_state
                .clone()
                .lock()
                .unwrap()
                .set_rgb(Rgb::from(150.0, 0.0, 0.0))?;
            bail!("Could not connect to Wi-Fi network: {:?}", err)
        }
    };
    let _server = web::server(app_state.clone())?;

    app_state
        .clone()
        .lock()
        .unwrap()
        .set_rgb(Rgb::from(0.0, 5.0, 0.0))?;
    loop {
        FreeRtos::delay_ms(1000)
    }
}
