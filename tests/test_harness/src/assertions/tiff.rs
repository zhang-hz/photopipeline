pub fn assert_valid_tiff(data: &[u8]) {
    assert!(data.len() >= 8, "TIFF data too short: {} bytes", data.len());

    let byte_order = &data[0..2];
    let little_endian = match byte_order {
        b"II" => true,
        b"MM" => false,
        _ => panic!("invalid TIFF byte order marker: {:02X?}", byte_order),
    };

    let magic = if little_endian {
        u16::from_le_bytes([data[2], data[3]])
    } else {
        u16::from_be_bytes([data[2], data[3]])
    };
    assert_eq!(magic, 42, "invalid TIFF magic number: {}", magic);

    let ifd_offset = if little_endian {
        u32::from_le_bytes([data[4], data[5], data[6], data[7]])
    } else {
        u32::from_be_bytes([data[4], data[5], data[6], data[7]])
    } as usize;
    assert!(ifd_offset < data.len(), "IFD offset out of bounds");

    let ifd_count = if little_endian {
        u16::from_le_bytes([data[ifd_offset], data[ifd_offset + 1]])
    } else {
        u16::from_be_bytes([data[ifd_offset], data[ifd_offset + 1]])
    } as usize;

    let tag_start = ifd_offset + 2;
    let tag_end = tag_start + ifd_count * 12;
    assert!(
        tag_end <= data.len(),
        "IFD tags exceed data boundary: {} > {}",
        tag_end,
        data.len()
    );

    let required_tags = [
        256u16, 257, 258, 259, 262, 273, 277, 278, 279, 282, 283, 296,
    ];
    let mut found_tags = Vec::new();

    for i in 0..ifd_count {
        let pos = tag_start + i * 12;
        let tag_id = if little_endian {
            u16::from_le_bytes([data[pos], data[pos + 1]])
        } else {
            u16::from_be_bytes([data[pos], data[pos + 1]])
        };
        found_tags.push(tag_id);
    }

    for required in &required_tags {
        assert!(
            found_tags.contains(required),
            "missing required TIFF tag {}",
            required
        );
    }
}

