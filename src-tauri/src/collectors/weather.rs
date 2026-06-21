//! Weather collector. Uses a dedicated command `read_weather(zip)` rather than the
//! generic `read_tile`, because it needs the stored US ZIP code as input.
//!
//! Real implementation runs on every platform (it is just two HTTPS GETs), but any
//! network or parse failure maps to `Unavailable` so the tile degrades calmly. We
//! geocode the ZIP via zippopotam.us, then fetch current + daily forecast from
//! Open-Meteo (no API key required).

use serde::Serialize;
use std::time::Duration;

#[derive(Serialize)]
pub struct DayForecast {
    pub date: String, // YYYY-MM-DD (local tz); frontend formats the weekday
    pub code: u8,
    pub high_c: f32,
    pub low_c: f32,
}

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum WeatherData {
    Ok {
        temp_c: f32,
        code: u8,
        high_c: f32,
        low_c: f32,
        place: String,
        days: Vec<DayForecast>,
    },
    Unavailable,
}

/// Map a WMO weather-interpretation code to a coarse condition string. The frontend
/// replicates this same mapping to pick an icon.
pub fn condition(code: u8) -> &'static str {
    match code {
        0 => "clear",
        1..=3 => "cloudy",
        45 | 48 => "fog",
        51..=67 => "rain",
        71..=86 => "snow",
        95..=99 => "thunder",
        _ => "cloudy",
    }
}

/// Geocode a US ZIP, then fetch current + daily forecast. Any error -> Unavailable.
pub fn read(zip: &str) -> WeatherData {
    match try_read(zip) {
        Some(data) => data,
        None => WeatherData::Unavailable,
    }
}

fn try_read(zip: &str) -> Option<WeatherData> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .connect_timeout(Duration::from_secs(3))
        .build()
        .ok()?;

    // 1) Geocode ZIP -> lat/lon + place name.
    let geo_url = format!("https://api.zippopotam.us/us/{}", zip);
    let geo: serde_json::Value = client.get(&geo_url).send().ok()?.json().ok()?;
    let place_obj = geo.get("places")?.get(0)?;
    let lat = place_obj.get("latitude")?.as_str()?;
    let lon = place_obj.get("longitude")?.as_str()?;
    let place_name = place_obj.get("place name")?.as_str()?;
    let state_abbr = place_obj
        .get("state abbreviation")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let place = if state_abbr.is_empty() {
        place_name.to_string()
    } else {
        format!("{}, {}", place_name, state_abbr)
    };

    // 2) Forecast.
    let fc_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}\
         &current=temperature_2m,weather_code\
         &daily=temperature_2m_max,temperature_2m_min,weather_code\
         &forecast_days=5&timezone=auto",
        lat, lon
    );
    let fc: serde_json::Value = client.get(&fc_url).send().ok()?.json().ok()?;

    let current = fc.get("current")?;
    let temp_c = current.get("temperature_2m")?.as_f64()? as f32;
    let code = current.get("weather_code")?.as_u64()? as u8;

    let daily = fc.get("daily")?;
    let times = daily.get("time")?.as_array()?;
    let maxs = daily.get("temperature_2m_max")?.as_array()?;
    let mins = daily.get("temperature_2m_min")?.as_array()?;
    let codes = daily.get("weather_code")?.as_array()?;

    let high_c = maxs.first()?.as_f64()? as f32;
    let low_c = mins.first()?.as_f64()? as f32;

    let n = times.len().min(5);
    let mut days = Vec::with_capacity(n);
    for i in 0..n {
        days.push(DayForecast {
            date: times[i].as_str().unwrap_or("").to_string(),
            code: codes.get(i).and_then(|v| v.as_u64()).unwrap_or(0) as u8,
            high_c: maxs.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            low_c: mins.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
        });
    }

    Some(WeatherData::Ok {
        temp_c,
        code,
        high_c,
        low_c,
        place,
        days,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn condition_clear() {
        assert_eq!(condition(0), "clear");
    }

    #[test]
    fn condition_rain() {
        assert_eq!(condition(61), "rain");
    }

    #[test]
    fn condition_snow() {
        assert_eq!(condition(71), "snow");
    }

    #[test]
    fn condition_thunder() {
        assert_eq!(condition(95), "thunder");
    }

    #[test]
    fn condition_buckets() {
        assert_eq!(condition(2), "cloudy");
        assert_eq!(condition(45), "fog");
        assert_eq!(condition(48), "fog");
        assert_eq!(condition(67), "rain");
        assert_eq!(condition(86), "snow");
        assert_eq!(condition(99), "thunder");
        assert_eq!(condition(200), "cloudy"); // out-of-range fallback
    }
}
