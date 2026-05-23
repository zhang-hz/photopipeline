pub fn crc32(data: &[u8]) -> u32 {
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

// ── PNG Chunk Iterator ──────────────────────────────────────────────

pub struct PngChunkIter<'a> {
    data: &'a [u8],
    pos: usize,
    done: bool,
}

pub struct PngChunkRef<'a> {
    pub offset: usize,
    pub length: u32,
    pub chunk_type: [u8; 4],
    pub crc_offset: usize,
    pub _data: &'a [u8],
}

pub fn png_chunks(data: &[u8]) -> PngChunkIter<'_> {
    PngChunkIter {
        data,
        pos: 8,
        done: false,
    }
}

impl<'a> Iterator for PngChunkIter<'a> {
    type Item = PngChunkRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done || self.pos + 12 > self.data.len() {
            return None;
        }

        let length = u32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        let chunk_type: [u8; 4] = [
            self.data[self.pos + 4],
            self.data[self.pos + 5],
            self.data[self.pos + 6],
            self.data[self.pos + 7],
        ];
        let crc_offset = self.pos + 8 + length as usize;
        let chunk_end = crc_offset + 4;

        if chunk_end > self.data.len() {
            self.done = true;
            return None;
        }

        let chunk_ref = PngChunkRef {
            offset: self.pos,
            length,
            chunk_type,
            crc_offset,
            _data: &self.data[self.pos..chunk_end],
        };

        if &chunk_type == b"IEND" {
            self.done = true;
        } else {
            self.pos = chunk_end;
        }

        Some(chunk_ref)
    }
}

// ── PNG Assertions ───────────────────────────────────────────────────

pub fn assert_png_signature(data: &[u8]) {
    assert!(
        data.len() >= 8,
        "data too short for PNG signature: {} bytes",
        data.len()
    );
    let signature: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];
    assert_eq!(
        &data[0..8],
        &signature,
        "invalid PNG signature: {:02X?}",
        &data[0..8]
    );
}

pub fn assert_png_ihdr_verified(
    data: &[u8],
    expected_w: u32,
    expected_h: u32,
    expected_depth: u8,
    expected_color: u8,
) {
    assert!(
        data.len() >= 26,
        "data too short for IHDR verification: {} bytes",
        data.len()
    );

    let length = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);
    assert_eq!(length, 13, "IHDR length must be 13, got {}", length);

    let chunk_type: [u8; 4] = [data[12], data[13], data[14], data[15]];
    assert_eq!(
        &chunk_type,
        b"IHDR",
        "expected IHDR chunk, got {:?}",
        std::str::from_utf8(&chunk_type).unwrap_or("???")
    );

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
        bit_depth, expected_depth,
        "IHDR bit_depth mismatch: expected {}, got {}",
        expected_depth, bit_depth
    );

    let color_type = data[25];
    assert_eq!(
        color_type, expected_color,
        "IHDR color_type mismatch: expected {}, got {}",
        expected_color, color_type
    );

    let expected_crc = u32::from_be_bytes([data[29], data[30], data[31], data[32]]);
    let computed_crc = crc32(&data[12..29]);
    assert_eq!(
        computed_crc, expected_crc,
        "IHDR CRC mismatch: computed {:08X}, stored {:08X}",
        computed_crc, expected_crc
    );
}

pub fn assert_png_chunk_data(data: &[u8], chunk_type: &[u8; 4], expected_bytes: &[u8]) {
    for chunk in png_chunks(data) {
        if &chunk.chunk_type == chunk_type {
            let payload_start = chunk.offset + 8;
            let payload_end = payload_start + chunk.length as usize;
            let payload = &data[payload_start..payload_end];
            assert_eq!(
                payload,
                expected_bytes,
                "chunk {:?} payload mismatch",
                std::str::from_utf8(chunk_type).unwrap_or("???")
            );
            return;
        }
    }
    panic!(
        "chunk {:?} not found",
        std::str::from_utf8(chunk_type).unwrap_or("???")
    );
}

