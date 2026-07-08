//! Weather collector. Uses a dedicated command `read_weather(zip)` rather than the
//! generic `read_tile`, because it needs the stored US ZIP code as input.
//!
//! We geocode the ZIP via zippopotam.us, then fetch the forecast from **Open-Meteo**
//! (no key). If Open-Meteo is unreachable, we fall back to the **US National Weather
//! Service** (api.weather.gov, no key, US-only) so a single provider's outage doesn't
//! blank the tile. Any network/parse failure maps to `Unavailable` so the tile degrades
//! calmly.

use reqwest::blocking::Client;
use serde::Serialize;
use std::sync::OnceLock;
use std::time::Duration;

/// One shared HTTP client for all weather fetches (connection pool + fixed timeouts),
/// instead of rebuilding one per refresh.
fn http() -> &'static Client {
    static CLIENT: OnceLock<Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        Client::builder()
            .timeout(Duration::from_secs(6))
            .connect_timeout(Duration::from_secs(3))
            // NWS requires a User-Agent; harmless for the other two hosts.
            .user_agent("momPanel (+https://github.com/kaislate/momPanel)")
            .build()
            // Building only fails if the TLS backend can't init; Client::new() is the
            // same default and would panic identically, so this is effectively infallible.
            .unwrap_or_else(|_| Client::new())
    })
}

// Temperatures are in Fahrenheit.
#[derive(Serialize)]
pub struct DayForecast {
    pub date: String, // YYYY-MM-DD (local tz); frontend formats the weekday
    pub code: u8,
    pub high_f: f32,
    pub low_f: f32,
    pub precip_prob: u8, // max precipitation probability for the day, 0-100
}

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum WeatherData {
    Ok {
        temp_f: f32,
        code: u8,
        high_f: f32,
        low_f: f32,
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

/// Map a US NWS `shortForecast` phrase to the closest WMO code, so the fallback data
/// flows through the same `condition()`/icon mapping as Open-Meteo.
pub fn nws_code(short_forecast: &str) -> u8 {
    let t = short_forecast.to_lowercase();
    if t.contains("thunder") {
        95
    } else if t.contains("snow")
        || t.contains("flurr")
        || t.contains("sleet")
        || t.contains("ice")
        || t.contains("blizzard")
    {
        71
    } else if t.contains("rain") || t.contains("shower") || t.contains("drizzle") {
        61
    } else if t.contains("fog") || t.contains("haze") || t.contains("mist") {
        45
    } else if t.contains("partly")
        || t.contains("mostly cloudy")
        || t.contains("scattered")
        || t.contains("broken")
    {
        2
    } else if t.contains("cloud") || t.contains("overcast") {
        3
    } else if t.contains("sunny") || t.contains("clear") || t.contains("fair") {
        0
    } else {
        2
    }
}

/// A parsed forecast payload shared by both providers.
type Forecast = (f32, u8, f32, f32, Vec<DayForecast>); // temp_f, code, high_f, low_f, days

/// Geocode a US ZIP, then fetch the forecast (Open-Meteo, falling back to NWS).
/// Any error -> Unavailable.
pub fn read(zip: &str) -> WeatherData {
    match try_read(zip) {
        Some(data) => data,
        None => WeatherData::Unavailable,
    }
}

fn try_read(zip: &str) -> Option<WeatherData> {
    let client = http();

    // 1) Geocode ZIP -> lat/lon + place name (cached in config while the ZIP matches).
    let geo = geocode(client, zip)?;

    // 2) Forecast: Open-Meteo, then NWS as a fallback.
    let (temp_f, code, high_f, low_f, days) = forecast_open_meteo(client, &geo.lat, &geo.lon)
        .or_else(|| forecast_nws(client, &geo.lat, &geo.lon))?;

    Some(WeatherData::Ok {
        temp_f,
        code,
        high_f,
        low_f,
        place: geo.place,
        days,
    })
}

/// A geocoded location. lat/lon are strings because that's the form the forecast APIs
/// consume (and what zippopotam.us returns).
struct Geo {
    lat: String,
    lon: String,
    place: String,
}

/// Whether the cached geocode can be reused for `zip`: the cached ZIP must match and all
/// three coordinate/place parts must be present. Pure, so it's unit-testable.
fn geo_cache_matches(
    cached_zip: Option<&str>,
    zip: &str,
    lat: Option<&str>,
    lon: Option<&str>,
    place: Option<&str>,
) -> bool {
    cached_zip == Some(zip) && lat.is_some() && lon.is_some() && place.is_some()
}

/// Resolve a ZIP to coordinates + place name. Reuses the cached result in config while
/// the ZIP is unchanged; otherwise geocodes via zippopotam.us, falling back to
/// Open-Meteo's geocoder, and writes the fresh result back through the config save path
/// so subsequent refreshes skip the geocoder entirely.
fn geocode(client: &Client, zip: &str) -> Option<Geo> {
    let cfg = crate::config::load();
    if geo_cache_matches(
        cfg.geo_zip.as_deref(),
        zip,
        cfg.geo_lat.as_deref(),
        cfg.geo_lon.as_deref(),
        cfg.geo_place.as_deref(),
    ) {
        // The guard above guarantees all three are Some.
        return Some(Geo {
            lat: cfg.geo_lat?,
            lon: cfg.geo_lon?,
            place: cfg.geo_place?,
        });
    }

    let geo = geocode_zippopotam(client, zip).or_else(|| geocode_open_meteo(client, zip))?;

    let (z, lat, lon, place) = (
        zip.to_string(),
        geo.lat.clone(),
        geo.lon.clone(),
        geo.place.clone(),
    );
    let _ = crate::config::update(|c| {
        c.geo_zip = Some(z);
        c.geo_lat = Some(lat);
        c.geo_lon = Some(lon);
        c.geo_place = Some(place);
        Ok(())
    });

    Some(geo)
}

/// Primary geocoder: zippopotam.us (US ZIP -> lat/lon strings + "City, ST").
fn geocode_zippopotam(client: &Client, zip: &str) -> Option<Geo> {
    let url = format!("https://api.zippopotam.us/us/{}", zip);
    let geo: serde_json::Value = client.get(&url).send().ok()?.json().ok()?;
    let place_obj = geo.get("places")?.get(0)?;
    let lat = place_obj.get("latitude")?.as_str()?.to_string();
    let lon = place_obj.get("longitude")?.as_str()?.to_string();
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
    Some(Geo { lat, lon, place })
}

/// Fallback geocoder: Open-Meteo's geocoding API, so a zippopotam.us outage doesn't
/// blank the tile. Its lat/lon are JSON numbers (unlike zippopotam's strings), so we
/// stringify them for the shared forecast path.
fn geocode_open_meteo(client: &Client, zip: &str) -> Option<Geo> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1",
        zip
    );
    let resp: serde_json::Value = client.get(&url).send().ok()?.json().ok()?;
    let hit = resp.get("results")?.as_array()?.first()?;
    // A bare ZIP can resolve to a postal code in another country; when the result
    // carries a country code, require the US match (accept if the field is absent).
    if let Some(cc) = hit.get("country_code").and_then(|v| v.as_str()) {
        if !cc.eq_ignore_ascii_case("US") {
            return None;
        }
    }
    let lat = hit.get("latitude")?.as_f64()?;
    let lon = hit.get("longitude")?.as_f64()?;
    let name = hit.get("name").and_then(|v| v.as_str()).unwrap_or(zip);
    let admin = hit.get("admin1").and_then(|v| v.as_str()).unwrap_or("");
    let place = if admin.is_empty() {
        name.to_string()
    } else {
        format!("{}, {}", name, admin)
    };
    Some(Geo {
        lat: lat.to_string(),
        lon: lon.to_string(),
        place,
    })
}

