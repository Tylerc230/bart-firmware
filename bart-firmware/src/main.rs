mod wifi;
mod http;
use bart_core::*;
use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop, hal::{
        self, cpu::Core, gpio::{self, Gpio4, InterruptType, PinDriver, Pull}, peripheral::Peripheral, prelude::Peripherals, spi::{
            self, SpiDriver, SpiDriverConfig, SPI2
        }, task::{self, queue::Queue, thread::ThreadSpawnConfiguration}, timer::{config::Config as TimerConfig, TimerDriver}, units::FromValueType
    }, wifi::EspWifi
};
mod spi_driver;
use spi_driver::Ws2812;

use smart_leds::SmartLedsWrite;
use std::{sync::mpsc, thread};

type SPI<'a> = spi::SpiBusDriver<'a, SpiDriver<'a>>;
type LEDs<'a> = Ws2812<SPI<'a>>;
type LEDIter = smart_leds::Brightness<core::array::IntoIter<smart_leds::RGB8, 44>>;
type WifiConnection = Box<EspWifi<'static>>;


static mut WIFI_CONNECTION: Option<Box<EspWifi<'static>>> = None;
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
    let motion_sensor_pin = pins.gpio4;
    let sclk = Some(pins.gpio12);
    let sdo = Some(pins.gpio11);
    let sdi = Some(pins.gpio13);

    let leds = LEDOutput::new(sclk, sdo, sdi, spi_pin, led_rx).unwrap();
    start_render_thread(leds);

    let modem = peripherals.modem;
    let timer00 = peripherals.timer00;
    let timer01 = peripherals.timer01;
    let mut shell = AppShell::new(led_tx, timer00, timer01, motion_sensor_pin)?;
    shell.start_motion_sensor()?;
    shell.start_render_timer()?;

    shell.connect_to_wifi(modem)?;
    shell.start_command_pump();
    Ok(())
}

#[derive(Clone, Copy)]
enum AppShellCommand {
    FetchSchedule,
    RenderLEDs,
    MotionSensed,
    WifiConnected
}
struct AppShell<'a> {
    app_state: AppState,
    fetch_schedule_timer: TimerDriver<'a>,
    render_led_timer: TimerDriver<'a>,
    motion_sensor: PinDriver<'a, Gpio4, hal::gpio::Input>,
    led_tx: mpsc::Sender<LEDIter>,
    command_queue: Queue<AppShellCommand>
}