pub fn assert_png_compression_ratio(
    original_pixels: usize,
    encoded_size: usize,
    min_ratio: f64,
    max_ratio: f64,
) {
    assert!(encoded_size > 0, "encoded_size must be > 0");
    let ratio = original_pixels as f64 / encoded_size as f64;
    assert!(
        ratio >= min_ratio,
        "compression ratio {:.4} below minimum {:.4}",
        ratio,
        min_ratio
    );
    assert!(
        ratio <= max_ratio,
        "compression ratio {:.4} above maximum {:.4}",
        ratio,
        max_ratio
    );
}

// ── ISOBMFF Box Parser ───────────────────────────────────────────────

pub struct IsobmffBoxIter<'a> {
    data: &'a [u8],
    pos: usize,
}

pub fn isobmff_boxes(data: &[u8]) -> IsobmffBoxIter<'_> {
    IsobmffBoxIter { data, pos: 0 }
}

impl<'a> Iterator for IsobmffBoxIter<'a> {
    type Item = (usize, u64, [u8; 4], usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + 8 > self.data.len() {
            return None;
        }

        let size_raw = u32::from_be_bytes([
            self.data[self.pos],
            self.data[self.pos + 1],
            self.data[self.pos + 2],
            self.data[self.pos + 3],
        ]);
        let box_type: [u8; 4] = [
            self.data[self.pos + 4],
            self.data[self.pos + 5],
            self.data[self.pos + 6],
            self.data[self.pos + 7],
        ];

        let (size, payload_offset, payload_len) = match size_raw {
            0 => {
                let remain = (self.data.len() - self.pos) as u64;
                (remain, self.pos + 8, self.data.len() - self.pos - 8)
            }
            1 => {
                if self.pos + 16 > self.data.len() {
                    return None;
                }
                let extended_size = u64::from_be_bytes([
                    self.data[self.pos + 8],
                    self.data[self.pos + 9],
                    self.data[self.pos + 10],
                    self.data[self.pos + 11],
                    self.data[self.pos + 12],
                    self.data[self.pos + 13],
                    self.data[self.pos + 14],
                    self.data[self.pos + 15],
                ]);
                let plen = extended_size.saturating_sub(16);
                (extended_size, self.pos + 16, plen as usize)
            }
            s if s < 8 => {
                return None;
            }
            s => {
                let s = s as u64;
                let plen = s.saturating_sub(8);
                (s, self.pos + 8, plen as usize)
            }
        };

        let offset = self.pos;

        if size_raw == 0 || offset + size as usize > self.data.len() {
            self.pos = self.data.len();
        } else {
            self.pos = offset + size as usize;
        }

        Some((offset, size, box_type, payload_offset, payload_len))
    }
}

// ── HEIF Assertions ──────────────────────────────────────────────────

pub fn assert_heif_ftyp_brand(data: &[u8], expected_brand: &[u8; 4]) {
    for (_offset, _size, box_type, payload_offset, payload_len) in isobmff_boxes(data) {
        if &box_type == b"ftyp" {
            assert!(
                payload_len >= 4,
                "ftyp box payload too short for major brand: {} bytes",
                payload_len
            );
            let major_brand: [u8; 4] = [
                data[payload_offset],
                data[payload_offset + 1],
                data[payload_offset + 2],
                data[payload_offset + 3],
            ];
            assert_eq!(
                &major_brand,
                expected_brand,
                "ftyp major brand mismatch: expected {:?}, got {:?}",
                std::str::from_utf8(expected_brand).unwrap_or("???"),
                std::str::from_utf8(&major_brand).unwrap_or("???")
            );
            return;
        }
    }
    panic!("ftyp box not found");
}

pub fn assert_heif_mdat_non_empty(data: &[u8]) {
    for (_offset, _size, box_type, _payload_offset, payload_len) in isobmff_boxes(data) {
        if &box_type == b"mdat" {
            assert!(payload_len > 0, "mdat box is empty");
            return;
        }
    }
    panic!("mdat box not found");
}

pub fn assert_heif_bitstream_size_range(
    data: &[u8],
    _quality: f32,
    _dimensions: (u32, u32),
    min_bytes: usize,
    max_bytes: usize,
) {
    let len = data.len();
    assert!(
        len >= min_bytes,
        "bitstream size {} below minimum {} bytes",
        len,
        min_bytes
    );
    assert!(
        len <= max_bytes,
        "bitstream size {} above maximum {} bytes",
        len,
        max_bytes
    );
}

