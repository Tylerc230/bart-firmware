mod wifi;
mod http;
mod app_state;
use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        sys::EspError,
        prelude::Peripherals, 
        spi::{
            self, SPI2, SpiDriver, SpiDriverConfig
        },
        gpio,
        units::FromValueType
    },
};
use ws2812_spi::Ws2812;

use std::{thread, time};
use smart_leds::RGB8;
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
    let app_config = CONFIG;
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let modem = peripherals.modem;
    let _wifi = wifi::wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        modem,
        sysloop,
    );
    let response = http::get("https://api.bart.gov/api/etd.aspx?cmd=etd&orig=ROCK&key=MW9S-E7SL-26DU-VV8V&json=y")?;
    let mut app_state = app_state::AppState::new();
    app_state.received_http_response(response);



    let pins = peripherals.pins;
    let spi_pin = peripherals.spi2;
    let th = thread::spawn(move || {
        flash_leds(pins, spi_pin, &app_state);
    });
    Ok(())
}

fn flash_leds(pins: gpio::Pins, spi_pin: spi::SPI2, app_state: &app_state::AppState) -> Result<(), EspError> {
    println!("This is thread {:?}", thread::current());
    let spi_bus = create_spi_bus(pins, spi_pin)?;
    let mut leds = Ws2812::new(spi_bus); 
    let delay = time::Duration::from_secs(3);



    let led_buffer = app_state.get_current_led_buffer();
    let mut data: [RGB8; 3] = [RGB8::default(); 3];
    let empty: [RGB8; 3] = [RGB8::default(); 3];
    loop {
        data[0] = RGB8 {
            r: 0,
            g: 0,
            b: 0x10,
        };
        data[1] = RGB8 {
            r: 0,
            g: 0x10,
            b: 0,
        };
        data[2] = RGB8 {
            r: 0x10,
            g: 0,
            b: 0,
        };
        leds.write(data.iter().cloned()).unwrap();
        thread::sleep(delay);
        leds.write(empty.iter().cloned()).unwrap();
        thread::sleep(delay);
    }
}

fn create_spi_bus<'a>(pins: gpio::Pins, spi_pin: spi::SPI2) -> Result<spi::SpiBusDriver<'a, SpiDriver<'a>>, EspError> {
    let sclk = pins.gpio12;
    let sdo = pins.gpio11;
    let sdi = pins.gpio13;
    let spi_config = spi::SpiConfig::new().baudrate(3.MHz().into());
    let mut spi_driver = SpiDriver::new::<SPI2>(spi_pin, sclk, sdo, Some(sdi), &SpiDriverConfig::new())?;
    spi::SpiBusDriver::new(spi_driver, &spi_config)//Crash here
}