impl AppShell<'_> 
{
    fn new
    <'a, T0: hal::timer::Timer, T1: hal::timer::Timer>
    (led_tx: mpsc::Sender<LEDIter>, 
        timer00: impl Peripheral<P = T0> + 'a, 
        timer01: impl Peripheral<P = T1> + 'a, 
        motion_sensor_pin: Gpio4)-> Result<AppShell<'a>> {
        let app_state = AppState::new();
        let command_queue = Queue::new(20);
        let fetch_schedule_timer = Self::create_command_timer(timer00, AppShellCommand::FetchSchedule, &command_queue, false)?;
        let render_led_timer = Self::create_command_timer(timer01, AppShellCommand::RenderLEDs, &command_queue, true)?;
        let motion_sensor = Self::create_motion_sensor(motion_sensor_pin, &command_queue)?;
        let shell = AppShell { app_state, fetch_schedule_timer, render_led_timer, motion_sensor, led_tx, command_queue };
        Ok(shell)
    }

    fn start_command_pump(&mut self) {
        loop {
            if let Some((command, _)) = self.command_queue.recv_front(1000) {
                self.handle_command(command);
            } else {
                log::error!("No command");
            }
        }
    }

    fn handle_command(&mut self, command: AppShellCommand) -> Result<()> {
        match command {
            AppShellCommand::FetchSchedule => {
                log::debug!("Fetching schedule");
                let result = http::get("https://api.bart.gov/api/etd.aspx?cmd=etd&orig=ROCK&key=MW9S-E7SL-26DU-VV8V&json=y");
                let next_fetch_sec = self.app_state.received_http_response(result);
                self.schedule_next_fetch(next_fetch_sec)?;
                self.app_state.network_activity_complete();
            }
            AppShellCommand::RenderLEDs => {
                self.render_leds()?;
            }
            AppShellCommand::WifiConnected => {
                self.schedule_next_fetch(0)?;
            }
            AppShellCommand::MotionSensed => {
                log::info!("Motion Sensed");
                self.start_motion_sensor();
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

    fn start_motion_sensor(&mut self) -> Result<()> {
        self.motion_sensor.enable_interrupt()?;
        Ok(())
    }


    fn create_command_timer<'d, T: hal::timer::Timer>(timer: impl Peripheral<P = T> + 'd, command: AppShellCommand, command_queue: &Queue<AppShellCommand>, repeat: bool) -> Result<TimerDriver<'d>> {
        let config = TimerConfig::new().auto_reload(repeat);
        let mut timer_driver = TimerDriver::new(timer, &config)?;
        unsafe {
            let c = Self::new_queue(command_queue);
            timer_driver.subscribe(move || {
                c.send_back(command, 100).unwrap();
            })?;
        }
        timer_driver.set_counter(0_u64)?;
        timer_driver.enable_interrupt()?;
        timer_driver.enable(true)?;
        Ok(timer_driver)
    }

    fn create_motion_sensor<'a>(pin: Gpio4, command_queue: &Queue<AppShellCommand>) -> Result<PinDriver<'a, Gpio4, hal::gpio::Input>> {
        let mut motion_sensor = PinDriver::input(pin)?;
        motion_sensor.set_pull(Pull::Down)?;
        motion_sensor.set_interrupt_type(InterruptType::PosEdge)?;
        unsafe {
            let c = Self::new_queue(command_queue);
            motion_sensor.subscribe(move || {
                c.send_front(AppShellCommand::MotionSensed, 100).unwrap();
                //log::info!("MOTION SENSED");
            })?;
        }
        Ok(motion_sensor)

    }

    fn connect_to_wifi(&mut self, modem: impl hal::peripheral::Peripheral<P = hal::modem::Modem> + 'static + std::marker::Send) -> Result<()> {
        self.app_state.network_activity_started();
        let config = ThreadSpawnConfiguration {
            name: Some(b"WifiConnectThread\0"),
            stack_size: 4096,
            priority: 5,
            inherit: false,
            pin_to_core: Some(Core::Core0),
        };

        config.set().unwrap();
        let c = Self::new_queue(&self.command_queue);

        std::thread::Builder::new()
            .name("RenderThread".to_string())
            .stack_size(4096)
            .spawn(move || {
                let app_config = CONFIG;
                let sysloop = EspSystemEventLoop::take().unwrap();
                let maybe_connection = wifi::wifi(
                    app_config.wifi_ssid,
                    app_config.wifi_psk,
                    modem,
                    sysloop,
                );
                match maybe_connection {
                    Ok(wifi_connection) => {
                        unsafe {
                            WIFI_CONNECTION = Some(wifi_connection);
                            c.send_back(AppShellCommand::WifiConnected, 100).unwrap();
                        }
                    },
                    Err(error) => {
                        log::error!("Failed to connect to wifi {:?}", error);
                    }
                }
            });
        Ok(())
    }

    fn new_queue(command_queue: &Queue<AppShellCommand>) -> Queue<AppShellCommand> {
        unsafe {
            Queue::new_borrowed(command_queue.as_raw())
        }
    }
}


fn start_render_thread(mut leds: LEDOutput) {
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
            leds.render_loop();
        })
        .unwrap();
}

struct LEDOutput {
    sclk: Option<gpio::Gpio12>,
    sdo: Option<gpio::Gpio11>,
    sdi: Option<gpio::Gpio13>,
    spi_device: Option<spi::SPI2>,
    led_rx: mpsc::Receiver<LEDIter>
}

    
impl LEDOutput {
    fn new(sclk: Option<gpio::Gpio12>, sdo: Option<gpio::Gpio11>, sdi: Option<gpio::Gpio13>, spi_device: spi::SPI2, led_rx: mpsc::Receiver<LEDIter>) -> Result<LEDOutput> {
        Ok(LEDOutput { sclk, sdo, sdi, spi_device: Some(spi_device), led_rx })
    }

    fn create_leds<'a>(& mut self) -> Result<LEDs<'a>> {
        let spi_config = spi::SpiConfig::new().baudrate(6410.kHz().into());
        let spi_driver = SpiDriver::new::<SPI2>(self.spi_device.take().unwrap(), self.sclk.take().unwrap(), self.sdo.take().unwrap(), self.sdi.take(), &SpiDriverConfig::new())?;
        let spi_bus = spi::SpiBusDriver::new(spi_driver, &spi_config)?;
        Ok(Ws2812::new(spi_bus))
    }

    fn render_loop(&mut self) -> Result<()> {
        let mut leds = self.create_leds()?;
        for led_buffer in &self.led_rx {
            log::debug!("Rendering leds");
            leds.write(led_buffer).unwrap();
        }
        Ok(())
    }
}


#[toml_cfg::toml_config]
pub struct Config {
    #[default("Wokwi-GUEST")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}


