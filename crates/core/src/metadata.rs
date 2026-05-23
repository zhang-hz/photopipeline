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
}
