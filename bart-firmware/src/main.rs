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
        task::thread::ThreadSpawnConfiguration,
        peripheral::Peripheral,
        sys::EspError,
        prelude::Peripherals, 
        spi::{
            self, SPI2, SpiDriver, SpiDriverConfig
        },
        gpio,

        cpu::Core,
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

    let (led_tx, led_rx) = mpsc::channel::<LEDIter>();
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let spi_pin = peripherals.spi2;

    let modem = peripherals.modem;
    let timer00 = peripherals.timer00;
    let mut shell = AppShell::new(led_tx, modem, timer00)?;
    shell.schedule_next_fetch(0);
    log::info!("Main core: {:?}", esp_idf_svc::hal::cpu::core());

    start_render_thread(pins, spi_pin, led_rx);
    shell.run_update_loop();
    Ok(())
}
type SPI<'a> = spi::SpiBusDriver<'a, SpiDriver<'a>>;
type LEDs<'a> = Ws2812<SPI<'a>>;
type LEDIter = smart_leds::Brightness<core::array::IntoIter<smart_leds::RGB8, 44>>;
enum AppShellCommand {
    FetchSchedule,
    RenderLEDs
}
struct AppShell<'a> {
    wifi_connection: Box<EspWifi<'static>>,
    app_state: AppState,
    last_fetch_response_timer: TimerDriver<'a>,
    led_tx: mpsc::Sender<LEDIter>,
    recv_fetch_response: Receiver<AppShellCommand>
}

impl AppShell<'_> 
{
    fn new<'a, T: hal::timer::Timer>(led_tx: mpsc::Sender<LEDIter>, modem: impl hal::peripheral::Peripheral<P = hal::modem::Modem> + 'static, timer00: impl Peripheral<P = T> + 'a)-> Result<AppShell<'a>> {
        let wifi_connection = Self::connect_to_wifi(modem)?;
        let mut app_state = AppState::new();
        let (last_fetch_timer, rx2) = Self::create_timer(timer00)?;
        let mut shell = AppShell { wifi_connection, app_state, last_fetch_response_timer: last_fetch_timer, recv_fetch_response: rx2, led_tx };
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
        self.led_tx.send(dimmed);
        Ok(())
    }

    fn handle_fetch_response(&mut self) {
        if let Ok(command) = self.recv_fetch_response.try_recv() {
            match command {
                AppShellCommand::FetchSchedule => {
                    let result = http::get("https://api.bart.gov/api/etd.aspx?cmd=etd&orig=ROCK&key=MW9S-E7SL-26DU-VV8V&json=y");
                    let next_fetch_sec = self.app_state.received_http_response(result);
                    self.schedule_next_fetch(next_fetch_sec);
                }
                AppShellCommand::RenderLEDs => {

                }
            }
        }
    }

    fn schedule_next_fetch(&mut self, fetch_after_sec: u64) {
        log::info!("Scheduling fetch in {} seconds", fetch_after_sec);
        self.last_fetch_response_timer.set_alarm(self.last_fetch_response_timer.tick_hz() * fetch_after_sec);
        self.last_fetch_response_timer.enable_alarm(true);
        self.last_fetch_response_timer.set_counter(0);
    }


    fn create_timer<'d, T: hal::timer::Timer>(timer: impl Peripheral<P = T> + 'd) -> Result<(TimerDriver<'d>, Receiver<AppShellCommand>)> {
        let (tx, rx) = mpsc::channel();
        let config = TimerConfig::new();
        let mut timer_driver = TimerDriver::new(timer, &config)?;
        unsafe {
            timer_driver.subscribe(move || {
                tx.send(AppShellCommand::FetchSchedule).unwrap();
            });
        }
        timer_driver.set_counter(0_u64)?;
        timer_driver.enable_interrupt()?;
        timer_driver.enable(true)?;
        Ok((timer_driver, rx))
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
}

fn start_render_thread(pins: gpio::Pins, spi_pin: spi::SPI2, led_rx: mpsc::Receiver<LEDIter>) {
    let config = ThreadSpawnConfiguration {
        name: Some(b"MyThread\0"),
        stack_size: 4096,
        priority: 5,
        inherit: false,
        pin_to_core: Some(Core::Core1),
    };

    config.set().unwrap();

    std::thread::Builder::new()
        .name("MyThread".to_string())
        .stack_size(4096)
        .spawn(move || {
            let mut leds = LEDOutput::<'_>::new(pins, spi_pin, led_rx).unwrap();
            leds.render_loop();
        })
        .unwrap();
}

struct LEDOutput<'a> {
    leds: LEDs<'a>,
    led_rx: mpsc::Receiver<LEDIter>
}

impl<'a> LEDOutput<'a> {
    fn new(pins: gpio::Pins, spi_pin: spi::SPI2, led_rx: mpsc::Receiver<LEDIter>) -> Result<LEDOutput<'a>> {
        let sclk = pins.gpio12;
        let sdo = pins.gpio11;
        let sdi = pins.gpio13;
        let spi_config = spi::SpiConfig::new().baudrate(6410.kHz().into());
        let spi_driver = SpiDriver::new::<SPI2>(spi_pin, sclk, sdo, Some(sdi), &SpiDriverConfig::new())?;
        let spi_bus = spi::SpiBusDriver::new(spi_driver, &spi_config)?;
        let leds = Ws2812::new(spi_bus);
        Ok(LEDOutput { leds, led_rx })
    }

    fn render_loop(&mut self) {
        for led_buffer in &self.led_rx {
            self.leds.write(led_buffer).unwrap();
        }
    }
}



