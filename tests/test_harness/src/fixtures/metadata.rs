use chrono::{DateTime, TimeZone, Utc};
use photopipeline_core::*;

fn ts(year: i32, month: u32, day: u32, hour: u32, min: u32, sec: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, min, sec)
        .unwrap()
}

pub fn exif_sony_a7r5() -> ExifData {
    ExifData {
        make: Some("SONY".into()),
        model: Some("ILCE-7RM5".into()),
        lens_model: Some("FE 24-70mm F2.8 GM II".into()),
        serial_number: Some("1234567".into()),
        software: Some("ILCE-7RM5 v1.01".into()),
        artist: Some("Test Photographer".into()),
        copyright: Some("2024 Test".into()),
        image_description: Some("Sony test image".into()),
        orientation: Some(1),
        date_time_original: Some(ts(2024, 6, 15, 14, 30, 0)),
        date_time_digitized: Some(ts(2024, 6, 15, 14, 30, 0)),
        sub_sec_time_original: Some("123".into()),
        offset_time_original: Some("+08:00".into()),
        exposure_time: Some("1/250".into()),
        f_number: Some("4.0".into()),
        iso: Some(100),
        focal_length: Some("35".into()),
        focal_length_35mm: Some(35),
        aperture_value: Some("4.0".into()),
        shutter_speed_value: Some("1/250".into()),
        brightness_value: Some("8.5".into()),
        exposure_bias: Some("0".into()),
        metering_mode: Some(5),
        flash: Some(0),
        exposure_program: Some(3),
        white_balance: Some(0),
        image_width: Some(9504),
        image_height: Some(6336),
        color_space: Some(1),
        bits_per_sample: Some(vec![8, 8, 8]),
        compression: Some(6),
        maker_note: Some(vec![0x00, 0x01, 0x02]),
        raw_tags: vec![
            RawExifTag {
                tag: "0x829A".into(),
                group: "ExifIFD".into(),
                value: "1/250".into(),
            },
            RawExifTag {
                tag: "0x829D".into(),
                group: "ExifIFD".into(),
                value: "4.0".into(),
            },
        ],
    }
}

pub fn exif_canon_r5(iso: u32) -> ExifData {
    ExifData {
        make: Some("Canon".into()),
        model: Some("EOS R5".into()),
        lens_model: Some("RF24-105mm F4 L IS USM".into()),
        serial_number: Some("CN12345678".into()),
        software: Some("Adobe Lightroom".into()),
        artist: Some("Canon Photographer".into()),
        copyright: Some("2024 Canon Test".into()),
        image_description: Some("Canon test image".into()),
        orientation: Some(1),
        date_time_original: Some(ts(2024, 6, 15, 10, 0, 0)),
        date_time_digitized: Some(ts(2024, 6, 15, 10, 0, 0)),
        sub_sec_time_original: Some("00".into()),
        offset_time_original: Some("+00:00".into()),
        exposure_time: Some("1/500".into()),
        f_number: Some("5.6".into()),
        iso: Some(iso),
        focal_length: Some("50".into()),
        focal_length_35mm: Some(50),
        aperture_value: Some("5.6".into()),
        shutter_speed_value: Some("1/500".into()),
        brightness_value: Some("7.0".into()),
        exposure_bias: Some("-0.3".into()),
        metering_mode: Some(5),
        flash: Some(0),
        exposure_program: Some(3),
        white_balance: Some(0),
        image_width: Some(8192),
        image_height: Some(5464),
        color_space: Some(1),
        bits_per_sample: Some(vec![8, 8, 8]),
        compression: Some(6),
        maker_note: None,
        raw_tags: vec![],
    }
}

pub fn exif_nikon_z9(focal_length: &str) -> ExifData {
    ExifData {
        make: Some("NIKON CORPORATION".into()),
        model: Some("NIKON Z 9".into()),
        lens_model: Some("NIKKOR Z 70-200mm f/2.8 VR S".into()),
        serial_number: Some("NK9000001".into()),
        software: Some("Nikon Z 9 v4.0".into()),
        artist: Some("Nikon Photographer".into()),
        copyright: Some("2024 Nikon Test".into()),
        image_description: Some("Nikon Z9 test image".into()),
        orientation: Some(1),
        date_time_original: Some(ts(2024, 6, 15, 18, 45, 0)),
        date_time_digitized: Some(ts(2024, 6, 15, 18, 45, 0)),
        sub_sec_time_original: Some("456".into()),
        offset_time_original: Some("+09:00".into()),
        exposure_time: Some("1/1000".into()),
        f_number: Some("2.8".into()),
        iso: Some(800),
        focal_length: Some(focal_length.to_string()),
        focal_length_35mm: Some(focal_length.parse().unwrap_or(200)),
        aperture_value: Some("2.8".into()),
        shutter_speed_value: Some("1/1000".into()),
        brightness_value: Some("5.0".into()),
        exposure_bias: Some("+0.7".into()),
        metering_mode: Some(5),
        flash: Some(0),
        exposure_program: Some(3),
        white_balance: Some(0),
        image_width: Some(8256),
        image_height: Some(5504),
        color_space: Some(1),
        bits_per_sample: Some(vec![8, 8, 8]),
        compression: Some(6),
        maker_note: None,
        raw_tags: vec![],
    }
}

