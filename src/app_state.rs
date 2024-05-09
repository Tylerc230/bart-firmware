use serde::Deserialize;
use anyhow::Result;
use smart_leds::RGB8;
use smart_leds::colors;
//24 for ouside ring
//16 for inside
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
        //if first > 16, write to outer ring, inner ring black
        //if first <= 16 write to inner ring, outer ring from 2nd
        let mut buff = LEDBuffer::new();
        let next_train =  self.minutes_until_next_trains[0].unwrap_or(0);
        let next_train = next_train as usize;
        if next_train > buff.inside_ring.len() {
            LEDBuffer::fill_ring(&mut buff.outside_ring, next_train, colors::YELLOW);

        } else {

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
                station.abbreviation == "ANTC" || station.abbreviation == "PITT"
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
    outside_ring: [RGB8; 24],
    inside_ring: [RGB8; 16],
    center_ring: [RGB8; 4]
}

impl LEDBuffer {
    fn new() -> LEDBuffer {
        LEDBuffer{outside_ring: Default::default(), inside_ring: Default::default(), center_ring: Default::default()}
    }

    fn fill_ring<const N: usize>(ring: &mut [RGB8; N], count: usize, value: RGB8) {
        for i in 0..count {
            ring[i] = value
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

