mod wifi;
mod http;
use bart_core::*;
use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    wifi::EspWifi,
    hal::{
        self,
        task::thread::ThreadSpawnConfiguration,
        task::queue::Queue,
        peripheral::Peripheral,
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

use smart_leds::SmartLedsWrite;
use std::sync::mpsc;

type SPI<'a> = spi::SpiBusDriver<'a, SpiDriver<'a>>;
type LEDs<'a> = Ws2812<SPI<'a>>;
type LEDIter = smart_leds::Brightness<core::array::IntoIter<smart_leds::RGB8, 44>>;

fn main() -> Result<()>{
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    log::info!("Main core: {:?}", esp_idf_svc::hal::cpu::core());

    let (led_tx, led_rx) = mpsc::channel::<LEDIter>();
    let peripherals = Peripherals::take().unwrap();
    let pins = peripherals.pins;
    let spi_pin = peripherals.spi2;
    start_render_thread(pins, spi_pin, led_rx);

    let modem = peripherals.modem;
    let timer00 = peripherals.timer00;
    let timer01 = peripherals.timer01;
    let mut shell = AppShell::new(led_tx, timer00, timer01)?;

    shell.connect_to_wifi(modem)?;
    shell.start_render_timer()?;
    shell.schedule_next_fetch(0)?;
    shell.command_pump()?;
    Ok(())
}

#[derive(Clone, Copy)]
enum AppShellCommand {
    FetchSchedule,
    RenderLEDs
}
struct AppShell<'a> {
    wifi_connection: Option<Box<EspWifi<'static>>>,
    app_state: AppState,
    fetch_schedule_timer: TimerDriver<'a>,
    render_led_timer: TimerDriver<'a>,
    led_tx: mpsc::Sender<LEDIter>,
    command_queue: Queue<AppShellCommand>
}

impl AppShell<'_> 
{
    fn new<'a, T0: hal::timer::Timer, T1: hal::timer::Timer>(led_tx: mpsc::Sender<LEDIter>, timer00: impl Peripheral<P = T0> + 'a, timer01: impl Peripheral<P = T1> + 'a)-> Result<AppShell<'a>> {
        let app_state = AppState::new();
        let command_queue = Queue::new(20);
        let fetch_schedule_timer = Self::create_command_timer(timer00, AppShellCommand::FetchSchedule, &command_queue, false)?;
        let render_led_timer = Self::create_command_timer(timer01, AppShellCommand::RenderLEDs, &command_queue, true)?;
        let shell = AppShell { wifi_connection: None, app_state, fetch_schedule_timer, render_led_timer, led_tx, command_queue };
        Ok(shell)
    }

    fn command_pump(&mut self) -> Result<()> {
        loop {
            if let Some((command, _)) = self.command_queue.recv_front(1000) {
                match command {
                    AppShellCommand::FetchSchedule => {
                        log::debug!("Fetching schedule");
                        let result = http::get("https://api.bart.gov/api/etd.aspx?cmd=etd&orig=ROCK&key=MW9S-E7SL-26DU-VV8V&json=y");
                        let next_fetch_sec = self.app_state.received_http_response(result);
                        self.schedule_next_fetch(next_fetch_sec)?;
                    }
                    AppShellCommand::RenderLEDs => {
                        log::debug!("Render request");
                        self.render_leds()?;
                    }
                }
            } else {
                log::error!("No command");
            }
        }
        Ok(())
    }

    fn start_render_timer(&mut self) -> Result<()> {
        let fps = 1;
        self.render_led_timer.set_alarm(self.render_led_timer.tick_hz() * 1/fps)?;
        self.render_led_timer.enable_alarm(true)?;
        Ok(())
    }

    fn render_leds(&mut self) -> Result<()>{
        let request_fetch_time_microsec = self.fetch_schedule_timer.counter()?;
        let led_buffer = self.app_state.get_current_led_buffer(request_fetch_time_microsec);
        let dimmed = smart_leds::brightness(led_buffer.rgb_buffer.into_iter(), 9); 
        self.led_tx.send(dimmed)?;
        Ok(())
    }

    fn schedule_next_fetch(&mut self, fetch_after_sec: u64) -> Result<()> {
        log::info!("Scheduling fetch in {} seconds", fetch_after_sec);
        self.fetch_schedule_timer.set_alarm(self.fetch_schedule_timer.tick_hz() * fetch_after_sec)?;
        self.fetch_schedule_timer.enable_alarm(true)?;
        self.fetch_schedule_timer.set_counter(0)?;
        Ok(())
    }


    fn create_command_timer<'d, T: hal::timer::Timer>(timer: impl Peripheral<P = T> + 'd, command: AppShellCommand, command_queue: &Queue<AppShellCommand>, repeat: bool) -> Result<TimerDriver<'d>> {
        let config = TimerConfig::new().auto_reload(repeat);
        let mut timer_driver = TimerDriver::new(timer, &config)?;
        unsafe {
            let c = Queue::new_borrowed(command_queue.as_raw());
            timer_driver.subscribe(move || {
                c.send_back(command, 100).unwrap();
            })?;
        }
        timer_driver.set_counter(0_u64)?;
        timer_driver.enable_interrupt()?;
        timer_driver.enable(true)?;
        Ok(timer_driver)
    }

    fn connect_to_wifi(&mut self, modem: impl hal::peripheral::Peripheral<P = hal::modem::Modem> + 'static) -> Result<()> {
        let app_config = CONFIG;
        let sysloop = EspSystemEventLoop::take().unwrap();
        self.wifi_connection =  Some(wifi::wifi(
            app_config.wifi_ssid,
            app_config.wifi_psk,
            modem,
            sysloop,
        )?);
        Ok(())
    }
}

fn start_render_thread(pins: gpio::Pins, spi_pin: spi::SPI2, led_rx: mpsc::Receiver<LEDIter>) {
    let config = ThreadSpawnConfiguration {
        name: Some(b"RenderThread\0"),
        stack_size: 4096,
        priority: 24,
        inherit: false,
        pin_to_core: Some(Core::Core1),
    };

    config.set().unwrap();

    std::thread::Builder::new()
        .name("RenderThread".to_string())
        .stack_size(4096)
        .spawn(move || {

            log::info!("Render loop core: {:?}", esp_idf_svc::hal::cpu::core());
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
            log::debug!("Rendering leds");
            self.leds.write(led_buffer).unwrap();
        }
    }
}


#[toml_cfg::toml_config]
pub struct Config {
    #[default("Wokwi-GUEST")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}


