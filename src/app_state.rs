use serde::Deserialize;
use anyhow::Result;
use smart_leds::RGB8;
use smart_leds::colors;
pub struct AppState {
    minutes_until_next_trains: [Option<i32>; 2]
}

impl AppState {
    pub fn new() -> AppState {
        AppState {minutes_until_next_trains: [None; 2]}
    }
    pub fn received_http_response(&mut self, response: String) {
        match self.parse_json(response) {
            Ok(json) => {
                log::info!("{:?}", json);
                self.update_state(json);
            },
            Err(error) => {
                log::error!("{:?}", error);
            }
        } 
    }

    pub fn get_current_led_buffer(&self) -> LEDBuffer {
        let mut buff = LEDBuffer::new();
        let next_train =  self.minutes_until_next_trains[0].unwrap_or(0);
        let next_train = next_train as usize;
        let color = colors::YELLOW;
        if next_train > LEDBuffer::INSIDE_RING_SIZE {
            LEDBuffer::fill_ring(&mut buff.outside_ring(), next_train, color);
        } else {
            LEDBuffer::fill_ring(&mut buff.inside_ring(), next_train, color);
            if let Some(next_next_train) = self.minutes_until_next_trains[1] {
                let next_next_train = next_next_train as usize;
                LEDBuffer::fill_ring(&mut buff.outside_ring(), next_next_train, color);
            }
        }
        buff
    }

    fn update_state(&mut self, json: Top) {
        let mut estimates = json.root
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
            .map(| esd| {
                esd.minutes.parse::<i32>().ok()
            })
            .filter(|maybe_est|{
                maybe_est.is_some() //esd.mintues can be "Leaving" need to filter those out
            })
            .collect::<Vec<Option<i32>>>();

        estimates.sort_by(|a, b| {
            a.cmp(b)
        });
        self.minutes_until_next_trains = estimates.into_iter().take(2).collect::<Vec<Option<i32>>>().try_into().unwrap_or_default();
        log::info!("sort {:?}", self.minutes_until_next_trains);

    }

    fn parse_json(&self, response: String) -> Result<Top, serde_json::Error> {
        serde_json::from_str(&response)
    }

}

pub struct LEDBuffer {
    pub rgb_buffer: [RGB8; Self::BUFFER_SIZE],
}

impl LEDBuffer {
    const OUTSIDE_RING_SIZE: usize = 24;
    const INSIDE_RING_SIZE: usize = 16;
    const CENTER_RING_SIZE: usize = 4;
    const BUFFER_SIZE: usize =  LEDBuffer::OUTSIDE_RING_SIZE + LEDBuffer::INSIDE_RING_SIZE + LEDBuffer::CENTER_RING_SIZE;
    fn new() -> LEDBuffer {
        LEDBuffer{rgb_buffer: [RGB8::default(); Self::BUFFER_SIZE]}
    }

    fn outside_ring(&mut self) -> &mut [RGB8] {
        &mut self.rgb_buffer[..Self::OUTSIDE_RING_SIZE]
    }

    fn inside_ring(&mut self) -> &mut [RGB8] {
        &mut self.rgb_buffer[Self::OUTSIDE_RING_SIZE..Self::OUTSIDE_RING_SIZE + Self::INSIDE_RING_SIZE]
    }

    fn center_ring(&mut self) -> &mut [RGB8] {
        &mut self.rgb_buffer[Self::OUTSIDE_RING_SIZE + Self::INSIDE_RING_SIZE..]
    }

    fn fill_ring(ring: &mut [RGB8], count: usize, value: RGB8) {
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
    name: String,
    etd: Vec<ETD>
}

#[derive(Deserialize, Debug)]
struct ETD {
    destination: String,
    abbreviation: String,
    estimate: Vec<Estimate>
}

#[derive(Deserialize, Debug)]
struct Estimate {
    minutes: String,
    delay: String,
}