pub fn gps_beijing() -> GpsData {
    GpsData {
        latitude: Some(39.9042),
        latitude_ref: Some("N".into()),
        longitude: Some(116.4074),
        longitude_ref: Some("E".into()),
        altitude: Some(43.5),
        altitude_ref: Some(0),
        timestamp: Some(ts(2024, 6, 15, 8, 0, 0)),
        img_direction: Some(180.0),
        img_direction_ref: Some("T".into()),
        map_datum: Some("WGS-84".into()),
        satellites: Some("12".into()),
        status: Some("A".into()),
        measure_mode: Some("3".into()),
        dop: Some(1.5),
        speed: Some(0.0),
        speed_ref: Some("K".into()),
        track: Some(90.0),
        track_ref: Some("T".into()),
        dest_bearing: None,
        dest_bearing_ref: None,
        dest_distance: None,
        dest_latitude: None,
        dest_longitude: None,
        processing_method: Some("GPS".into()),
        area_information: None,
        date_stamp: Some("2024:06:15".into()),
    }
}

pub fn gps_nyc() -> GpsData {
    GpsData {
        latitude: Some(40.7128),
        latitude_ref: Some("N".into()),
        longitude: Some(-74.0060),
        longitude_ref: Some("W".into()),
        altitude: Some(10.0),
        altitude_ref: Some(0),
        timestamp: Some(ts(2024, 7, 4, 12, 0, 0)),
        img_direction: Some(270.0),
        img_direction_ref: Some("T".into()),
        map_datum: Some("WGS-84".into()),
        satellites: Some("8".into()),
        status: Some("A".into()),
        measure_mode: Some("3".into()),
        dop: Some(2.0),
        speed: Some(5.5),
        speed_ref: Some("K".into()),
        track: Some(270.0),
        track_ref: Some("T".into()),
        dest_bearing: None,
        dest_bearing_ref: None,
        dest_distance: None,
        dest_latitude: None,
        dest_longitude: None,
        processing_method: Some("GPS".into()),
        area_information: None,
        date_stamp: Some("2024:07:04".into()),
    }
}

pub fn gps_tokyo() -> GpsData {
    GpsData {
        latitude: Some(35.6762),
        latitude_ref: Some("N".into()),
        longitude: Some(139.6503),
        longitude_ref: Some("E".into()),
        altitude: Some(40.0),
        altitude_ref: Some(0),
        timestamp: Some(ts(2024, 8, 1, 9, 0, 0)),
        img_direction: Some(45.0),
        img_direction_ref: Some("T".into()),
        map_datum: Some("WGS-84".into()),
        satellites: Some("10".into()),
        status: Some("A".into()),
        measure_mode: Some("3".into()),
        dop: Some(1.0),
        speed: Some(2.0),
        speed_ref: Some("K".into()),
        track: Some(45.0),
        track_ref: Some("T".into()),
        dest_bearing: None,
        dest_bearing_ref: None,
        dest_distance: None,
        dest_latitude: None,
        dest_longitude: None,
        processing_method: Some("GPS".into()),
        area_information: None,
        date_stamp: Some("2024:08:01".into()),
    }
}

pub fn gps_london() -> GpsData {
    GpsData {
        latitude: Some(51.5074),
        latitude_ref: Some("N".into()),
        longitude: Some(-0.1278),
        longitude_ref: Some("W".into()),
        altitude: Some(35.0),
        altitude_ref: Some(0),
        timestamp: Some(ts(2024, 9, 1, 14, 0, 0)),
        img_direction: Some(0.0),
        img_direction_ref: Some("T".into()),
        map_datum: Some("WGS-84".into()),
        satellites: Some("11".into()),
        status: Some("A".into()),
        measure_mode: Some("3".into()),
        dop: Some(0.8),
        speed: Some(3.5),
        speed_ref: Some("K".into()),
        track: Some(0.0),
        track_ref: Some("T".into()),
        dest_bearing: None,
        dest_bearing_ref: None,
        dest_distance: None,
        dest_latitude: None,
        dest_longitude: None,
        processing_method: Some("GPS".into()),
        area_information: None,
        date_stamp: Some("2024:09:01".into()),
    }
}

