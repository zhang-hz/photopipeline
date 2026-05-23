use std::collections::HashSet;

use photopipeline_core::{PixelBuffer, PixelFormat};

fn read_pixel_f64(buf: &PixelBuffer, pixel: usize, channel: usize) -> f64 {
    let bpc = buf.format.bytes_per_channel();
    let channels = buf.layout.channel_count() as usize;
    let total_pixels = (buf.width as usize) * (buf.height as usize);
    let offset = if buf.layout.is_interleaved() {
        (pixel * channels + channel) * bpc
    } else {
        channel * total_pixels * bpc + pixel * bpc
    };
    pixel_value_f64(&buf.data.data, offset, buf.format)
}

pub fn pixel_value_f64(data: &[u8], offset: usize, format: PixelFormat) -> f64 {
    let bpc = format.bytes_per_channel();
    if offset + bpc > data.len() {
        return 0.0;
    }
    match format {
        PixelFormat::U8 => data[offset] as f64 / 255.0,
        PixelFormat::U16 => {
            let v = u16::from_le_bytes([data[offset], data[offset + 1]]);
            v as f64 / 65535.0
        }
        PixelFormat::F16 => {
            let v = u16::from_le_bytes([data[offset], data[offset + 1]]);
            f16_to_f64(v)
        }
        PixelFormat::U32 => {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[offset..offset + 4]);
            u32::from_le_bytes(bytes) as f64 / u32::MAX as f64
        }
        PixelFormat::F32 => {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[offset..offset + 4]);
            f32::from_le_bytes(bytes) as f64
        }
    }
}

fn f16_to_f64(v: u16) -> f64 {
    let sign = (v >> 15) as f64;
    let exp = ((v >> 10) & 0x1F) as f64;
    let frac = (v & 0x3FF) as f64;
    if exp == 0.0 {
        (sign * -2.0 + 1.0) * 2.0f64.powi(-14) * (frac / 1024.0)
    } else if exp < 31.0 {
        (sign * -2.0 + 1.0) * 2.0f64.powi((exp as i32) - 15) * (1.0 + frac / 1024.0)
    } else {
        if frac == 0.0 { f64::INFINITY } else { f64::NAN }
    }
}

fn max_value_f64(format: PixelFormat) -> f64 {
    match format {
        PixelFormat::U8 => 255.0,
        PixelFormat::U16 => 65535.0,
        PixelFormat::F16 => 65535.0,
        PixelFormat::U32 => u32::MAX as f64,
        PixelFormat::F32 => 1.0,
    }
}

fn read_channel_value(
    data: &[u8],
    pixel: usize,
    channels: usize,
    bpc: usize,
    channel: usize,
) -> f64 {
    let offset = (pixel * channels + channel) * bpc;
    if offset + bpc > data.len() {
        return 0.0;
    }
    match bpc {
        1 => data[offset] as f64 / 255.0,
        2 => {
            let v = u16::from_le_bytes([data[offset], data[offset + 1]]);
            v as f64 / 65535.0
        }
        _ => {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(&data[offset..offset + 4]);
            f32::from_le_bytes(bytes) as f64
        }
    }
}

fn read_channel_u32(
    data: &[u8],
    pixel: usize,
    channels: usize,
    bpc: usize,
    channel: usize,
) -> usize {
    let offset = (pixel * channels + channel) * bpc;
    if offset + bpc > data.len() {
        return 0;
    }
    match bpc {
        1 => data[offset] as usize,
        2 => u16::from_le_bytes([data[offset], data[offset + 1]]) as usize,
        _ => {
            let v = read_channel_value(data, pixel, channels, bpc, channel);
            (v * 255.0).round() as usize
        }
    }
}

pub fn compute_entropy(
    data: &[u8],
    width: u32,
    height: u32,
    channels: usize,
    bytes_per_channel: usize,
    channel: usize,
) -> f64 {
    let pixel_count = (width as usize) * (height as usize);
    if pixel_count == 0 || channel >= channels {
        return 0.0;
    }

    let num_bins: usize = match bytes_per_channel {
        1 => 256,
        2 => 65536,
        _ => 256,
    };
    let mut histogram = vec![0u64; num_bins];

    for p in 0..pixel_count {
        let bin = read_channel_u32(data, p, channels, bytes_per_channel, channel);
        if bin < num_bins {
            histogram[bin] += 1;
        }
    }

    let n = pixel_count as f64;
    let mut entropy = 0.0f64;
    for &count in &histogram {
        if count > 0 {
            let p = count as f64 / n;
            entropy -= p * p.log2();
        }
    }
    entropy
}

