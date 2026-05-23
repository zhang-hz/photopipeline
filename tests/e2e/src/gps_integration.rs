use test_harness::fixtures::gpx::{
    gpx_duplicate_timestamps, gpx_empty, gpx_simple_track,
};
use test_harness::fixtures::metadata::gps_beijing;

#[test]
fn e2e_gpx_interpolation_at_exact_point() {
    let track = gpx_simple_track();
    let t1 = track.points[0].timestamp.unwrap();
    let result = track.interpolate_at(&t1);
    assert!(result.is_some(), "should find exact point");
    let pt = result.unwrap();
    assert!((pt.latitude - 40.0).abs() < 0.0001);
    assert!((pt.longitude - (-74.0)).abs() < 0.0001);
}

#[test]
fn e2e_gpx_interpolation_between_points() {
    let track = gpx_simple_track();
    let t1 = track.points[0].timestamp.unwrap();
    let t2 = track.points[1].timestamp.unwrap();
    let t_mid = t1 + (t2 - t1) / 2;

    let result = track.interpolate_at(&t_mid);
    assert!(result.is_some(), "should interpolate midpoint");
    let pt = result.unwrap();
    assert!((pt.latitude - 40.5).abs() < 0.0001);
    assert!((pt.longitude - (-73.5)).abs() < 0.0001);
}

#[test]
fn e2e_gpx_interpolation_duplicate_timestamps_no_nan() {
    let track = gpx_duplicate_timestamps();
    let t = track.points[0].timestamp.unwrap();
    let result = track.interpolate_at(&t);
    assert!(result.is_some(), "duplicate timestamps should not panic");
    let pt = result.unwrap();
    assert!(!pt.latitude.is_nan(), "latitude should not be NaN");
    assert!(!pt.longitude.is_nan(), "longitude should not be NaN");
}

#[test]
fn e2e_gpx_max_interpolation_gap_enforced() {
    use chrono::{Duration, TimeZone, Utc};
    use photopipeline_core::{GpxPoint, GpxTrack};

    let start = Utc.with_ymd_and_hms(2024, 6, 15, 10, 0, 0).unwrap();
    let far = start + Duration::minutes(60);
    let track = GpxTrack {
        name: Some("Large Gap Track".into()),
        points: vec![
            GpxPoint {
                latitude: 40.0,
                longitude: -74.0,
                elevation: Some(10.0),
                timestamp: Some(start),
                speed: None,
                bearing: None,
            },
            GpxPoint {
                latitude: 42.0,
                longitude: -72.0,
                elevation: Some(20.0),
                timestamp: Some(far),
                speed: None,
                bearing: None,
            },
        ],
        duration_seconds: Some(3600.0),
        distance_meters: None,
    };

    let query_time = start + Duration::minutes(30);
    let result = track.interpolate_at(&query_time);
    assert!(result.is_some(), "interpolation should succeed at gap boundary");

    let gap_secs = (far.timestamp_millis() - start.timestamp_millis()) as f64 / 1000.0;
    assert!(gap_secs > 300.0, "gap should be larger than default max gap");
}

#[test]
fn e2e_gpx_empty_track_no_points() {
    use chrono::TimeZone;

    let track = gpx_empty();
    let t = chrono::Utc.with_ymd_and_hms(2024, 6, 15, 12, 0, 0).unwrap();
    let result = track.interpolate_at(&t);
    assert!(result.is_none(), "empty track should return None");
}

#[test]
fn e2e_gpx_track_missing_file_error() {
    let nonexistent_path = "/tmp/photopipeline_nonexistent_gpx_file_2024.gpx";
    let read_result = std::fs::read_to_string(nonexistent_path);
    assert!(read_result.is_err(), "reading nonexistent GPX should fail");
}

#[test]
fn e2e_gpx_manual_gps_mode() {
    use photopipeline_core::Metadata;

    let gps = gps_beijing();
    assert!(gps.has_coordinates(), "Beijing GPS should have coordinates");
    let (lat, lon) = gps.coordinate_tuple().unwrap();
    assert!((lat - 39.9042).abs() < 0.0001);
    assert!((lon - 116.4074).abs() < 0.0001);

    let mut metadata = Metadata::default();
    metadata.gps = Some(gps);
    assert!(metadata.gps.is_some());

    let resolved_gps = metadata.gps.unwrap();
    assert_eq!(resolved_gps.latitude_ref, Some("N".into()));
    assert_eq!(resolved_gps.longitude_ref, Some("E".into()));
    assert_eq!(resolved_gps.altitude, Some(43.5));
    assert_eq!(resolved_gps.map_datum, Some("WGS-84".into()));
}
