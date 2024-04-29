mod wifi;
mod http;
use core::str;
use anyhow::Result;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::prelude::Peripherals,
};

use serde::Deserialize;

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
    log::info!("Hello, world!");
    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take().unwrap();
    let _wifi = wifi::wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    );
    let response = http::get("https://api.bart.gov/api/etd.aspx?cmd=etd&orig=ROCK&key=MW9S-E7SL-26DU-VV8V&json=y")?;
    let json: Top = serde_json::from_str(&response)?;
    print!("{:?}", json);
    Ok(())
}

#[derive(Deserialize, Debug)]
struct Top {
    root: Root
}

#[derive(Deserialize, Debug)]
struct Root {
    station: Vec<Station>
}


#[derive(Deserialize, Debug)]
struct Station {
    name: String,
    etd: Vec<ETD>
}

#[derive(Deserialize, Debug)]
struct ETD {
    destination: String,
    estimate: Vec<Estimate>
}

#[derive(Deserialize, Debug)]
struct Estimate {
    minutes: String,
    delay: String,
}

