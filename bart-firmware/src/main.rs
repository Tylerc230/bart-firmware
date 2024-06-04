mod wifi;
mod http;
use bart_core::*;
use anyhow::{Result, Error};
use std::sync::mpsc::Receiver;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    wifi::EspWifi,
    hal::{
        self,
        peripheral::Peripheral,
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
mod spi_driver;
use spi_driver::Ws2812;

use std::{thread, time};
use smart_leds::SmartLedsWrite;
use esp_idf_svc::timer::{EspTaskTimerService, EspTimer};
use core::time::Duration;
use std::sync::mpsc;

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
    let mut shell = AppShell::new()?;
    shell.schedule_next_fetch(0);
    shell.run_update_loop();
    Ok(())
}
type SPI<'a> = spi::SpiBusDriver<'a, SpiDriver<'a>>;
type LEDs<'a> = Ws2812<SPI<'a>>;
struct AppShell<'a> {
    leds: LEDs<'a>,
    wifi_connection: Box<EspWifi<'static>>,
    app_state: AppState,
    last_fetch_response_timer: TimerDriver<'a>,
    schedule_next_fetch_alarm: EspTimer<'a>,
    recv_fetch_response: Receiver<Result<String, Error>>
}

impl AppShell<'_> {
    fn new<'a>() -> Result<AppShell<'a>> {
        let peripherals = Peripherals::take().unwrap();
        let modem = peripherals.modem;
        let pins = peripherals.pins;
        let spi_pin = peripherals.spi2;
        let wifi_connection = Self::connect_to_wifi(modem)?;
        let leds = Self::create_leds(pins, spi_pin)?;
        let mut app_state = AppState::new();
        let last_fetch_timer = Self::create_timer(peripherals.timer00)?;
        let (rx, fetch_timer) = AppShell::create_fetch_timer()?;
        let mut shell = AppShell { leds, wifi_connection, app_state, last_fetch_response_timer: last_fetch_timer, schedule_next_fetch_alarm: fetch_timer, recv_fetch_response: rx };
        Ok(shell)
    }

    fn run_update_loop(&mut self) -> Result<(), EspError> {
        let delay = time::Duration::from_secs(3);
        loop {
            self.handle_fetch_response();
            self.render_leds()?;
            thread::sleep(delay);
        }
    }

    fn render_leds(&mut self) -> Result<(), EspError>{
        let request_fetch_time_microsec = self.last_fetch_response_timer.counter()?;
        let led_buffer = self.app_state.get_current_led_buffer(request_fetch_time_microsec);
        let dimmed = smart_leds::brightness(led_buffer.rgb_buffer.into_iter(), 9); 
        self.leds.write(dimmed).unwrap();
        Ok(())
    }

    fn handle_fetch_response(&mut self) {
        if let Ok(result) = self.recv_fetch_response.try_recv() {
            let next_fetch_sec = self.app_state.received_http_response(result);
            self.schedule_next_fetch(next_fetch_sec);
            self.last_fetch_response_timer.set_counter(0);
        }
    }

    fn schedule_next_fetch(&mut self, fetch_after_sec: u64) {
        log::info!("Scheduling fetch in {} seconds", fetch_after_sec);
        self.schedule_next_fetch_alarm.after(Duration::from_secs(fetch_after_sec));
    }

    fn create_leds<'a>(pins: gpio::Pins, spi_pin: spi::SPI2) -> Result<LEDs<'a>> {
        let sclk = pins.gpio12;
        let sdo = pins.gpio11;
        let sdi = pins.gpio13;
        let spi_config = spi::SpiConfig::new().baudrate(6410.kHz().into());
        let spi_driver = SpiDriver::new::<SPI2>(spi_pin, sclk, sdo, Some(sdi), &SpiDriverConfig::new())?;
        let spi_bus = spi::SpiBusDriver::new(spi_driver, &spi_config)?;
        Ok(Ws2812::new(spi_bus))
    }

    fn create_timer<'d, T: hal::timer::Timer>(timer: impl Peripheral<P = T> + 'd) -> Result<TimerDriver<'d>> {
        let config = TimerConfig::new();
        let mut timer_driver = TimerDriver::new(timer, &config)?;
        timer_driver.set_counter(0_u64)?;
        timer_driver.enable(true)?;
        Ok(timer_driver)
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

    fn create_fetch_timer<'a>() -> Result<(Receiver<Result<String, Error>>, EspTimer<'a>), EspError> {
        let (tx, rx) = mpsc::channel();
        let timer_service = EspTaskTimerService::new()?;
        unsafe {
            let timer = timer_service.timer_nonstatic(move || {
                let result = http::get("https://api.bart.gov/api/etd.aspx?cmd=etd&orig=ROCK&key=MW9S-E7SL-26DU-VV8V&json=y");
                tx.send(result).unwrap();
            })?;
            Ok((rx, timer))
        }
    }

}