pub fn full_metadata() -> Metadata {
    Metadata {
        exif: Some(exif_sony_a7r5()),
        xmp: Some(XmpData {
            creator: Some("Test Author".into()),
            rights: Some("Copyright 2024".into()),
            title: Some("Test Image".into()),
            description: Some("Full metadata test image".into()),
            create_date: Some(ts(2024, 6, 15, 14, 30, 0)),
            modify_date: Some(ts(2024, 6, 15, 14, 35, 0)),
            rating: Some(5),
            label: Some("Approved".into()),
            subject: vec!["landscape".into(), "sunset".into()],
            raw_properties: vec![RawXmpProperty {
                namespace: "dc".into(),
                name: "format".into(),
                value: "image/jpeg".into(),
            }],
        }),
        iptc: Some(IptcData {
            creator: Some("Test Author".into()),
            headline: Some("Test Headline".into()),
            caption: Some("A test caption".into()),
            keywords: vec!["test".into(), "photography".into()],
            copyright_notice: Some("Copyright 2024".into()),
            date_created: Some(ts(2024, 6, 15, 0, 0, 0)),
            time_created: Some("14:30:00+08:00".into()),
            city: Some("Beijing".into()),
            state: Some("Beijing".into()),
            country: Some("China".into()),
            raw_tags: vec![RawIptcTag {
                record: 2,
                dataset: 80,
                name: "Byline".into(),
                value: "Test Author".into(),
            }],
        }),
        gps: Some(gps_beijing()),
        custom: vec![CustomTag {
            key: "project".into(),
            value: "photopipeline".into(),
            namespace: Some("app".into()),
        }],
    }
}

pub fn empty_metadata() -> Metadata {
    Metadata::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exif_sony_a7r5_has_make() {
        let exif = exif_sony_a7r5();
        assert_eq!(exif.make, Some("SONY".into()));
        assert_eq!(exif.iso, Some(100));
    }

    #[test]
    fn exif_canon_r5_custom_iso() {
        let exif = exif_canon_r5(6400);
        assert_eq!(exif.make, Some("Canon".into()));
        assert_eq!(exif.iso, Some(6400));
    }

    #[test]
    fn exif_canon_r5_low_iso() {
        let exif = exif_canon_r5(50);
        assert_eq!(exif.iso, Some(50));
    }

    #[test]
    fn exif_nikon_z9_variable_focal() {
        let exif = exif_nikon_z9("200");
        assert_eq!(exif.focal_length, Some("200".into()));
    }

    #[test]
    fn gps_beijing_coordinates() {
        let gps = gps_beijing();
        assert!(gps.has_coordinates());
        let (lat, lon) = gps.coordinate_tuple().unwrap();
        assert!((lat - 39.9042).abs() < 0.0001);
        assert!((lon - 116.4074).abs() < 0.0001);
    }

    #[test]
    fn gps_nyc_coordinates() {
        let gps = gps_nyc();
        assert!(gps.has_coordinates());
        let (lat, lon) = gps.coordinate_tuple().unwrap();
        assert!((lat - 40.7128).abs() < 0.0001);
        assert!((lon - (-74.0060)).abs() < 0.0001);
    }

    #[test]
    fn gps_tokyo_coordinates() {
        let gps = gps_tokyo();
        assert!(gps.has_coordinates());
    }

    #[test]
    fn gps_london_coordinates() {
        let gps = gps_london();
        assert!(gps.has_coordinates());
        let (lat, _lon) = gps.coordinate_tuple().unwrap();
        assert!((lat - 51.5074).abs() < 0.0001);
    }

    #[test]
    fn full_metadata_has_all_sections() {
        let m = full_metadata();
        assert!(m.exif.is_some());
        assert!(m.xmp.is_some());
        assert!(m.iptc.is_some());
        assert!(m.gps.is_some());
        assert!(!m.custom.is_empty());
    }

    #[test]
    fn empty_metadata_is_default() {
        let m = empty_metadata();
        assert_eq!(m, Metadata::default());
    }
}
