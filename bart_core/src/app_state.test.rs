use crate::AppState;
#[test]
fn test_add() {
    let mut app_state = AppState::new();
    app_state.received_http_response(JSON.to_string());
    assert_eq!(app_state.minutes_until_next_trains[0], Some(4));
    assert_eq!(app_state.minutes_until_next_trains[1], Some(15));
}

const JSON: &str = r##"{
"?xml": {
"@version": "1.0",
"@encoding": "utf-8"
},
"root": {
"@id": "1",
"uri": {
"#cdata-section": "http://api.bart.gov/api/etd.aspx?cmd=etd&orig=ROCK&json=y"
},
"date": "05/09/2024",
"time": "04:24:03 PM PDT",
"station": [
{
"name": "Rockridge",
"abbr": "ROCK",
"etd": [
{
"destination": "Antioch",
"abbreviation": "ANTC",
"limited": "0",
"estimate": [
{
"minutes": "21",
"platform": "1",
"direction": "North",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "337",
"cancelflag": "0",
"dynamicflag": "0"
},
{
"minutes": "35",
"platform": "1",
"direction": "North",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "0",
"cancelflag": "0",
"dynamicflag": "0"
},
{
"minutes": "55",
"platform": "1",
"direction": "North",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "0",
"cancelflag": "0",
"dynamicflag": "0"
}
]
},
{
"destination": "Pittsburg/Bay Point",
"abbreviation": "PITT",
"limited": "0",
"estimate": [
{
"minutes": "11",
"platform": "1",
"direction": "North",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "403",
"cancelflag": "0",
"dynamicflag": "0"
},
{
"minutes": "26",
"platform": "1",
"direction": "North",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "91",
"cancelflag": "0",
"dynamicflag": "0"
},
{
"minutes": "45",
"platform": "1",
"direction": "North",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "0",
"cancelflag": "0",
"dynamicflag": "0"
}
]
},
{
"destination": "SF Airport",
"abbreviation": "SFIA",
"limited": "0",
"estimate": [
{
"minutes": "4",
"platform": "2",
"direction": "South",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "0",
"cancelflag": "0",
"dynamicflag": "0"
},
{
"minutes": "15",
"platform": "2",
"direction": "South",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "0",
"cancelflag": "0",
"dynamicflag": "0"
},
{
"minutes": "23",
"platform": "2",
"direction": "South",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "0",
"cancelflag": "0",
"dynamicflag": "0"
}
]
}
]
}
],
"message": ""
}
}"##;



