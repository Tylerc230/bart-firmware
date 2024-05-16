mod wifi;
mod http;
use bart_core::*;
use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    wifi::EspWifi,
    hal::{
        self,
        sys::EspError,
        prelude::Peripherals, 
        spi::{
            self, SPI2, SpiDriver, SpiDriverConfig
        },
        gpio,
        units::FromValueType,
        timer::config::Config as TimerConfig,
        timer::TimerDriver
    },
};
use ws2812_spi::Ws2812;

use std::{thread, time};
use smart_leds::SmartLedsWrite;

#[toml_cfg::toml_config]
pub struct Config {
    #[default("Wokwi-GUEST")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}
fn main() -> Result<()>{
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let peripherals = Peripherals::take().unwrap();

    let mut app_state = AppState::new();
    let modem = peripherals.modem;
    let _wifi = connect_to_wifi(modem);

    fetch_schedule(&mut app_state);
    let timer = create_timer(peripherals.timer00);
    let pins = peripherals.pins;
    let spi_pin = peripherals.spi2;
    let _th = thread::spawn(move || {
        let _ = render_leds(pins, spi_pin, &timer, &app_state);
    });
    Ok(())
}

fn render_leds(pins: gpio::Pins, spi_pin: spi::SPI2, timer: &TimerDriver, app_state: &AppState) -> Result<(), EspError> {
    let spi_bus = create_spi_bus(pins, spi_pin)?;
    let mut leds = Ws2812::new(spi_bus); 
    let delay = time::Duration::from_secs(3);
    loop {
        let request_fetch_time_microsec = timer.counter().unwrap();
        let led_buffer = app_state.get_current_led_buffer(request_fetch_time_microsec);
        let dimmed = smart_leds::brightness(led_buffer.rgb_buffer.into_iter(), 9); 
        leds.write(dimmed).unwrap();
        thread::sleep(delay);
    }
}

fn fetch_schedule(app_state: &mut AppState) {
    let result = http::get("https://api.bart.gov/api/etd.aspx?cmd=etd&orig=ROCK&key=MW9S-E7SL-26DU-VV8V&json=y");
    match result {
        Ok(response) => {
            log::info!("Fetch Success");
            app_state.received_http_response(response)
        },
        Err(error) => log::error!("Fetch failed {:?}", error),
    }
}

fn create_spi_bus<'a>(pins: gpio::Pins, spi_pin: spi::SPI2) -> Result<spi::SpiBusDriver<'a, SpiDriver<'a>>, EspError> {
    let sclk = pins.gpio12;
    let sdo = pins.gpio11;
    let sdi = pins.gpio13;
    let spi_config = spi::SpiConfig::new().baudrate(3.MHz().into());
    let spi_driver = SpiDriver::new::<SPI2>(spi_pin, sclk, sdo, Some(sdi), &SpiDriverConfig::new())?;
    spi::SpiBusDriver::new(spi_driver, &spi_config)
}

fn connect_to_wifi(modem: impl hal::peripheral::Peripheral<P = hal::modem::Modem> + 'static) -> Result<Box<EspWifi<'static>>> {
    let app_config = CONFIG;
    let sysloop = EspSystemEventLoop::take().unwrap();
    wifi::wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        modem,
        sysloop,
    )
}

fn create_timer<'d>(timer00: hal::timer::TIMER00) -> TimerDriver<'d> {
    let config = TimerConfig::new();
    let mut timer1 = TimerDriver::new(timer00, &config).unwrap();
    timer1.set_counter(0_u64).unwrap();
    timer1.enable(true).unwrap();
    timer1
}