/// Primary provider: Open-Meteo (current + 7-day daily, WMO codes, Fahrenheit).
fn forecast_open_meteo(client: &Client, lat: &str, lon: &str) -> Option<Forecast> {
    let fc_url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}\
         &current=temperature_2m,weather_code\
         &daily=temperature_2m_max,temperature_2m_min,weather_code,precipitation_probability_max\
         &forecast_days=7&temperature_unit=fahrenheit&timezone=auto",
        lat, lon
    );
    let fc: serde_json::Value = client.get(&fc_url).send().ok()?.json().ok()?;

    let current = fc.get("current")?;
    let temp_f = current.get("temperature_2m")?.as_f64()? as f32;
    let code = current.get("weather_code")?.as_u64()? as u8;

    let daily = fc.get("daily")?;
    let times = daily.get("time")?.as_array()?;
    let maxs = daily.get("temperature_2m_max")?.as_array()?;
    let mins = daily.get("temperature_2m_min")?.as_array()?;
    let codes = daily.get("weather_code")?.as_array()?;
    let probs = daily.get("precipitation_probability_max").and_then(|v| v.as_array());

    let high_f = maxs.first()?.as_f64()? as f32;
    let low_f = mins.first()?.as_f64()? as f32;

    let n = times.len().min(7);
    let mut days = Vec::with_capacity(n);
    for i in 0..n {
        days.push(DayForecast {
            date: times[i].as_str().unwrap_or("").to_string(),
            code: codes.get(i).and_then(|v| v.as_u64()).unwrap_or(0) as u8,
            high_f: maxs.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            low_f: mins.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
            precip_prob: probs
                .and_then(|a| a.get(i))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u8,
        });
    }

    Some((temp_f, code, high_f, low_f, days))
}