pub fn assert_tiff_tag(data: &[u8], tag: u16, expected_value: &[u8]) {
    assert!(data.len() >= 8, "TIFF data too short");
    let little_endian = match &data[0..2] {
        b"II" => true,
        b"MM" => false,
        _ => panic!("invalid TIFF byte order"),
    };

    let ifd_offset = if little_endian {
        u32::from_le_bytes([data[4], data[5], data[6], data[7]])
    } else {
        u32::from_be_bytes([data[4], data[5], data[6], data[7]])
    } as usize;

    let ifd_count = if little_endian {
        u16::from_le_bytes([data[ifd_offset], data[ifd_offset + 1]])
    } else {
        u16::from_be_bytes([data[ifd_offset], data[ifd_offset + 1]])
    } as usize;

    let tag_start = ifd_offset + 2;
    for i in 0..ifd_count {
        let pos = tag_start + i * 12;
        let tag_id = if little_endian {
            u16::from_le_bytes([data[pos], data[pos + 1]])
        } else {
            u16::from_be_bytes([data[pos], data[pos + 1]])
        };
        if tag_id == tag {
            let dtype = if little_endian {
                u16::from_le_bytes([data[pos + 2], data[pos + 3]])
            } else {
                u16::from_be_bytes([data[pos + 2], data[pos + 3]])
            };
            let count = if little_endian {
                u32::from_le_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
            } else {
                u32::from_be_bytes([data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]])
            } as usize;

            let byte_size_per_value: usize = match dtype {
                1 | 2 | 6 | 7 => 1,
                3 | 8 => 2,
                4 | 9 | 11 => 4,
                5 | 10 | 12 => 8,
                _ => 1,
            };
            let total_bytes = count * byte_size_per_value;

            if total_bytes <= 4 {
                assert_eq!(
                    &data[pos + 8..pos + 8 + total_bytes],
                    expected_value,
                    "tag {} inline value mismatch",
                    tag
                );
            } else {
                let value_offset = if little_endian {
                    u32::from_le_bytes([
                        data[pos + 8],
                        data[pos + 9],
                        data[pos + 10],
                        data[pos + 11],
                    ])
                } else {
                    u32::from_be_bytes([
                        data[pos + 8],
                        data[pos + 9],
                        data[pos + 10],
                        data[pos + 11],
                    ])
                } as usize;
                assert!(
                    value_offset + expected_value.len() <= data.len(),
                    "tag {} value offset out of bounds: {} + {} > {}",
                    tag,
                    value_offset,
                    expected_value.len(),
                    data.len()
                );
                assert_eq!(
                    &data[value_offset..value_offset + expected_value.len()],
                    expected_value,
                    "tag {} value mismatch",
                    tag
                );
            }
            return;
        }
    }
    panic!("tag {} not found in TIFF", tag);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_minimal_tiff() -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(b"II");
        data.extend_from_slice(&42u16.to_le_bytes());
        data.extend_from_slice(&8u32.to_le_bytes());

        let num_tags: u16 = 12;
        let ifd_header_pos = data.len();
        data.extend_from_slice(&num_tags.to_le_bytes());

        let tags: [(u16, u16, u32); 12] = [
            (256, 3, 1),
            (257, 3, 1),
            (258, 3, 3),
            (259, 3, 1),
            (262, 3, 1),
            (273, 4, 1),
            (277, 3, 1),
            (278, 4, 1),
            (279, 4, 1),
            (282, 5, 1),
            (283, 5, 1),
            (296, 3, 1),
        ];

        let value_area_base = ifd_header_pos + 2 + 12 * 12;
        let mut next_value_offset = value_area_base;
        let mut value_data: Vec<u8> = Vec::new();

        for &(tag_id, dtype, count) in &tags {
            let value_bytes = match dtype {
                3 => count as usize * 2,
                4 => count as usize * 4,
                5 => count as usize * 8,
                _ => 0,
            };
            let value_fits = value_bytes <= 4;

            data.extend_from_slice(&tag_id.to_le_bytes());
            data.extend_from_slice(&dtype.to_le_bytes());
            data.extend_from_slice(&count.to_le_bytes());

            if value_fits {
                let inline: u32 = if tag_id == 296 { 2 } else { 1 };
                data.extend_from_slice(&inline.to_le_bytes());
            } else {
                let off = next_value_offset as u32;
                data.extend_from_slice(&off.to_le_bytes());
                next_value_offset += value_bytes;
                value_data.resize(next_value_offset - value_area_base, 0);
            }
        }

        while data.len() < value_area_base {
            data.push(0);
        }

        for &(tag_id, dtype, count) in &tags {
            let value_bytes = match dtype {
                3 => count as usize * 2,
                4 => count as usize * 4,
                5 => count as usize * 8,
                _ => 0,
            };
            if value_bytes > 4 {
                let offset_in_value_area = {
                    let tag_start = ifd_header_pos + 2;
                    let mut pos = tag_start;
                    for &(tid, _dt, _ct) in &tags {
                        if tid == tag_id {
                            break;
                        }
                        pos += 12;
                    }
                    let off_bytes = &data[pos + 8..pos + 12];
                    u32::from_le_bytes([off_bytes[0], off_bytes[1], off_bytes[2], off_bytes[3]])
                        as usize
                        - value_area_base
                };
                match tag_id {
                    258 => {
                        for (i, &v) in [8u16, 8, 8].iter().enumerate() {
                            let b = v.to_le_bytes();
                            let start = offset_in_value_area + i * 2;
                            value_data[start..start + 2].copy_from_slice(&b);
                        }
                    }
                    282 | 283 => {
                        let b72 = 72u32.to_le_bytes();
                        let b1 = 1u32.to_le_bytes();
                        value_data[offset_in_value_area..offset_in_value_area + 4]
                            .copy_from_slice(&b72);
                        value_data[offset_in_value_area + 4..offset_in_value_area + 8]
                            .copy_from_slice(&b1);
                    }
                    _ => {}
                }
            }
        }

        data.extend_from_slice(&value_data);
        data
    }

    #[test]
    fn assert_valid_tiff_minimal() {
        let tiff = make_minimal_tiff();
        assert_valid_tiff(&tiff);
    }

    #[test]
    #[should_panic]
    fn assert_valid_tiff_bad_magic() {
        let mut tiff = make_minimal_tiff();
        tiff[2] = 0;
        assert_valid_tiff(&tiff);
    }

    #[test]
    #[should_panic]
    fn assert_valid_tiff_too_short() {
        assert_valid_tiff(&[0u8; 4]);
    }

    #[test]
    fn assert_tiff_tag_valid() {
        let tiff = make_minimal_tiff();
        assert_tiff_tag(&tiff, 256, &[1, 0]);
    }

    #[test]
    #[should_panic]
    fn assert_tiff_tag_missing() {
        let tiff = make_minimal_tiff();
        assert_tiff_tag(&tiff, 999, &[]);
    }
}
