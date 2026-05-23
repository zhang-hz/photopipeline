use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Metadata {
    pub exif: Option<ExifData>,
    pub xmp: Option<XmpData>,
    pub iptc: Option<IptcData>,
    pub gps: Option<GpsData>,
    pub custom: Vec<CustomTag>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ExifData {
    pub make: Option<String>,
    pub model: Option<String>,
    pub lens_model: Option<String>,
    pub serial_number: Option<String>,
    pub software: Option<String>,
    pub artist: Option<String>,
    pub copyright: Option<String>,
    pub image_description: Option<String>,
    pub orientation: Option<u16>,

    pub date_time_original: Option<DateTime<Utc>>,
    pub date_time_digitized: Option<DateTime<Utc>>,
    pub sub_sec_time_original: Option<String>,
    pub offset_time_original: Option<String>,

    pub exposure_time: Option<String>,
    pub f_number: Option<String>,
    pub iso: Option<u32>,
    pub focal_length: Option<String>,
    pub focal_length_35mm: Option<u16>,
    pub aperture_value: Option<String>,
    pub shutter_speed_value: Option<String>,
    pub brightness_value: Option<String>,
    pub exposure_bias: Option<String>,
    pub metering_mode: Option<u16>,
    pub flash: Option<u16>,
    pub exposure_program: Option<u16>,
    pub white_balance: Option<u16>,

    pub image_width: Option<u32>,
    pub image_height: Option<u32>,
    pub color_space: Option<u16>,
    pub bits_per_sample: Option<Vec<u16>>,
    pub compression: Option<u16>,

    pub maker_note: Option<Vec<u8>>,
    pub raw_tags: Vec<RawExifTag>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawExifTag {
    pub tag: String,
    pub group: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct XmpData {
    pub creator: Option<String>,
    pub rights: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub create_date: Option<DateTime<Utc>>,
    pub modify_date: Option<DateTime<Utc>>,
    pub rating: Option<u8>,
    pub label: Option<String>,
    pub subject: Vec<String>,
    pub raw_properties: Vec<RawXmpProperty>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawXmpProperty {
    pub namespace: String,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct IptcData {
    pub creator: Option<String>,
    pub headline: Option<String>,
    pub caption: Option<String>,
    pub keywords: Vec<String>,
    pub copyright_notice: Option<String>,
    pub date_created: Option<DateTime<Utc>>,
    pub time_created: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub raw_tags: Vec<RawIptcTag>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawIptcTag {
    pub record: u8,
    pub dataset: u8,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct GpsData {
    pub latitude: Option<f64>,
    pub latitude_ref: Option<String>,
    pub longitude: Option<f64>,
    pub longitude_ref: Option<String>,
    pub altitude: Option<f64>,
    pub altitude_ref: Option<i8>,
    pub timestamp: Option<DateTime<Utc>>,
    pub img_direction: Option<f64>,
    pub img_direction_ref: Option<String>,
    pub map_datum: Option<String>,
    pub satellites: Option<String>,
    pub status: Option<String>,
    pub measure_mode: Option<String>,
    pub dop: Option<f64>,
    pub speed: Option<f64>,
    pub speed_ref: Option<String>,
    pub track: Option<f64>,
    pub track_ref: Option<String>,
    pub dest_bearing: Option<f64>,
    pub dest_bearing_ref: Option<String>,
    pub dest_distance: Option<f64>,
    pub dest_latitude: Option<f64>,
    pub dest_longitude: Option<f64>,
    pub processing_method: Option<String>,
    pub area_information: Option<String>,
    pub date_stamp: Option<String>,
}

impl GpsData {
    pub fn has_coordinates(&self) -> bool {
        self.latitude.is_some() && self.longitude.is_some()
    }

    pub fn coordinate_tuple(&self) -> Option<(f64, f64)> {
        match (self.latitude, self.longitude) {
            (Some(lat), Some(lon)) => Some((lat, lon)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CustomTag {
    pub key: String,
    pub value: String,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetadataScope {
    EXIF,
    XMP,
    IPTC,
    GPS,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataTarget {
    pub path: String,
    pub format: crate::types::ImageFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataWriteReport {
    pub tags_written: u32,
    pub tags_skipped: u32,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpxTrack {
    pub name: Option<String>,
    pub points: Vec<GpxPoint>,
    pub duration_seconds: Option<f64>,
    pub distance_meters: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GpxPoint {
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: Option<f64>,
    pub timestamp: Option<DateTime<Utc>>,
    pub speed: Option<f64>,
    pub bearing: Option<f64>,
}

impl GpxTrack {
    pub fn interpolate_at(&self, timestamp: &DateTime<Utc>) -> Option<GpxPoint> {
        let pts: Vec<&GpxPoint> = self.points.iter()
            .filter(|p| p.timestamp.is_some())
            .collect();
        if pts.is_empty() {
            return None;
        }

        let target_ts = timestamp.timestamp_millis();
        let mut before: Option<&GpxPoint> = None;
        let mut after: Option<&GpxPoint> = None;

        for pt in &pts {
            let ts = pt.timestamp.unwrap().timestamp_millis();
            if ts <= target_ts {
                before = Some(pt);
            }
            if ts >= target_ts && after.is_none() {
                after = Some(pt);
            }
        }

        match (before, after) {
            (Some(b), Some(a)) if b.timestamp == a.timestamp => {
                Some(GpxPoint { timestamp: Some(*timestamp), ..*b })
            }
            (Some(b), Some(a)) => {
                let t0 = b.timestamp.unwrap().timestamp_millis() as f64;
                let t1 = a.timestamp.unwrap().timestamp_millis() as f64;
                let t = target_ts as f64;
                let frac = ((t - t0) / (t1 - t0)).clamp(0.0, 1.0);
                Some(GpxPoint {
                    latitude: b.latitude + (a.latitude - b.latitude) * frac,
                    longitude: b.longitude + (a.longitude - b.longitude) * frac,
                    elevation: interpolate_f64(
                        b.elevation, a.elevation, frac,
                    ),
                    timestamp: Some(*timestamp),
                    speed: interpolate_f64(b.speed, a.speed, frac),
                    bearing: interpolate_bearing(b.bearing, a.bearing, frac),
                })
            }
            (Some(b), None) => Some(GpxPoint {
                timestamp: Some(*timestamp),
                ..*b
            }),
            (None, Some(a)) => Some(GpxPoint {
                timestamp: Some(*timestamp),
                ..*a
            }),
            _ => None,
        }
    }
}

fn interpolate_f64(a: Option<f64>, b: Option<f64>, frac: f64) -> Option<f64> {
    match (a, b) {
        (Some(va), Some(vb)) => Some(va + (vb - va) * frac),
        (Some(va), None) => Some(va),
        (None, Some(vb)) => Some(vb),
        (None, None) => None,
    }
}

fn interpolate_bearing(a: Option<f64>, b: Option<f64>, frac: f64) -> Option<f64> {
    match (a, b) {
        (Some(va), Some(vb)) => {
            let mut diff = vb - va;
            if diff > 180.0 { diff -= 360.0; }
            if diff < -180.0 { diff += 360.0; }
            let result = va + diff * frac;
            Some(if result < 0.0 { result + 360.0 } else if result >= 360.0 { result - 360.0 } else { result })
        }
        (Some(va), None) => Some(va),
        (None, Some(vb)) => Some(vb),
        (None, None) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn gps_data_has_coordinates() {
        let gps = GpsData::default();
        assert!(!gps.has_coordinates());

        let gps = GpsData {
            latitude: Some(34.0522),
            longitude: None,
            ..Default::default()
        };
        assert!(!gps.has_coordinates());

        let gps = GpsData {
            latitude: Some(34.0522),
            longitude: Some(-118.2437),
            ..Default::default()
        };
        assert!(gps.has_coordinates());
    }

    #[test]
    fn gps_data_coordinate_tuple() {
        let gps = GpsData::default();
        assert_eq!(gps.coordinate_tuple(), None);

        let gps = GpsData {
            latitude: Some(34.0522),
            longitude: Some(-118.2437),
            ..Default::default()
        };
        assert_eq!(gps.coordinate_tuple(), Some((34.0522, -118.2437)));
    }

    #[test]
    fn metadata_default_is_empty() {
        let m = Metadata::default();
        assert!(m.exif.is_none());
        assert!(m.xmp.is_none());
        assert!(m.iptc.is_none());
        assert!(m.gps.is_none());
        assert!(m.custom.is_empty());
    }

    #[test]
    fn gpx_track_interpolate_exact_match() {
        let t1 = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let t2 = Utc.with_ymd_and_hms(2024, 1, 1, 13, 0, 0).unwrap();
        let track = GpxTrack {
            name: None,
            points: vec![
                GpxPoint {
                    latitude: 40.0,
                    longitude: -74.0,
                    elevation: Some(10.0),
                    timestamp: Some(t1),
                    speed: Some(5.0),
                    bearing: Some(90.0),
                },
                GpxPoint {
                    latitude: 41.0,
                    longitude: -73.0,
                    elevation: Some(20.0),
                    timestamp: Some(t2),
                    speed: Some(10.0),
                    bearing: Some(180.0),
                },
            ],
            duration_seconds: None,
            distance_meters: None,
        };

        let result = track.interpolate_at(&t1);
        assert!(result.is_some());
        let pt = result.unwrap();
        assert!((pt.latitude - 40.0).abs() < 0.0001);
        assert!((pt.longitude - (-74.0)).abs() < 0.0001);
    }

    #[test]
    fn gpx_track_interpolate_midpoint() {
        let t1 = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let t2 = Utc.with_ymd_and_hms(2024, 1, 1, 14, 0, 0).unwrap();
        let t_mid = Utc.with_ymd_and_hms(2024, 1, 1, 13, 0, 0).unwrap();
        let track = GpxTrack {
            name: None,
            points: vec![
                GpxPoint {
                    latitude: 40.0,
                    longitude: -74.0,
                    elevation: Some(10.0),
                    timestamp: Some(t1),
                    speed: Some(5.0),
                    bearing: Some(0.0),
                },
                GpxPoint {
                    latitude: 42.0,
                    longitude: -72.0,
                    elevation: Some(20.0),
                    timestamp: Some(t2),
                    speed: Some(15.0),
                    bearing: Some(90.0),
                },
            ],
            duration_seconds: None,
            distance_meters: None,
        };

        let result = track.interpolate_at(&t_mid);
        assert!(result.is_some());
        let pt = result.unwrap();
        assert!((pt.latitude - 41.0).abs() < 0.0001);
        assert!((pt.longitude - (-73.0)).abs() < 0.0001);
        assert!((pt.elevation.unwrap() - 15.0).abs() < 0.0001);
        assert!((pt.speed.unwrap() - 10.0).abs() < 0.0001);
    }

    #[test]
    fn gpx_track_interpolate_before_first() {
        let t1 = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let t_before = Utc.with_ymd_and_hms(2024, 1, 1, 11, 0, 0).unwrap();
        let track = GpxTrack {
            name: None,
            points: vec![
                GpxPoint {
                    latitude: 40.0,
                    longitude: -74.0,
                    elevation: Some(10.0),
                    timestamp: Some(t1),
                    speed: Some(5.0),
                    bearing: Some(90.0),
                },
            ],
            duration_seconds: None,
            distance_meters: None,
        };

        let result = track.interpolate_at(&t_before);
        assert!(result.is_some());
        let pt = result.unwrap();
        assert!((pt.latitude - 40.0).abs() < 0.0001);
        assert!(pt.timestamp == Some(t_before));
    }

    #[test]
    fn gpx_track_interpolate_after_last() {
        let t1 = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        let t_after = Utc.with_ymd_and_hms(2024, 1, 1, 13, 0, 0).unwrap();
        let track = GpxTrack {
            name: None,
            points: vec![
                GpxPoint {
                    latitude: 40.0,
                    longitude: -74.0,
                    elevation: Some(10.0),
                    timestamp: Some(t1),
                    speed: Some(5.0),
                    bearing: Some(90.0),
                },
            ],
            duration_seconds: None,
            distance_meters: None,
        };

        let result = track.interpolate_at(&t_after);
        assert!(result.is_some());
        let pt = result.unwrap();
        assert!((pt.latitude - 40.0).abs() < 0.0001);
        assert!(pt.timestamp == Some(t_after));
    }

    #[test]
    fn gpx_track_interpolate_empty() {
        let track = GpxTrack {
            name: None,
            points: vec![],
            duration_seconds: None,
            distance_meters: None,
        };
        let t = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
        assert_eq!(track.interpolate_at(&t), None);
    }

    #[test]
    fn interpolate_f64_both_some() {
        assert_eq!(interpolate_f64(Some(0.0), Some(10.0), 0.5), Some(5.0));
    }

    #[test]
    fn interpolate_f64_one_some() {
        assert_eq!(interpolate_f64(Some(5.0), None, 0.5), Some(5.0));
        assert_eq!(interpolate_f64(None, Some(5.0), 0.5), Some(5.0));
    }

    #[test]
    fn interpolate_f64_both_none() {
        assert_eq!(interpolate_f64(None, None, 0.5), None);
    }

    #[test]
    fn interpolate_bearing_short_arc() {
        let result = interpolate_bearing(Some(10.0), Some(20.0), 0.5);
        assert!(result.is_some());
        assert!((result.unwrap() - 15.0).abs() < 0.001);
    }

    #[test]
    fn interpolate_bearing_cross_0() {
        let result = interpolate_bearing(Some(350.0), Some(10.0), 0.5);
        assert!(result.is_some());
        let v = result.unwrap();
        assert!((v - 0.0).abs() < 0.001 || (v - 360.0).abs() < 0.001);
    }

    #[test]
    fn interpolate_bearing_one_none() {
        assert_eq!(interpolate_bearing(Some(45.0), None, 0.5), Some(45.0));
        assert_eq!(interpolate_bearing(None, Some(45.0), 0.5), Some(45.0));
    }

    #[test]
    fn interpolate_bearing_both_none() {
        assert_eq!(interpolate_bearing(None, None, 0.5), None);
    }

    #[test]
    fn gps_data_lat_only_no_coordinates() {
        let gps = GpsData { latitude: Some(34.0), longitude: None, ..Default::default() };
        assert!(!gps.has_coordinates());
    }

    #[test]
    fn gps_data_lon_only_no_coordinates() {
        let gps = GpsData { latitude: None, longitude: Some(-118.0), ..Default::default() };
        assert!(!gps.has_coordinates());
    }

    #[test]
    fn gps_data_both_none_no_coordinates() {
        let gps = GpsData { latitude: None, longitude: None, ..Default::default() };
        assert!(!gps.has_coordinates());
    }

    #[test]
    fn gps_data_coordinate_tuple_zero_zero() {
        let gps = GpsData { latitude: Some(0.0), longitude: Some(0.0), ..Default::default() };
        assert_eq!(gps.coordinate_tuple(), Some((0.0, 0.0)));
    }

    #[test]
    fn gps_data_coordinate_tuple_max_values() {
        let gps = GpsData { latitude: Some(90.0), longitude: Some(180.0), ..Default::default() };
        assert_eq!(gps.coordinate_tuple(), Some((90.0, 180.0)));
    }

    #[test]
    fn gps_data_coordinate_tuple_min_values() {
        let gps = GpsData { latitude: Some(-90.0), longitude: Some(-180.0), ..Default::default() };
        assert_eq!(gps.coordinate_tuple(), Some((-90.0, -180.0)));
    }

    #[test]
    fn interpolate_f64_negative_values() {
        assert_eq!(interpolate_f64(Some(-10.0), Some(10.0), 0.5), Some(0.0));
    }

    #[test]
    fn interpolate_f64_very_large_values() {
        let result = interpolate_f64(Some(1e10), Some(2e10), 0.5);
        assert!(result.is_some());
        assert!((result.unwrap() - 1.5e10).abs() < 1.0);
    }

    #[test]
    fn interpolate_f64_at_zero_fraction() {
        assert_eq!(interpolate_f64(Some(5.0), Some(15.0), 0.0), Some(5.0));
    }

    #[test]
    fn interpolate_f64_at_one_fraction() {
        assert_eq!(interpolate_f64(Some(5.0), Some(15.0), 1.0), Some(15.0));
    }

    #[test]
    fn interpolate_bearing_short_way_0_to_359() {
        let result = interpolate_bearing(Some(0.0), Some(359.0), 0.5);
        assert!(result.is_some());
        let v = result.unwrap();
        assert!(v < 1.0 || v > 358.0);
    }

    #[test]
    fn interpolate_bearing_short_way_359_to_0() {
        let result = interpolate_bearing(Some(359.0), Some(0.0), 0.5);
        assert!(result.is_some());
        let v = result.unwrap();
        assert!(v < 1.0 || v > 358.0);
    }

    #[test]
    fn interpolate_bearing_180_to_181() {
        let result = interpolate_bearing(Some(180.0), Some(181.0), 0.5);
        assert!(result.is_some());
        assert!((result.unwrap() - 180.5).abs() < 0.1);
    }

    #[test]
    fn interpolate_bearing_0_to_180() {
        let result = interpolate_bearing(Some(0.0), Some(180.0), 0.5);
        assert!(result.is_some());
        assert!((result.unwrap() - 90.0).abs() < 0.1);
    }

    #[test]
    fn gpx_track_empty_points_no_interpolation() {
        let track = GpxTrack { name: None, points: vec![], duration_seconds: None, distance_meters: None };
        let t = chrono::Utc::now();
        assert_eq!(track.interpolate_at(&t), None);
    }

    #[test]
    fn gpx_track_all_points_no_timestamps() {
        let track = GpxTrack {
            name: None,
            points: vec![
                GpxPoint { latitude: 40.0, longitude: -74.0, elevation: None, timestamp: None, speed: None, bearing: None },
            ],
            duration_seconds: None,
            distance_meters: None,
        };
        let t = chrono::Utc::now();
        assert_eq!(track.interpolate_at(&t), None);
    }

    #[test]
    fn gpx_track_single_point_with_timestamp() {
        let t1 = chrono::Utc::now();
        let track = GpxTrack {
            name: None,
            points: vec![
                GpxPoint { latitude: 40.0, longitude: -74.0, elevation: None, timestamp: Some(t1), speed: None, bearing: None },
            ],
            duration_seconds: None,
            distance_meters: None,
        };
        let result = track.interpolate_at(&t1);
        assert!(result.is_some());
        assert!((result.unwrap().latitude - 40.0).abs() < 0.0001);
    }

    #[test]
    fn gpx_track_duplicate_timestamps() {
        let t1 = chrono::Utc::now();
        let track = GpxTrack {
            name: None,
            points: vec![
                GpxPoint { latitude: 40.0, longitude: -74.0, elevation: None, timestamp: Some(t1), speed: None, bearing: None },
                GpxPoint { latitude: 41.0, longitude: -73.0, elevation: None, timestamp: Some(t1), speed: None, bearing: None },
            ],
            duration_seconds: None,
            distance_meters: None,
        };
        let result = track.interpolate_at(&t1);
        assert!(result.is_some());
    }

    #[test]
    fn gpx_track_interpolate_exact_start() {
        let t1 = chrono::Utc::now();
        let t2 = t1 + chrono::Duration::hours(1);
        let track = GpxTrack {
            name: None,
            points: vec![
                GpxPoint { latitude: 10.0, longitude: 20.0, elevation: None, timestamp: Some(t1), speed: None, bearing: None },
                GpxPoint { latitude: 20.0, longitude: 30.0, elevation: None, timestamp: Some(t2), speed: None, bearing: None },
            ],
            duration_seconds: None,
            distance_meters: None,
        };
        let result = track.interpolate_at(&t1);
        assert!(result.is_some());
        let pt = result.unwrap();
        assert!((pt.latitude - 10.0).abs() < 0.0001);
    }

    #[test]
    fn gpx_track_interpolate_exact_end() {
        let t1 = chrono::Utc::now();
        let t2 = t1 + chrono::Duration::hours(1);
        let track = GpxTrack {
            name: None,
            points: vec![
                GpxPoint { latitude: 10.0, longitude: 20.0, elevation: None, timestamp: Some(t1), speed: None, bearing: None },
                GpxPoint { latitude: 20.0, longitude: 30.0, elevation: None, timestamp: Some(t2), speed: None, bearing: None },
            ],
            duration_seconds: None,
            distance_meters: None,
        };
        let result = track.interpolate_at(&t2);
        assert!(result.is_some());
        let pt = result.unwrap();
        assert!((pt.latitude - 20.0).abs() < 0.0001);
    }

    #[test]
    fn exif_data_default_all_none() {
        let exif = ExifData::default();
        assert!(exif.make.is_none());
        assert!(exif.model.is_none());
        assert!(exif.iso.is_none());
    }

    #[test]
    fn exif_data_all_fields_some() {
        let now = chrono::Utc::now();
        let exif = ExifData {
            make: Some("Canon".into()),
            model: Some("EOS R5".into()),
            lens_model: Some("RF24-105".into()),
            serial_number: Some("12345".into()),
            software: Some("LR".into()),
            artist: Some("Photographer".into()),
            copyright: Some("2024".into()),
            image_description: Some("Sunset".into()),
            orientation: Some(1),
            date_time_original: Some(now),
            date_time_digitized: Some(now),
            sub_sec_time_original: Some("00".into()),
            offset_time_original: Some("+00:00".into()),
            exposure_time: Some("1/125".into()),
            f_number: Some("5.6".into()),
            iso: Some(400),
            focal_length: Some("50".into()),
            focal_length_35mm: Some(50),
            aperture_value: Some("5.6".into()),
            shutter_speed_value: Some("1/125".into()),
            brightness_value: Some("0".into()),
            exposure_bias: Some("0".into()),
            metering_mode: Some(5),
            flash: Some(0),
            exposure_program: Some(3),
            white_balance: Some(0),
            image_width: Some(8192),
            image_height: Some(5464),
            color_space: Some(1),
            bits_per_sample: Some(vec![8, 8, 8]),
            compression: Some(6),
            maker_note: Some(vec![]),
            raw_tags: vec![],
        };
        assert_eq!(exif.make, Some("Canon".into()));
        assert_eq!(exif.iso, Some(400));
        assert_eq!(exif.focal_length, Some("50".into()));
    }

    #[test]
    fn exif_data_serde_roundtrip() {
        let exif = ExifData {
            make: Some("Sony".into()),
            model: Some("ILCE-7RM5".into()),
            iso: Some(100),
            ..Default::default()
        };
        let json = serde_json::to_string(&exif).unwrap();
        let exif2: ExifData = serde_json::from_str(&json).unwrap();
        assert_eq!(exif2.make, Some("Sony".into()));
        assert_eq!(exif2.iso, Some(100));
    }

    #[test]
    fn xmp_data_default_empty() {
        let xmp = XmpData::default();
        assert!(xmp.creator.is_none());
        assert!(xmp.subject.is_empty());
    }

    #[test]
    fn xmp_data_serde_roundtrip() {
        let xmp = XmpData {
            creator: Some("Author".into()),
            title: Some("Title".into()),
            rating: Some(5),
            ..Default::default()
        };
        let json = serde_json::to_string(&xmp).unwrap();
        let xmp2: XmpData = serde_json::from_str(&json).unwrap();
        assert_eq!(xmp2.title, Some("Title".into()));
        assert_eq!(xmp2.rating, Some(5));
    }

    #[test]
    fn iptc_data_default_empty() {
        let iptc = IptcData::default();
        assert!(iptc.caption.is_none());
        assert!(iptc.keywords.is_empty());
    }

    #[test]
    fn iptc_data_serde_roundtrip() {
        let iptc = IptcData {
            city: Some("NYC".into()),
            country: Some("USA".into()),
            ..Default::default()
        };
        let json = serde_json::to_string(&iptc).unwrap();
        let iptc2: IptcData = serde_json::from_str(&json).unwrap();
        assert_eq!(iptc2.city, Some("NYC".into()));
    }

    #[test]
    fn gps_data_all_fields() {
        let gps = GpsData {
            latitude: Some(48.8566),
            longitude: Some(2.3522),
            altitude: Some(35.0),
            speed: Some(5.0),
            track: Some(270.0),
            ..Default::default()
        };
        assert!(gps.has_coordinates());
        assert_eq!(gps.altitude, Some(35.0));
    }

    #[test]
    fn metadata_scoped_fields() {
        assert_eq!(MetadataScope::EXIF, MetadataScope::EXIF);
        assert_ne!(MetadataScope::All, MetadataScope::GPS);
    }

    #[test]
    fn custom_tag_creation() {
        let tag = CustomTag {
            key: "my_key".into(),
            value: "my_value".into(),
            namespace: Some("ns".into()),
        };
        assert_eq!(tag.key, "my_key");
        assert_eq!(tag.namespace, Some("ns".into()));
    }

    #[test]
    fn raw_exif_tag_fields() {
        let tag = RawExifTag { tag: "0x829A".into(), group: "ExifIFD".into(), value: "1/125".into() };
        assert_eq!(tag.tag, "0x829A");
    }

    #[test]
    fn raw_xmp_property_fields() {
        let prop = RawXmpProperty { namespace: "dc".into(), name: "creator".into(), value: "me".into() };
        assert_eq!(prop.namespace, "dc");
    }

    #[test]
    fn raw_iptc_tag_fields() {
        let tag = RawIptcTag { record: 2, dataset: 80, name: "Byline".into(), value: "Author".into() };
        assert_eq!(tag.record, 2);
        assert_eq!(tag.dataset, 80);
    }

    #[test]
    fn metadata_target_fields() {
        use crate::types::ImageFormat;
        let target = MetadataTarget { path: "/tmp/img.jpg".into(), format: ImageFormat::JPEG };
        assert_eq!(target.path, "/tmp/img.jpg");
        assert_eq!(target.format, ImageFormat::JPEG);
    }

    #[test]
    fn metadata_write_report_defaults() {
        let report = MetadataWriteReport { tags_written: 10, tags_skipped: 2, warnings: vec![] };
        assert_eq!(report.tags_written, 10);
        assert_eq!(report.tags_skipped, 2);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn gpx_track_with_name_and_duration() {
        let track = GpxTrack {
            name: Some("Morning Run".into()),
            points: vec![],
            duration_seconds: Some(3600.0),
            distance_meters: Some(10000.0),
        };
        assert_eq!(track.name, Some("Morning Run".into()));
        assert_eq!(track.distance_meters, Some(10000.0));
    }

    #[test]
    fn gpx_point_all_fields_some() {
        let ts = chrono::Utc::now();
        let pt = GpxPoint {
            latitude: 34.0,
            longitude: -118.0,
            elevation: Some(100.0),
            timestamp: Some(ts),
            speed: Some(5.5),
            bearing: Some(90.0),
        };
        assert!(pt.timestamp.is_some());
        assert_eq!(pt.speed, Some(5.5));
    }

    #[test]
    fn gpx_point_all_fields_none() {
        let pt = GpxPoint {
            latitude: 0.0, longitude: 0.0,
            elevation: None, timestamp: None, speed: None, bearing: None,
        };
        assert!(pt.elevation.is_none());
    }
}
