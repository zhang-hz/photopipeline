use chrono::{DateTime, Duration, TimeZone, Utc};
use photopipeline_core::*;

pub fn gpx_simple_track() -> GpxTrack {
    let t1 = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
    let t2 = Utc.with_ymd_and_hms(2024, 6, 15, 10, 5, 0).unwrap();
    GpxTrack {
        name: Some("Simple Track".into()),
        points: vec![
            GpxPoint {
                latitude: 40.0,
                longitude: -74.0,
                elevation: Some(10.0),
                timestamp: Some(t1),
                speed: Some(5.0),
                bearing: Some(45.0),
            },
            GpxPoint {
                latitude: 41.0,
                longitude: -73.0,
                elevation: Some(20.0),
                timestamp: Some(t2),
                speed: Some(10.0),
                bearing: Some(90.0),
            },
        ],
        duration_seconds: Some(300.0),
        distance_meters: Some(150000.0),
    }
}

pub fn gpx_nyc_to_boston() -> GpxTrack {
    let start = Utc.with_ymd_and_hms(2024, 7, 1, 8, 0, 0).unwrap();
    let lat_start = 40.7128;
    let lon_start = -74.0060;
    let lat_end = 42.3601;
    let lon_end = -71.0589;
    let n = 10u32;
    let mut points = Vec::with_capacity(n as usize);
    for i in 0..n {
        let frac = i as f64 / (n - 1) as f64;
        points.push(GpxPoint {
            latitude: lat_start + (lat_end - lat_start) * frac,
            longitude: lon_start + (lon_end - lon_start) * frac,
            elevation: Some(50.0 + frac * 100.0),
            timestamp: Some(start + Duration::minutes(i as i64 * 30)),
            speed: Some(80.0 + frac * 20.0),
            bearing: Some(45.0 + frac * 15.0),
        });
    }
    GpxTrack {
        name: Some("NYC to Boston".into()),
        points,
        duration_seconds: Some((n as f64) * 30.0 * 60.0),
        distance_meters: Some(350000.0),
    }
}

pub fn gpx_hourly_track(start_time: DateTime<Utc>, lat: f64, lon: f64) -> GpxTrack {
    let mut points = Vec::with_capacity(60);
    for i in 0..60 {
        points.push(GpxPoint {
            latitude: lat + (i as f64) * 0.001,
            longitude: lon + (i as f64) * 0.001,
            elevation: Some(100.0 + (i as f64) * 10.0),
            timestamp: Some(start_time + Duration::seconds(i as i64 * 60)),
            speed: Some(3.0 + (i as f64 % 10.0)),
            bearing: Some((i as f64 * 6.0) % 360.0),
        });
    }
    GpxTrack {
        name: Some("Hourly Track".into()),
        points,
        duration_seconds: Some(3600.0),
        distance_meters: None,
    }
}

pub fn gpx_mountain_hike() -> GpxTrack {
    let start = Utc.with_ymd_and_hms(2024, 8, 15, 6, 0, 0).unwrap();
    let waypoints = vec![
        (46.5, 7.0, 500.0, 5.0, 180.0),
        (46.51, 7.01, 600.0, 4.0, 170.0),
        (46.52, 7.015, 750.0, 3.5, 190.0),
        (46.53, 7.02, 900.0, 2.0, 160.0),
        (46.54, 7.025, 1100.0, 1.5, 175.0),
        (46.545, 7.018, 1300.0, 0.8, 200.0),
        (46.55, 7.01, 1500.0, 1.0, 220.0),
        (46.555, 7.0, 1400.0, 3.0, 260.0),
        (46.56, 6.995, 1200.0, 4.0, 280.0),
        (46.565, 6.99, 1000.0, 5.0, 300.0),
    ];
    let mut points = Vec::with_capacity(waypoints.len());
    for (i, (lat, lon, elev, speed, bearing)) in waypoints.iter().enumerate() {
        points.push(GpxPoint {
            latitude: *lat,
            longitude: *lon,
            elevation: Some(*elev),
            timestamp: Some(start + Duration::minutes(i as i64 * 20)),
            speed: Some(*speed),
            bearing: Some(*bearing),
        });
    }
    GpxTrack {
        name: Some("Mountain Hike".into()),
        points,
        duration_seconds: Some(waypoints.len() as f64 * 20.0 * 60.0),
        distance_meters: Some(15000.0),
    }
}

pub fn gpx_duplicate_timestamps() -> GpxTrack {
    let t = Utc.with_ymd_and_hms(2024, 9, 1, 12, 0, 0).unwrap();
    GpxTrack {
        name: Some("Duplicate Timestamps".into()),
        points: vec![
            GpxPoint {
                latitude: 40.0,
                longitude: -74.0,
                elevation: Some(10.0),
                timestamp: Some(t),
                speed: Some(5.0),
                bearing: Some(90.0),
            },
            GpxPoint {
                latitude: 41.0,
                longitude: -73.0,
                elevation: Some(20.0),
                timestamp: Some(t),
                speed: Some(10.0),
                bearing: Some(180.0),
            },
        ],
        duration_seconds: Some(0.0),
        distance_meters: None,
    }
}

pub fn gpx_empty() -> GpxTrack {
    GpxTrack {
        name: Some("Empty Track".into()),
        points: vec![],
        duration_seconds: Some(0.0),
        distance_meters: Some(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpx_simple_track_has_two_points() {
        let track = gpx_simple_track();
        assert_eq!(track.points.len(), 2);
        assert_eq!(track.name, Some("Simple Track".into()));
    }

    #[test]
    fn gpx_nyc_to_boston_has_10_points() {
        let track = gpx_nyc_to_boston();
        assert_eq!(track.points.len(), 10);
        let first = &track.points[0];
        assert!((first.latitude - 40.7128).abs() < 0.01);
        let last = track.points.last().unwrap();
        assert!((last.latitude - 42.3601).abs() < 0.01);
    }

    #[test]
    fn gpx_hourly_track_has_60_points() {
        let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let track = gpx_hourly_track(start, 0.0, 0.0);
        assert_eq!(track.points.len(), 60);
        assert_eq!(track.duration_seconds, Some(3600.0));
    }

    #[test]
    fn gpx_mountain_hike_has_realistic_data() {
        let track = gpx_mountain_hike();
        assert_eq!(track.points.len(), 10);
        for pt in &track.points {
            assert!(pt.latitude > 46.0);
            assert!(pt.elevation.unwrap() > 400.0);
        }
    }

    #[test]
    fn gpx_duplicate_timestamps_has_same_ts() {
        let track = gpx_duplicate_timestamps();
        assert_eq!(track.points.len(), 2);
        assert_eq!(track.points[0].timestamp, track.points[1].timestamp);
    }

    #[test]
    fn gpx_empty_has_no_points() {
        let track = gpx_empty();
        assert!(track.points.is_empty());
    }

    #[test]
    fn gpx_nyc_to_boston_interpolation_at_start() {
        let track = gpx_nyc_to_boston();
        let result = track.interpolate_at(&track.points[0].timestamp.unwrap());
        assert!(result.is_some());
        let pt = result.unwrap();
        assert!((pt.latitude - 40.7128).abs() < 0.01);
    }

    #[test]
    fn gpx_duplicate_interpolate() {
        let track = gpx_duplicate_timestamps();
        let t = track.points[0].timestamp.unwrap();
        let result = track.interpolate_at(&t);
        assert!(result.is_some());
    }

    #[test]
    fn gpx_empty_interpolate() {
        let track = gpx_empty();
        let t = Utc::now();
        assert!(track.interpolate_at(&t).is_none());
    }
}