/// Fallback provider: US National Weather Service. Two hops: `/points/{lat},{lon}` gives
/// the forecast URL, whose day/night periods we aggregate into daily highs/lows.
fn forecast_nws(client: &Client, lat: &str, lon: &str) -> Option<Forecast> {
    let latf: f64 = lat.parse().ok()?;
    let lonf: f64 = lon.parse().ok()?;
    let points_url = format!("https://api.weather.gov/points/{:.4},{:.4}", latf, lonf);
    let points: serde_json::Value = client.get(&points_url).send().ok()?.json().ok()?;
    let fc_url = points.get("properties")?.get("forecast")?.as_str()?;

    let fc: serde_json::Value = client.get(fc_url).send().ok()?.json().ok()?;
    let periods = fc.get("properties")?.get("periods")?.as_array()?;
    let first = periods.first()?;
    let temp_f = first.get("temperature")?.as_f64()? as f32;
    let cur_code = nws_code(first.get("shortForecast").and_then(|v| v.as_str()).unwrap_or(""));

    // Aggregate the chronological day/night periods into per-date highs/lows.
    let mut days: Vec<DayForecast> = Vec::new();
    for p in periods {
        let start = p.get("startTime").and_then(|v| v.as_str()).unwrap_or("");
        if start.len() < 10 {
            continue;
        }
        let date = start[..10].to_string();
        let temp = p.get("temperature").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
        let is_day = p.get("isDaytime").and_then(|v| v.as_bool()).unwrap_or(true);
        let sf = p.get("shortForecast").and_then(|v| v.as_str()).unwrap_or("");
        let precip = p
            .get("probabilityOfPrecipitation")
            .and_then(|v| v.get("value"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;

        match days.last_mut() {
            Some(d) if d.date == date => {
                if temp > d.high_f {
                    d.high_f = temp;
                }
                if temp < d.low_f {
                    d.low_f = temp;
                }
                if is_day {
                    d.code = nws_code(sf); // prefer the daytime condition
                }
                if precip > d.precip_prob {
                    d.precip_prob = precip;
                }
            }
            _ => days.push(DayForecast {
                date,
                code: nws_code(sf),
                high_f: temp,
                low_f: temp,
                precip_prob: precip,
            }),
        }
    }
    days.truncate(7);

    let high_f = days.first()?.high_f;
    let low_f = days.first()?.low_f;
    Some((temp_f, cur_code, high_f, low_f, days))
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

    #[test]
    fn nws_maps_to_wmo_buckets() {
        assert_eq!(nws_code("Sunny"), 0);
        assert_eq!(nws_code("Mostly Sunny"), 0);
        assert_eq!(nws_code("Partly Cloudy"), 2);
        assert_eq!(nws_code("Mostly Cloudy"), 2);
        assert_eq!(nws_code("Cloudy"), 3);
        assert_eq!(nws_code("Chance Rain Showers"), 61);
        assert_eq!(nws_code("Light Snow"), 71);
        assert_eq!(nws_code("Scattered Thunderstorms"), 95);
        assert_eq!(nws_code("Patchy Fog"), 45);
        assert_eq!(nws_code("Areas Of Smoke"), 2); // unknown -> cloudy bucket
    }

    #[test]
    fn geo_cache_reused_only_on_exact_zip_with_all_parts() {
        // Complete cache for the same ZIP -> reuse.
        assert!(geo_cache_matches(
            Some("90210"), "90210", Some("34.09"), Some("-118.4"), Some("Beverly Hills, CA")
        ));
        // Different ZIP -> miss (must re-geocode).
        assert!(!geo_cache_matches(
            Some("10001"), "90210", Some("34.09"), Some("-118.4"), Some("Beverly Hills, CA")
        ));
        // No cached ZIP -> miss.
        assert!(!geo_cache_matches(None, "90210", Some("34.09"), Some("-118.4"), Some("X")));
        // Matching ZIP but a missing part -> miss.
        assert!(!geo_cache_matches(Some("90210"), "90210", None, Some("-118.4"), Some("X")));
        assert!(!geo_cache_matches(Some("90210"), "90210", Some("34.09"), None, Some("X")));
        assert!(!geo_cache_matches(Some("90210"), "90210", Some("34.09"), Some("-118.4"), None));
    }

    #[test]
    fn nws_code_maps_through_condition() {
        // The fallback's codes must land in the same buckets the frontend expects.
        assert_eq!(condition(nws_code("Thunderstorms")), "thunder");
        assert_eq!(condition(nws_code("Snow")), "snow");
        assert_eq!(condition(nws_code("Rain")), "rain");
        assert_eq!(condition(nws_code("Sunny")), "clear");
    }
}
