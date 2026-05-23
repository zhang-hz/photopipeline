pub fn assert_valid_png(data: &[u8]) {
    assert!(
        data.len() >= 8,
        "data too short to be PNG: {} bytes",
        data.len()
    );

    let signature: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    assert_eq!(
        &data[0..8],
        &signature,
        "invalid PNG signature: {:02X?}",
        &data[0..8]
    );

    let mut pos = 8;
    let mut chunks_seen = 0;
    let mut has_ihdr = false;
    let mut last_chunk_type: Option<[u8; 4]> = None;

    while pos + 12 <= data.len() {
        let length = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let chunk_type: [u8; 4] = [data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]];
        let crc_pos = pos + 8 + length as usize;
        let chunk_end = crc_pos + 4;

        if chunk_end > data.len() {
            panic!("PNG chunk at {} exceeds data boundary", pos);
        }

        let expected_crc = u32::from_be_bytes([
            data[crc_pos],
            data[crc_pos + 1],
            data[crc_pos + 2],
            data[crc_pos + 3],
        ]);
        let computed_crc = crc32(&data[pos + 4..pos + 8 + length as usize]);
        assert_eq!(
            computed_crc,
            expected_crc,
            "CRC mismatch for chunk {:?} at offset {}: computed {:08X}, stored {:08X}",
            std::str::from_utf8(&chunk_type).unwrap_or("???"),
            pos,
            computed_crc,
            expected_crc
        );

        if chunks_seen == 0 {
            assert_eq!(
                &chunk_type,
                b"IHDR",
                "first PNG chunk must be IHDR, got {:?}",
                std::str::from_utf8(&chunk_type).unwrap_or("???")
            );
            has_ihdr = true;
        }

        last_chunk_type = Some(chunk_type);
        chunks_seen += 1;

        if &chunk_type == b"IEND" {
            assert_eq!(length, 0, "IEND chunk length must be 0, got {}", length);
            break;
        }

        pos = chunk_end;
    }

    assert!(has_ihdr, "PNG missing IHDR chunk");
    assert_eq!(
        last_chunk_type,
        Some(*b"IEND"),
        "last PNG chunk must be IEND, got {:?}",
        last_chunk_type.map(|ct| std::str::from_utf8(&ct).unwrap_or("???").to_string())
    );
}

pub fn assert_png_chunk(data: &[u8], chunk_type: &[u8; 4]) {
    let signature: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    assert_eq!(&data[0..8], &signature, "not a valid PNG (bad signature)");

    let mut pos = 8;
    while pos + 12 <= data.len() {
        let length = u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let ct: [u8; 4] = [data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7]];
        if &ct == chunk_type {
            return;
        }
        let chunk_end = pos + 12 + length as usize;
        if chunk_end > data.len() {
            break;
        }
        if &ct == b"IEND" {
            break;
        }
        pos = chunk_end;
    }
    panic!(
        "PNG chunk {:?} not found",
        std::str::from_utf8(chunk_type).unwrap_or("???")
    );
}

pub fn assert_png_ihdr(
    data: &[u8],
    expected_w: u32,
    expected_h: u32,
    expected_bit_depth: u8,
    expected_color_type: u8,
) {
    let signature: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    assert_eq!(&data[0..8], &signature, "not a valid PNG (bad signature)");

    let length = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    assert_eq!(length, 13, "IHDR length must be 13, got {}", length);

    let chunk_type: [u8; 4] = [data[12], data[13], data[14], data[15]];
    assert_eq!(&chunk_type, b"IHDR", "not an IHDR chunk");

    let width = u32::from_be_bytes([data[16], data[17], data[18], data[19]]);
    assert_eq!(
        width, expected_w,
        "IHDR width mismatch: expected {}, got {}",
        expected_w, width
    );

    let height = u32::from_be_bytes([data[20], data[21], data[22], data[23]]);
    assert_eq!(
        height, expected_h,
        "IHDR height mismatch: expected {}, got {}",
        expected_h, height
    );

    let bit_depth = data[24];
    assert_eq!(
        bit_depth, expected_bit_depth,
        "IHDR bit_depth mismatch: expected {}, got {}",
        expected_bit_depth, bit_depth
    );

    let color_type = data[25];
    assert_eq!(
        color_type, expected_color_type,
        "IHDR color_type mismatch: expected {}, got {}",
        expected_color_type, color_type
    );
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_minimal_png() -> Vec<u8> {
        let mut data = Vec::new();
        let signature: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
        data.extend_from_slice(&signature);

        let mut ihdr_data = Vec::new();
        ihdr_data.extend_from_slice(&100u32.to_be_bytes());
        ihdr_data.extend_from_slice(&200u32.to_be_bytes());
        ihdr_data.push(8u8);
        ihdr_data.push(2u8);
        ihdr_data.push(0u8);
        ihdr_data.push(0u8);
        ihdr_data.push(0u8);

        let mut chunk = Vec::new();
        chunk.extend_from_slice(&(ihdr_data.len() as u32).to_be_bytes());
        chunk.extend_from_slice(b"IHDR");
        chunk.extend_from_slice(&ihdr_data);
        let crc = crc32(&chunk[4..]);
        chunk.extend_from_slice(&crc.to_be_bytes());
        data.extend_from_slice(&chunk);

        let mut iend_chunk = Vec::new();
        iend_chunk.extend_from_slice(&0u32.to_be_bytes());
        iend_chunk.extend_from_slice(b"IEND");
        let crc = crc32(&iend_chunk[4..]);
        iend_chunk.extend_from_slice(&crc.to_be_bytes());
        data.extend_from_slice(&iend_chunk);

        data
    }

    #[test]
    fn assert_valid_png_minimal() {
        let png = make_minimal_png();
        assert_valid_png(&png);
    }

    #[test]
    #[should_panic]
    fn assert_valid_png_bad_signature() {
        let mut png = make_minimal_png();
        png[0] = 0;
        assert_valid_png(&png);
    }

    #[test]
    #[should_panic]
    fn assert_valid_png_too_short() {
        assert_valid_png(&[0u8; 4]);
    }

    #[test]
    fn assert_png_chunk_ihdr() {
        let png = make_minimal_png();
        assert_png_chunk(&png, b"IHDR");
    }

    #[test]
    fn assert_png_chunk_iend() {
        let png = make_minimal_png();
        assert_png_chunk(&png, b"IEND");
    }

    #[test]
    #[should_panic]
    fn assert_png_chunk_missing() {
        let png = make_minimal_png();
        assert_png_chunk(&png, b"tEXt");
    }

    #[test]
    fn assert_png_ihdr_dimensions() {
        let png = make_minimal_png();
        assert_png_ihdr(&png, 100, 200, 8, 2);
    }

    #[test]
    #[should_panic]
    fn assert_png_ihdr_wrong_width() {
        let png = make_minimal_png();
        assert_png_ihdr(&png, 999, 200, 8, 2);
    }

    #[test]
    #[should_panic]
    fn assert_png_ihdr_wrong_height() {
        let png = make_minimal_png();
        assert_png_ihdr(&png, 100, 999, 8, 2);
    }

    #[test]
    fn crc32_known_values() {
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
        assert_eq!(crc32(&[]), 0x00000000);
    }
}
