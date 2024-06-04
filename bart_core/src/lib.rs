use serde::Deserialize;
use anyhow::Result;
use smart_leds::RGB8;
use smart_leds::colors;
#[cfg(test)]
#[path = "lib.test.rs"]
mod tests;

const FETCH_CORRECTION_TIME_MIN: i32 = 10;
const FETCH_REFRESH_TIME_MIN: i32 = 5;
const FETCH_NEXT_TRAIN_TIME_MIN: i32 = 2;
const FETCH_RETRY_TIME_MIN: i32 = 2;

pub struct AppState {
    etd_mins: Vec<i32>
}

impl AppState {
    pub fn new() -> AppState {
        AppState {etd_mins: Vec::new()}
    }

    pub fn received_http_response(&mut self, response: Result<String>) -> u64 {
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

    pub fn get_current_led_buffer(&self, elapse_time_microsec: u64) -> LEDBuffer {
        const MICROSEC_PER_MIN: u64 = 60000000;
        let elapse_time_min = i32::try_from(elapse_time_microsec/MICROSEC_PER_MIN).unwrap();
        let current_etd_min: Vec<i32> = self.etd_mins.iter()
            .map(|etd| etd - elapse_time_min)//subtract time since fetch
            .filter(|etd| *etd > 0i32)//Filter out trains which have already left
            .collect();
        let mut buff = LEDBuffer::new();
        if current_etd_min.is_empty() {
            return buff;
        }
        let next_train =  current_etd_min[0];
        let color = colors::WHITE;
        if next_train > LEDBuffer::INSIDE_RING_SIZE {
            LEDBuffer::fill_ring(buff.outside_ring(), next_train, color);
        } else {
            LEDBuffer::fill_ring(buff.inside_ring(), next_train, color);
            if current_etd_min.len() >= 2 {
                let next_next_train = current_etd_min[1];
                LEDBuffer::fill_ring(buff.outside_ring(), next_next_train, color);
            }
        }
        buff
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

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LEDBuffer {
    pub rgb_buffer: [RGB8; Self::BUFFER_SIZE as usize],
}

impl LEDBuffer {
    const OUTSIDE_RING_SIZE: i32 = 24;
    const INSIDE_RING_SIZE: i32 = 16;
    const CENTER_RING_SIZE: i32 = 4;
    const BUFFER_SIZE: i32 =  LEDBuffer::OUTSIDE_RING_SIZE + LEDBuffer::INSIDE_RING_SIZE + LEDBuffer::CENTER_RING_SIZE;
    fn new() -> LEDBuffer {
        LEDBuffer{rgb_buffer: [RGB8::default(); Self::BUFFER_SIZE as usize]}
    }

    fn outside_ring(&mut self) -> &mut [RGB8] {
        &mut self.rgb_buffer[..Self::OUTSIDE_RING_SIZE as usize]
    }

    fn inside_ring(&mut self) -> &mut [RGB8] {
        &mut self.rgb_buffer[Self::OUTSIDE_RING_SIZE as usize ..(Self::OUTSIDE_RING_SIZE + Self::INSIDE_RING_SIZE) as usize]
    }

    fn center_ring(&mut self) -> &mut [RGB8] {
        &mut self.rgb_buffer[(Self::OUTSIDE_RING_SIZE + Self::INSIDE_RING_SIZE) as usize..]
    }

    fn fill_ring(ring: &mut [RGB8], count: i32, value: RGB8) {
        let count = count as usize;
        for led in ring.iter_mut().take(count) {
            *led = value;
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