pub fn compute_psnr(original: &PixelBuffer, decoded: &PixelBuffer) -> f64 {
    assert_eq!(original.width, decoded.width);
    assert_eq!(original.height, decoded.height);
    assert_eq!(original.format, decoded.format);
    assert_eq!(original.layout, decoded.layout);

    let channels = original.layout.channel_count() as usize;
    let total_pixels = (original.width as usize) * (original.height as usize);
    let max_val = max_value_f64(original.format);

    if total_pixels == 0 || channels == 0 {
        return f64::INFINITY;
    }

    let mut sum_sq_diff = 0.0f64;
    for p in 0..total_pixels {
        for c in 0..channels {
            let orig = read_pixel_f64(original, p, c);
            let dec = read_pixel_f64(decoded, p, c);
            let diff = orig - dec;
            sum_sq_diff += diff * diff;
        }
    }
    let mse = sum_sq_diff / (total_pixels * channels) as f64;

    if mse == 0.0 {
        f64::INFINITY
    } else {
        20.0 * max_val.log10() - 10.0 * mse.log10()
    }
}

pub fn compute_mae(original: &PixelBuffer, decoded: &PixelBuffer) -> f64 {
    let channels = original.layout.channel_count() as usize;
    let total_pixels = (original.width as usize) * (original.height as usize);
    let max_val = max_value_f64(original.format);

    if total_pixels == 0 || channels == 0 {
        return 0.0;
    }

    let mut sum_abs_diff = 0.0f64;
    for p in 0..total_pixels {
        for c in 0..channels {
            let orig = read_pixel_f64(original, p, c);
            let dec = read_pixel_f64(decoded, p, c);
            sum_abs_diff += (orig - dec).abs();
        }
    }
    sum_abs_diff / (total_pixels * channels) as f64 / max_val
}

fn sobel_gradient_magnitude(buf: &PixelBuffer) -> Vec<f64> {
    let w = buf.width as usize;
    let h = buf.height as usize;
    let total = w * h;
    let channels = buf.layout.channel_count() as usize;
    let mut mag = vec![0.0f64; total];

    let gx_kernel: [[f64; 3]; 3] = [[-1.0, 0.0, 1.0], [-2.0, 0.0, 2.0], [-1.0, 0.0, 1.0]];
    let gy_kernel: [[f64; 3]; 3] = [[-1.0, -2.0, -1.0], [0.0, 0.0, 0.0], [1.0, 2.0, 1.0]];

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            let idx = y * w + x;
            let mut gx = 0.0f64;
            let mut gy = 0.0f64;
            for ky in 0..3usize {
                for kx in 0..3usize {
                    let py = y + ky - 1;
                    let px = x + kx - 1;
                    let pixel_idx = py * w + px;
                    let mut luminance = 0.0f64;
                    for c in 0..channels {
                        luminance += read_pixel_f64(buf, pixel_idx, c);
                    }
                    luminance /= channels as f64;
                    gx += gx_kernel[ky][kx] * luminance;
                    gy += gy_kernel[ky][kx] * luminance;
                }
            }
            mag[idx] = (gx * gx + gy * gy).sqrt();
        }
    }
    mag
}

fn nearest_neighbor_scale(
    data: &[f64],
    src_w: usize,
    src_h: usize,
    dst_w: usize,
    dst_h: usize,
) -> Vec<f64> {
    let mut scaled = vec![0.0f64; dst_w * dst_h];
    for y in 0..dst_h {
        for x in 0..dst_w {
            let sx = (x as f64 * src_w as f64 / dst_w as f64) as usize;
            let sy = (y as f64 * src_h as f64 / dst_h as f64) as usize;
            let sx = sx.min(src_w - 1);
            let sy = sy.min(src_h - 1);
            scaled[y * dst_w + x] = data[sy * src_w + sx];
        }
    }
    scaled
}

fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len());
    let n = x.len() as f64;
    if n == 0.0 {
        return 0.0;
    }

    let sum_x: f64 = x.iter().sum();
    let sum_y: f64 = y.iter().sum();
    let mean_x = sum_x / n;
    let mean_y = sum_y / n;

    let mut cov = 0.0f64;
    let mut var_x = 0.0f64;
    let mut var_y = 0.0f64;

    for i in 0..x.len() {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 && var_y == 0.0 {
        return 1.0;
    }
    if var_x == 0.0 || var_y == 0.0 {
        return 0.0;
    }

    cov / (var_x.sqrt() * var_y.sqrt())
}

pub fn compute_structure_similarity(original: &PixelBuffer, decoded: &PixelBuffer) -> f64 {
    let orig_mag = sobel_gradient_magnitude(original);
    let dec_mag = sobel_gradient_magnitude(decoded);

    let ow = original.width as usize;
    let oh = original.height as usize;
    let dw = decoded.width as usize;
    let dh = decoded.height as usize;

    let (a, b) = if ow == dw && oh == dh {
        (orig_mag, dec_mag)
    } else if ow * oh > dw * dh {
        let scaled = nearest_neighbor_scale(&dec_mag, dw, dh, ow, oh);
        (orig_mag, scaled)
    } else {
        let scaled = nearest_neighbor_scale(&orig_mag, ow, oh, dw, dh);
        (scaled, dec_mag)
    };

    let r = pearson_correlation(&a, &b);
    r.max(0.0).min(1.0)
}