// ── TIFF Assertions ──────────────────────────────────────────────────

pub fn assert_tiff_byte_order(data: &[u8], little_endian: bool) {
    assert!(
        data.len() >= 8,
        "TIFF data too short for header: {} bytes",
        data.len()
    );

    let expected_marker = if little_endian { b"II" } else { b"MM" };
    let actual_marker = &data[0..2];
    assert_eq!(
        actual_marker,
        expected_marker,
        "TIFF byte order mismatch: expected {:?}, got {:?}",
        std::str::from_utf8(expected_marker).unwrap_or("???"),
        std::str::from_utf8(actual_marker).unwrap_or("???")
    );

    let magic = if little_endian {
        u16::from_le_bytes([data[2], data[3]])
    } else {
        u16::from_be_bytes([data[2], data[3]])
    };
    assert_eq!(magic, 42, "TIFF magic number must be 42, got {}", magic);
}

pub fn assert_tiff_ifd_entry_count(data: &[u8], expected_count: u16) {
    assert!(
        data.len() >= 8,
        "TIFF data too short for IFD lookup: {} bytes",
        data.len()
    );

    let little_endian = match &data[0..2] {
        b"II" => true,
        b"MM" => false,
        other => panic!("invalid TIFF byte order marker: {:02X?}", other),
    };

    let ifd_offset = if little_endian {
        u32::from_le_bytes([data[4], data[5], data[6], data[7]])
    } else {
        u32::from_be_bytes([data[4], data[5], data[6], data[7]])
    } as usize;

    assert!(
        ifd_offset + 2 <= data.len(),
        "IFD offset {} out of bounds (data len: {})",
        ifd_offset,
        data.len()
    );

    let count = if little_endian {
        u16::from_le_bytes([data[ifd_offset], data[ifd_offset + 1]])
    } else {
        u16::from_be_bytes([data[ifd_offset], data[ifd_offset + 1]])
    };

    assert_eq!(
        count, expected_count,
        "IFD entry count mismatch: expected {}, got {}",
        expected_count, count
    );
}

