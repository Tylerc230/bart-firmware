#![feature(duration_abs_diff)]
use serde::Deserialize;
use anyhow::Result;
pub use core::time::Duration;
#[cfg(test)]
#[path = "lib.test.rs"]
mod tests;
mod led_pipeline;
use led_pipeline::{Dim, ETDLEDs, LEDBuffer, NetworkAnimation, PipelineStep};

const FETCH_CORRECTION_TIME_MIN: i32 = 10;
const FETCH_REFRESH_TIME_MIN: i32 = 5;
const FETCH_NEXT_TRAIN_TIME_MIN: i32 = 2;
const FETCH_RETRY_TIME_MIN: i32 = 2;
const NETWORK_SLEEP_TIME_MIN: u64 = 10;

pub struct AppState {
    etd_mins: Vec<i32>,
    network_animation: Option<NetworkAnimation>,
    last_motion_sensed: Duration
}

impl AppState {
    pub fn new(now: Duration) -> AppState {
        AppState {etd_mins: Vec::new(), network_animation: None, last_motion_sensed: now}
    }

    pub fn network_activity_started(&mut self, elapse_time_microsec: u64) {
        self.network_animation = Some(NetworkAnimation::new(elapse_time_microsec));
    }

    pub fn network_activity_complete(&mut self) {
        self.network_animation = None;
    }

    pub fn received_http_response(&mut self, response: Result<String>) -> u64 {
        self.network_activity_complete();
        let minutes = match response {
            Err(error) => {
                log::error!("Server Response Err {:?}", error);
                FETCH_RETRY_TIME_MIN
            }
            Ok(payload) => {
                match self.parse_json(payload) {
                    Ok(json) => {
                        log::info!("JSON: {:?}", json);
                        self.update_state(json);
                        self.next_fetch_time()
                    },
                    Err(error) => {
                        log::error!("JSON: parse error {:?}", error);
                        FETCH_RETRY_TIME_MIN
                    }
                } 
            }
        };
        minutes as u64 * 60
    }

    pub fn get_current_led_buffer(&mut self, elapse_time_microsec: u64) -> LEDBuffer {
        let mut etd_led = ETDLEDs::new();
        etd_led.update(&self.etd_mins, elapse_time_microsec);
        let mut pipeline = vec![&mut etd_led as &mut dyn PipelineStep];
        if let Some(animation) = self.network_animation.as_mut() {
            pipeline.push(animation);
        }
        let mut dim = Dim::new(16);
        pipeline.push(&mut dim);
        LEDBuffer::process_pipeline(pipeline, elapse_time_microsec)
    }

    pub fn motion_sensed(&mut self, now: Duration)  {
        self.last_motion_sensed = now;
    }

    pub fn should_perform_fetch(&self, now: Duration) -> bool {
        let elapsed = self.last_motion_sensed.abs_diff(now);
        elapsed.as_secs() < NETWORK_SLEEP_TIME_MIN * 60
    }

    fn update_state(&mut self, json: Top) {
        self.etd_mins = json.root
            .station
            .into_iter()
            .flat_map(|station| {
                station.etd
            })
            .filter(|station| {
                station.abbreviation == "MLBR" || station.abbreviation == "SFIA"

            })
            .flat_map(|etd| {
                etd.estimate
            })
            .filter_map(| esd| {
                esd.minutes.parse::<i32>().ok() //esd.mintues can be "Leaving" need to filter those out
            })
            .collect::<Vec<i32>>();

        self.etd_mins.sort_by(|a, b| {
            a.cmp(b)
        });
        log::info!("Esimates {:?}", self.etd_mins);

    }

    fn parse_json(&self, response: String) -> Result<Top, serde_json::Error> {
        serde_json::from_str(&response)
    }

    fn next_fetch_time(&self) -> i32 {
        match self.etd_mins.len() {
            0..=1 => {
                FETCH_REFRESH_TIME_MIN
            }
            2 => {
                let next_train = self.etd_mins[0];
                let before_next_train = next_train - FETCH_NEXT_TRAIN_TIME_MIN;
                before_next_train.clamp(0, FETCH_CORRECTION_TIME_MIN)
            }
            _ => {
                FETCH_CORRECTION_TIME_MIN
            }
        }

    }

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
    etd: Vec<Etd>
}

#[derive(Deserialize, Debug)]
struct Etd {
    abbreviation: String,
    estimate: Vec<Estimate>
}

#[derive(Deserialize, Debug)]
struct Estimate {
    minutes: String,
}
