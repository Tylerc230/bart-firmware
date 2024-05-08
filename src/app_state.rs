use serde::Deserialize;
use anyhow::Result;
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