pub fn assert_tiff_strip_offsets_valid(data: &[u8], _image_dimensions: (u32, u32)) {
    assert!(data.len() >= 8, "TIFF data too short: {} bytes", data.len());

    let little_endian = match &data[0..2] {
        b"II" => true,
        b"MM" => false,
        other => panic!("invalid TIFF byte order marker: {:02X?}", other),
    };

    let read_u16 = |pos: usize| -> u16 {
        if little_endian {
            u16::from_le_bytes([data[pos], data[pos + 1]])
        } else {
            u16::from_be_bytes([data[pos], data[pos + 1]])
        }
    };
    let read_u32 = |pos: usize| -> u32 {
        if little_endian {
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
        } else {
            u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
        }
    };

    let ifd_offset = read_u32(4) as usize;
    assert!(
        ifd_offset + 2 <= data.len(),
        "IFD offset {} out of bounds",
        ifd_offset
    );

    let ifd_count = read_u16(ifd_offset) as usize;
    let tag_start = ifd_offset + 2;
    let tag_end = tag_start + ifd_count * 12;
    assert!(tag_end <= data.len(), "IFD entries out of bounds");

    let mut strip_offsets: Option<(usize, u32)> = None;
    let mut strip_byte_counts: Option<(usize, u32)> = None;

    for i in 0..ifd_count {
        let pos = tag_start + i * 12;
        let tag_id = read_u16(pos);
        let _dtype = read_u16(pos + 2);
        let count = read_u32(pos + 4);

        match tag_id {
            273 => strip_offsets = Some((pos + 8, count)),
            279 => strip_byte_counts = Some((pos + 8, count)),
            _ => {}
        }
    }

    let (offsets_pos, offsets_count) = strip_offsets.expect("StripOffsets tag (273) not found");
    let (counts_pos, counts_count) =
        strip_byte_counts.expect("StripByteCounts tag (279) not found");

    assert_eq!(
        offsets_count, counts_count,
        "StripOffsets count ({}) != StripByteCounts count ({})",
        offsets_count, counts_count
    );

    let offsets_inline = offsets_count <= 2;
    let counts_inline = counts_count <= 2;

    for strip in 0..offsets_count as usize {
        let offset_val = if offsets_inline {
            let val_start = offsets_pos + strip * 4;
            assert!(
                val_start + 4 <= data.len(),
                "StripOffsets value for strip {} out of bounds",
                strip
            );
            read_u32(val_start) as usize
        } else {
            let ref_offset = read_u32(offsets_pos) as usize;
            let val_addr = ref_offset + strip * 4;
            assert!(
                val_addr + 4 <= data.len(),
                "StripOffsets reference for strip {} out of bounds",
                strip
            );
            read_u32(val_addr) as usize
        };

        let byte_count_val = if counts_inline {
            let val_start = counts_pos + strip * 4;
            assert!(
                val_start + 4 <= data.len(),
                "StripByteCounts value for strip {} out of bounds",
                strip
            );
            read_u32(val_start) as usize
        } else {
            let ref_offset = read_u32(counts_pos) as usize;
            let val_addr = ref_offset + strip * 4;
            assert!(
                val_addr + 4 <= data.len(),
                "StripByteCounts reference for strip {} out of bounds",
                strip
            );
            read_u32(val_addr) as usize
        };

        assert!(
            offset_val < data.len(),
            "StripOffsets[{}] = {} is beyond file size {}",
            strip,
            offset_val,
            data.len()
        );
        assert!(
            offset_val + byte_count_val <= data.len(),
            "StripOffsets[{}] + StripByteCounts[{}] = {} exceeds file size {}",
            strip,
            strip,
            offset_val + byte_count_val,
            data.len()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── CRC32 ────────────────────────────────────────────────

    #[test]
    fn crc32_known_values() {
        assert_eq!(crc32(b"123456789"), 0xCBF43926);
        assert_eq!(crc32(&[]), 0x00000000);
    }

    // ── PNG helpers ──────────────────────────────────────────

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

    // ── PngChunkIter ─────────────────────────────────────────

    #[test]
    fn png_chunks_iterates_ihdr_and_iend() {
        let png = make_minimal_png();
        let chunks: Vec<_> = png_chunks(&png).collect();
        assert_eq!(chunks.len(), 2);
        assert_eq!(&chunks[0].chunk_type, b"IHDR");
        assert_eq!(chunks[0].length, 13);
        assert_eq!(&chunks[1].chunk_type, b"IEND");
        assert_eq!(chunks[1].length, 0);
    }

    // ── assert_png_signature ─────────────────────────────────

    #[test]
    fn assert_png_signature_valid() {
        let png = make_minimal_png();
        assert_png_signature(&png);
    }

    #[test]
    #[should_panic(expected = "invalid PNG signature")]
    fn assert_png_signature_bad() {
        let mut png = make_minimal_png();
        png[0] = 0;
        assert_png_signature(&png);
    }

    #[test]
    #[should_panic(expected = "too short")]
    fn assert_png_signature_too_short() {
        assert_png_signature(&[0u8; 4]);
    }

    // ── assert_png_ihdr_verified ─────────────────────────────

    #[test]
    fn assert_png_ihdr_verified_matches() {
        let png = make_minimal_png();
        assert_png_ihdr_verified(&png, 100, 200, 8, 2);
    }

    #[test]
    #[should_panic(expected = "width mismatch")]
    fn assert_png_ihdr_verified_wrong_width() {
        let png = make_minimal_png();
        assert_png_ihdr_verified(&png, 999, 200, 8, 2);
    }

    // ── assert_png_chunk_data ────────────────────────────────

    #[test]
    fn assert_png_chunk_data_matches() {
        let png = make_minimal_png();
        let expected_ihdr = {
            let mut d = Vec::new();
            d.extend_from_slice(&100u32.to_be_bytes());
            d.extend_from_slice(&200u32.to_be_bytes());
            d.push(8);
            d.push(2);
            d.push(0);
            d.push(0);
            d.push(0);
            d
        };
        assert_png_chunk_data(&png, b"IHDR", &expected_ihdr);
    }

    #[test]
    #[should_panic]
    fn assert_png_chunk_data_not_found() {
        let png = make_minimal_png();
        assert_png_chunk_data(&png, b"tEXt", &[]);
    }

    // ── assert_png_compression_ratio ─────────────────────────

    #[test]
    fn assert_png_compression_ratio_in_range() {
        assert_png_compression_ratio(1000, 200, 1.0, 10.0);
    }

    #[test]
    #[should_panic(expected = "below minimum")]
    fn assert_png_compression_ratio_below() {
        assert_png_compression_ratio(10, 200, 1.0, 10.0);
    }

    #[test]
    #[should_panic(expected = "above maximum")]
    fn assert_png_compression_ratio_above() {
        assert_png_compression_ratio(10000, 200, 1.0, 10.0);
    }

    #[test]
    #[should_panic(expected = "encoded_size must be > 0")]
    fn assert_png_compression_ratio_zero_encoded() {
        assert_png_compression_ratio(100, 0, 1.0, 10.0);
    }

    // ── ISOBMFF Box Parser ───────────────────────────────────

    fn make_heif_like() -> Vec<u8> {
        let mut data = Vec::new();
        // ftyp box
        let ftyp_payload: Vec<u8> = {
            let mut p = Vec::new();
            p.extend_from_slice(b"heic");
            p.extend_from_slice(&0u32.to_be_bytes());
            p.extend_from_slice(b"mif1");
            p.extend_from_slice(b"heic");
            p
        };
        let ftyp_size = (8 + ftyp_payload.len()) as u32;
        data.extend_from_slice(&ftyp_size.to_be_bytes());
        data.extend_from_slice(b"ftyp");
        data.extend_from_slice(&ftyp_payload);

        // mdat box
        let mdat_payload = vec![0u8; 256];
        let mdat_size = (8 + mdat_payload.len()) as u32;
        data.extend_from_slice(&mdat_size.to_be_bytes());
        data.extend_from_slice(b"mdat");
        data.extend_from_slice(&mdat_payload);

        data
    }

    #[test]
    fn isobmff_boxes_finds_ftyp_and_mdat() {
        let data = make_heif_like();
        let boxes: Vec<_> = isobmff_boxes(&data).collect();
        assert_eq!(boxes.len(), 2);
        assert_eq!(&boxes[0].2, b"ftyp");
        assert_eq!(&boxes[1].2, b"mdat");
    }

    #[test]
    fn isobmff_boxes_empty_data() {
        let boxes: Vec<_> = isobmff_boxes(&[]).collect();
        assert_eq!(boxes.len(), 0);
    }

    // ── assert_heif_ftyp_brand ───────────────────────────────

    #[test]
    fn assert_heif_ftyp_brand_heic() {
        let data = make_heif_like();
        assert_heif_ftyp_brand(&data, b"heic");
    }

    #[test]
    #[should_panic(expected = "major brand mismatch")]
    fn assert_heif_ftyp_brand_wrong_brand() {
        let data = make_heif_like();
        assert_heif_ftyp_brand(&data, b"avif");
    }

    // ── assert_heif_mdat_non_empty ───────────────────────────

    #[test]
    fn assert_heif_mdat_non_empty_valid() {
        let data = make_heif_like();
        assert_heif_mdat_non_empty(&data);
    }

    #[test]
    #[should_panic(expected = "mdat box not found")]
    fn assert_heif_mdat_non_empty_no_mdat() {
        let data = vec![0u8; 8]; // ftyp only
        assert_heif_mdat_non_empty(&data);
    }

    // ── assert_heif_bitstream_size_range ─────────────────────

    #[test]
    fn assert_heif_bitstream_size_in_range() {
        let data = vec![0u8; 500];
        assert_heif_bitstream_size_range(&data, 80.0, (64, 64), 100, 1000);
    }

    #[test]
    #[should_panic(expected = "below minimum")]
    fn assert_heif_bitstream_size_below() {
        let data = vec![0u8; 50];
        assert_heif_bitstream_size_range(&data, 80.0, (64, 64), 100, 1000);
    }

    #[test]
    #[should_panic(expected = "above maximum")]
    fn assert_heif_bitstream_size_above() {
        let data = vec![0u8; 2000];
        assert_heif_bitstream_size_range(&data, 80.0, (64, 64), 100, 1000);
    }

    // ── TIFF helpers ─────────────────────────────────────────

    fn make_minimal_tiff(little_endian: bool) -> Vec<u8> {
        let mut data = Vec::new();
        if little_endian {
            data.extend_from_slice(b"II");
            data.extend_from_slice(&42u16.to_le_bytes());
            data.extend_from_slice(&8u32.to_le_bytes());
        } else {
            data.extend_from_slice(b"MM");
            data.extend_from_slice(&42u16.to_be_bytes());
            data.extend_from_slice(&8u32.to_be_bytes());
        }

        let num_tags: u16 = 2;
        let ifd_start = data.len();
        if little_endian {
            data.extend_from_slice(&num_tags.to_le_bytes());
        } else {
            data.extend_from_slice(&num_tags.to_be_bytes());
        }

        // StripOffsets tag (273), type LONG (4), count 1
        let write_tag = |d: &mut Vec<u8>, tag_id: u16, dtype: u16, count: u32, value: u32| {
            if little_endian {
                d.extend_from_slice(&tag_id.to_le_bytes());
                d.extend_from_slice(&dtype.to_le_bytes());
                d.extend_from_slice(&count.to_le_bytes());
                d.extend_from_slice(&value.to_le_bytes());
            } else {
                d.extend_from_slice(&tag_id.to_be_bytes());
                d.extend_from_slice(&dtype.to_be_bytes());
                d.extend_from_slice(&count.to_be_bytes());
                d.extend_from_slice(&value.to_be_bytes());
            }
        };

        let data_offset = ifd_start + 2 + 2 * 12;
        write_tag(&mut data, 273, 4, 1, data_offset as u32);
        write_tag(&mut data, 279, 4, 1, 100);

        // Extra space for strip data
        let pad = data_offset + 100 - data.len();
        data.resize(data_offset + 100, 0);

        data
    }

    // ── assert_tiff_byte_order ───────────────────────────────

    #[test]
    fn assert_tiff_byte_order_little_endian() {
        let tiff = make_minimal_tiff(true);
        assert_tiff_byte_order(&tiff, true);
    }

    #[test]
    fn assert_tiff_byte_order_big_endian() {
        let tiff = make_minimal_tiff(false);
        assert_tiff_byte_order(&tiff, false);
    }

    #[test]
    #[should_panic(expected = "byte order mismatch")]
    fn assert_tiff_byte_order_wrong() {
        let tiff = make_minimal_tiff(true);
        assert_tiff_byte_order(&tiff, false);
    }

    // ── assert_tiff_ifd_entry_count ──────────────────────────

    #[test]
    fn assert_tiff_ifd_entry_count_matches() {
        let tiff = make_minimal_tiff(true);
        assert_tiff_ifd_entry_count(&tiff, 2);
    }

    #[test]
    #[should_panic(expected = "IFD entry count mismatch")]
    fn assert_tiff_ifd_entry_count_wrong() {
        let tiff = make_minimal_tiff(true);
        assert_tiff_ifd_entry_count(&tiff, 99);
    }

    // ── assert_tiff_strip_offsets_valid ──────────────────────

    #[test]
    fn assert_tiff_strip_offsets_valid_ok() {
        let tiff = make_minimal_tiff(true);
        assert_tiff_strip_offsets_valid(&tiff, (100, 100));
    }

    #[test]
    #[should_panic(expected = "StripOffsets tag")]
    fn assert_tiff_strip_offsets_valid_no_strip_offsets() {
        let mut data = Vec::new();
        data.extend_from_slice(b"II");
        data.extend_from_slice(&42u16.to_le_bytes());
        data.extend_from_slice(&8u32.to_le_bytes());
        data.extend_from_slice(&0u16.to_le_bytes()); // 0 tags
        assert_tiff_strip_offsets_valid(&data, (100, 100));
    }
}
