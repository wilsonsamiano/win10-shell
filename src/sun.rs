//! Sunrise and sunset. Same NOAA equation the Android launcher used.
//! No location yet: the time zone meridian still separates day and night.

use chrono::Datelike;

pub fn is_day(now_ms: i64, lat: f64, lon: f64) -> bool {
    let (rise, set) = sun_times(lat, lon, now_ms);
    now_ms >= rise && now_ms <= set
}

fn sun_times(lat: f64, lon: f64, when_ms: i64) -> (i64, i64) {
    let date = chrono::DateTime::from_timestamp_millis(when_ms)
        .unwrap_or_else(chrono::Utc::now)
        .date_naive();
    let j_date = julian(date.year(), date.month() as i32, date.day() as i32) + 0.5;
    let n = j_date - 2_451_545.0 + 0.0008;
    let j_star = n - lon / 360.0;
    let m = ((357.5291 + 0.98560028 * j_star) % 360.0).to_radians();
    let c = 1.9148 * m.sin() + 0.0200 * (2.0 * m).sin() + 0.0003 * (3.0 * m).sin();
    let lambda = (m.to_degrees() + c + 180.0 + 102.9372).rem_euclid(360.0).to_radians();
    let j_transit = 2_451_545.0 + j_star + 0.0053 * m.sin() - 0.0069 * (2.0 * lambda).sin();
    let delta = (lambda.sin() * 23.4397_f64.to_radians().sin()).asin();
    let lat_r = lat.to_radians();
    let cos_omega = ((-0.833_f64).to_radians().sin() - lat_r.sin() * delta.sin())
        / (lat_r.cos() * delta.cos());
    if cos_omega >= 1.0 {
        return (0, 0);
    }
    if cos_omega <= -1.0 {
        return (when_ms - 1, when_ms + 86_400_000);
    }
    let omega = cos_omega.acos().to_degrees();
    let rise = julian_to_millis(j_transit - omega / 360.0);
    let set = julian_to_millis(j_transit + omega / 360.0);
    (rise, set)
}

fn julian(year: i32, month: i32, day: i32) -> f64 {
    let (y, m) = if month <= 2 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let a = (y as f64 / 100.0).floor();
    let b = 2.0 - a + (a / 4.0).floor();
    (365.25 * (y as f64 + 4716.0)).floor() + (30.6001 * (m as f64 + 1.0)).floor() + day as f64 + b
        - 1524.5
}

fn julian_to_millis(j: f64) -> i64 {
    ((j - 2_440_587.5) * 86_400_000.0) as i64
}

pub fn timezone_place() -> (f64, f64) {
    let hours = chrono::Local::now().offset().local_minus_utc() as f64 / 3600.0;
    (31.8, hours * 15.0)
}

/// `latest` is newer than `current`. Both are dotted numbers, an optional leading v is ignored.
pub fn is_newer(latest: &str, current: &str) -> bool {
    let parse = |s: &str| {
        s.trim()
            .trim_start_matches('v')
            .split(['.', '-'])
            .filter_map(|p| p.parse::<u32>().ok())
            .collect::<Vec<_>>()
    };
    let a = parse(latest);
    let b = parse(current);
    if a.is_empty() {
        return false;
    }
    let n = a.len().max(b.len());
    for i in 0..n {
        let av = *a.get(i).unwrap_or(&0);
        let bv = *b.get(i).unwrap_or(&0);
        if av != bv {
            return av > bv;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newer_versions() {
        assert!(is_newer("0.2.0", "0.1.0"));
        assert!(is_newer("v0.1.1", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.1.0", "0.2.0"));
    }

    #[test]
    fn el_paso_morning_is_day() {
        // 2026-06-15 18:00 UTC is noon in El Paso (UTC-6).
        let noon = chrono::NaiveDate::from_ymd_opt(2026, 6, 15)
            .unwrap()
            .and_hms_opt(18, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp_millis();
        assert!(is_day(noon, 31.76, -106.49));
    }
}
