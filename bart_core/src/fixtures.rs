use anyhow::Result;

pub fn json_with_etd_3_trains(first: &str, second:&str) -> Result<String> {
    Ok(format!(r##"{{
"?xml": {{
"@version": "1.0",
"@encoding": "utf-8"
}},
"root": {{
"@id": "1",
"uri": {{
"#cdata-section": "http://api.bart.gov/api/etd.aspx?cmd=etd&orig=ROCK&json=y"
}},
"date": "05/09/2024",
"time": "04:24:03 PM PDT",
"station": [
{{
"name": "Rockridge",
"abbr": "ROCK",
"etd": [
{{
"destination": "Antioch",
"abbreviation": "ANTC",
"limited": "0",
"estimate": [
{{
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
}},
{{
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
}},
{{
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
}}
]
}},
{{
"destination": "Pittsburg/Bay Point",
"abbreviation": "PITT",
"limited": "0",
"estimate": [
{{
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
}},
{{
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
}},
{{
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
}}
]
}},
{{
"destination": "SF Airport",
"abbreviation": "SFIA",
"limited": "0",
"estimate": [
{{
"minutes": "{}",
"platform": "2",
"direction": "South",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "0",
"cancelflag": "0",
"dynamicflag": "0"
}},
{{
"minutes": "{}",
"platform": "2",
"direction": "South",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "0",
"cancelflag": "0",
"dynamicflag": "0"
}},
{{
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
}}
]
}}
]
}}
],
"message": ""
}}
}}"##, first, second))
}


pub fn json_with_etd_2_trains(first: &str, second:&str) -> Result<String> {
    Ok(format!(r##"{{
"?xml": {{
"@version": "1.0",
"@encoding": "utf-8"
}},
"root": {{
"@id": "1",
"uri": {{
"#cdata-section": "http://api.bart.gov/api/etd.aspx?cmd=etd&orig=ROCK&json=y"
}},
"date": "05/09/2024",
"time": "04:24:03 PM PDT",
"station": [
{{
"name": "Rockridge",
"abbr": "ROCK",
"etd": [
{{
"destination": "Antioch",
"abbreviation": "ANTC",
"limited": "0",
"estimate": [
{{
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
}},
{{
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
}},
{{
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
}}
]
}},
{{
"destination": "Pittsburg/Bay Point",
"abbreviation": "PITT",
"limited": "0",
"estimate": [
{{
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
}},
{{
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
}},
{{
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
}}
]
}},
{{
"destination": "SF Airport",
"abbreviation": "SFIA",
"limited": "0",
"estimate": [
{{
"minutes": "{}",
"platform": "2",
"direction": "South",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "0",
"cancelflag": "0",
"dynamicflag": "0"
}},
{{
"minutes": "{}",
"platform": "2",
"direction": "South",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "0",
"cancelflag": "0",
"dynamicflag": "0"
}}
]
}}
]
}}
],
"message": ""
}}
}}"##, first, second))
}


pub fn json_with_etd_1_train(first: &str) -> Result<String> {
    Ok(format!(r##"{{
"?xml": {{
"@version": "1.0",
"@encoding": "utf-8"
}},
"root": {{
"@id": "1",
"uri": {{
"#cdata-section": "http://api.bart.gov/api/etd.aspx?cmd=etd&orig=ROCK&json=y"
}},
"date": "05/09/2024",
"time": "04:24:03 PM PDT",
"station": [
{{
"name": "Rockridge",
"abbr": "ROCK",
"etd": [
{{
"destination": "Antioch",
"abbreviation": "ANTC",
"limited": "0",
"estimate": [
{{
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
}},
{{
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
}},
{{
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
}}
]
}},
{{
"destination": "Pittsburg/Bay Point",
"abbreviation": "PITT",
"limited": "0",
"estimate": [
{{
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
}},
{{
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
}},
{{
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
}}
]
}},
{{
"destination": "SF Airport",
"abbreviation": "SFIA",
"limited": "0",
"estimate": [
{{
"minutes": "{}",
"platform": "2",
"direction": "South",
"length": "8",
"color": "YELLOW",
"hexcolor": "#ffff33",
"bikeflag": "1",
"delay": "0",
"cancelflag": "0",
"dynamicflag": "0"
}}
]
}}
]
}}
],
"message": ""
}}
}}"##, first))
}









