use std::num::NonZeroU32;

use anyhow::Result;
use app_state::AppState;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        delay,
        gpio::{InterruptType, Level, PinDriver, Pull},
        prelude::*,
        task::notification::Notification,
    },
};

mod app_state;
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

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;

    let app_state = AppState::default().shared();

    log::info!("{:?}", CONFIG);
    let _wifi = wifi::wifi(
        CONFIG.wifi_ssid,
        CONFIG.wifi_psk,
        peripherals.modem,
        sysloop,
    )?;

    let notification = Notification::new();
    let notifier = notification.notifier();

    let _server = web::server(app_state.clone(), notifier.clone())?;

    let mut led = PinDriver::input_output(peripherals.pins.gpio5)?;

    let mut button = PinDriver::input(peripherals.pins.gpio7)?;
    button.set_pull(Pull::Up)?;
    button.set_interrupt_type(InterruptType::PosEdge)?;

    unsafe {
        let notifier = notifier.clone();
        button.subscribe(move || {
            notifier.notify_and_yield(NonZeroU32::new(3).unwrap());
        })?;
    }

    loop {
        button.enable_interrupt()?;

        let prev_level = app_state.clone().lock().unwrap().led;

        let current_level = notification
            .wait(delay::BLOCK)
            .map(|value| {
                log::info!("Notification value is {:?}", value);
                match value.get() {
                    1 => Level::Low,
                    2 => Level::High,
                    3 => !prev_level,
                    _ => Level::Low,
                }
            })
            .unwrap_or(prev_level);

        app_state.clone().lock().unwrap().led = current_level;

        if led.get_level() != current_level {
            led.set_level(current_level)?;
            log::info!(
                "led changed to {:?} state",
                if led.is_set_high() { "on" } else { "off" }
            );
        }
    }
}
