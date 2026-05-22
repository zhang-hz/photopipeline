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
            (Some(b), Some(a)) if b.timestamp != a.timestamp => {
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