pub fn compute_histogram_divergence(a: &PixelBuffer, b: &PixelBuffer, channel: usize) -> f64 {
    let channels_a = a.layout.channel_count() as usize;
    let channels_b = b.layout.channel_count() as usize;

    if channel >= channels_a || channel >= channels_b {
        return 0.0;
    }

    let total_a = (a.width as usize) * (a.height as usize);
    let total_b = (b.width as usize) * (b.height as usize);

    let num_bins: usize = match a.format.bytes_per_channel() {
        1 => 256,
        2 => 65536,
        _ => 256,
    };

    let mut hist_a = vec![0u64; num_bins];
    let mut hist_b = vec![0u64; num_bins];

    for p in 0..total_a {
        let v = read_pixel_f64(a, p, channel);
        let bin = (v * (num_bins - 1) as f64).round() as usize;
        let bin = bin.min(num_bins - 1);
        hist_a[bin] += 1;
    }
    for p in 0..total_b {
        let v = read_pixel_f64(b, p, channel);
        let bin = (v * (num_bins - 1) as f64).round() as usize;
        let bin = bin.min(num_bins - 1);
        hist_b[bin] += 1;
    }

    let smooth_a: Vec<f64> = hist_a.iter().map(|&c| c as f64 + 1.0).collect();
    let smooth_b: Vec<f64> = hist_b.iter().map(|&c| c as f64 + 1.0).collect();
    let sum_a: f64 = smooth_a.iter().sum();
    let sum_b: f64 = smooth_b.iter().sum();

    let mut divergence = 0.0f64;
    for i in 0..num_bins {
        let pa = smooth_a[i] / sum_a;
        let pb = smooth_b[i] / sum_b;
        divergence += pa * (pa / pb).log2();
    }
    divergence
}

pub fn count_unique_values(
    data: &[u8],
    width: u32,
    height: u32,
    bytes_per_channel: usize,
    channels: usize,
    channel: usize,
) -> usize {
    let pixel_count = (width as usize) * (height as usize);
    if pixel_count == 0 || channel >= channels {
        return 0;
    }

    let mut seen = HashSet::new();
    for p in 0..pixel_count {
        let val = read_channel_u32(data, p, channels, bytes_per_channel, channel);
        seen.insert(val);
    }
    seen.len()
}

pub fn assert_compression_ratio(original_size: usize, encoded_size: usize, min_ratio: f64) {
    let ratio = original_size as f64 / encoded_size as f64;
    assert!(ratio >= min_ratio);
}

pub fn assert_quality_psnr(original: &PixelBuffer, decoded: &PixelBuffer, min_psnr_db: f64) {
    let psnr = compute_psnr(original, decoded);
    assert!(psnr >= min_psnr_db);
}

pub fn assert_entropy_preserved(original: &PixelBuffer, decoded: &PixelBuffer, tolerance: f64) {
    let channels = original.layout.channel_count() as usize;
    let bpc = original.format.bytes_per_channel();

    for c in 0..channels {
        let ent_orig = compute_entropy(
            &original.data.data,
            original.width,
            original.height,
            channels,
            bpc,
            c,
        );
        let ent_dec = compute_entropy(
            &decoded.data.data,
            decoded.width,
            decoded.height,
            channels,
            bpc,
            c,
        );
        assert!(
            (ent_orig - ent_dec).abs() < tolerance,
            "entropy mismatch for channel {c}: orig={ent_orig}, dec={ent_dec}, tolerance={tolerance}"
        );
    }
}

pub fn assert_bit_depth_preserved(original: &PixelBuffer, decoded: &PixelBuffer) {
    let channels = original.layout.channel_count() as usize;
    let bpc = original.format.bytes_per_channel();

    for c in 0..channels {
        let count_orig = count_unique_values(
            &original.data.data,
            original.width,
            original.height,
            bpc,
            channels,
            c,
        );
        let count_dec = count_unique_values(
            &decoded.data.data,
            decoded.width,
            decoded.height,
            bpc,
            channels,
            c,
        );
        assert_eq!(
            count_orig, count_dec,
            "unique value count mismatch for channel {c}: orig={count_orig}, dec={count_dec}"
        );
    }
}

pub fn assert_structure_preserved(
    original: &PixelBuffer,
    decoded: &PixelBuffer,
    min_similarity: f64,
) {
    let ss = compute_structure_similarity(original, decoded);
    assert!(
        ss >= min_similarity,
        "structure similarity {ss} below minimum {min_similarity}"
    );
}
