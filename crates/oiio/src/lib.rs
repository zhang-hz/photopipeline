use photopipeline_core::*;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};

#[cfg(oiio_found)]
mod ffi {
    use std::os::raw::{c_char, c_void};

    extern "C" {
        pub fn OIIO_read_image(
            path: *const c_char,
            width: *mut i32,
            height: *mut i32,
            channels: *mut i32,
        ) -> *mut c_void;

        pub fn OIIO_write_image(
            path: *const c_char,
            data: *const u8,
            w: i32,
            h: i32,
            ch: i32,
            bit_depth: i32,
            is_float: i32,
        ) -> i32;

        pub fn OIIO_free_image(ptr: *mut c_void);

        pub fn OIIO_get_image_info(
            path: *const c_char,
            w: *mut i32,
            h: *mut i32,
            ch: *mut i32,
            bit_depth: *mut i32,
        ) -> i32;

        pub fn OIIO_image_metadata(
            path: *const c_char,
            key: *const c_char,
            value: *mut c_char,
            value_len: i32,
        ) -> i32;
    }
}

pub struct OiioContext;

impl OiioContext {
    pub fn available() -> bool {
        cfg!(oiio_found)
    }

    pub fn read_image(path: &str) -> Result<DecodedImage, String> {
        #[cfg(not(oiio_found))]
        {
            let _ = path;
            return Err("OIIO runtime not available".into());
        }
        #[cfg(oiio_found)]
        {
            let c_path =
                CString::new(path).map_err(|_| "invalid path: contains null byte".to_string())?;
            let mut width: i32 = 0;
            let mut height: i32 = 0;
            let mut channels: i32 = 0;

            let ptr = unsafe {
                ffi::OIIO_read_image(c_path.as_ptr(), &mut width, &mut height, &mut channels)
            };
            if ptr.is_null() {
                return Err(format!("OIIO failed to read image: {}", path));
            }

            if width <= 0 || height <= 0 || channels <= 0 {
                unsafe { ffi::OIIO_free_image(ptr) };
                return Err(format!(
                    "OIIO returned invalid dimensions: {}x{}x{}",
                    width, height, channels
                ));
            }

            let data_len = width as usize * height as usize * channels as usize * 4;
            let mut data = vec![0u8; data_len];
            unsafe {
                std::ptr::copy_nonoverlapping(ptr as *const u8, data.as_mut_ptr(), data_len);
                ffi::OIIO_free_image(ptr);
            }

            let layout = match channels {
                1 => ChannelLayout::Gray,
                2 => ChannelLayout::GrayAlpha,
                3 => ChannelLayout::RGB,
                4 => ChannelLayout::RGBA,
                n => ChannelLayout::Custom(n as u8),
            };

            let mut aligned = AlignedBuffer::new(data_len, 64);
            aligned.data.copy_from_slice(&data);

            let buffer = PixelBuffer {
                width: width as u32,
                height: height as u32,
                layout,
                format: PixelFormat::F32,
                color_space: ColorSpace::default(),
                icc_profile: None,
                data: aligned,
            };

            Ok(DecodedImage {
                buffer,
                metadata: Metadata::default(),
                format: ImageFormat::Unknown(path.to_string()),
            })
        }
    }

    pub fn write_image(
        path: &str,
        buffer: &PixelBuffer,
        metadata: &Metadata,
    ) -> Result<(), String> {
        #[cfg(not(oiio_found))]
        {
            let _ = (path, buffer, metadata);
            return Err("OIIO runtime not available".into());
        }
        #[cfg(oiio_found)]
        {
            let c_path =
                CString::new(path).map_err(|_| "invalid path: contains null byte".to_string())?;
            let channels = buffer.layout.channel_count() as i32;
            let bit_depth = (buffer.format.bytes_per_channel() * 8) as i32;
            let is_float = if buffer.format.is_float() { 1 } else { 0 };

            let ret = unsafe {
                ffi::OIIO_write_image(
                    c_path.as_ptr(),
                    buffer.data.data.as_ptr(),
                    buffer.width as i32,
                    buffer.height as i32,
                    channels,
                    bit_depth,
                    is_float,
                )
            };
            if ret != 0 {
                return Err(format!(
                    "OIIO failed to write image: {} (code {})",
                    path, ret
                ));
            }

            if let Some(ref exif) = metadata.exif {
                if let Some(ref desc) = exif.image_description {
                    let _ = Self::set_metadata(path, "ImageDescription", desc);
                }
                if let Some(ref artist) = exif.artist {
                    let _ = Self::set_metadata(path, "Artist", artist);
                }
                if let Some(ref copyright) = exif.copyright {
                    let _ = Self::set_metadata(path, "Copyright", copyright);
                }
            }

            Ok(())
        }
    }

    pub fn get_image_info(path: &str) -> Result<ImageInfo, String> {
        #[cfg(not(oiio_found))]
        {
            let _ = path;
            return Err("OIIO runtime not available".into());
        }
        #[cfg(oiio_found)]
        {
            let c_path =
                CString::new(path).map_err(|_| "invalid path: contains null byte".to_string())?;
            let mut w: i32 = 0;
            let mut h: i32 = 0;
            let mut ch: i32 = 0;
            let mut bd: i32 = 0;

            let ret = unsafe {
                ffi::OIIO_get_image_info(c_path.as_ptr(), &mut w, &mut h, &mut ch, &mut bd)
            };
            if ret != 0 {
                return Err(format!("OIIO failed to get image info for: {}", path));
            }

            Ok(ImageInfo {
                id: uuid::Uuid::new_v4(),
                path: path.to_string(),
                filename: std::path::Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
                format: ImageFormat::Unknown(path.to_string()),
                width: w as u32,
                height: h as u32,
                file_size_bytes: std::fs::metadata(path).map(|m| m.len()).unwrap_or(0),
                pixel_format: PixelFormat::F32,
                color_space: ColorSpace::default(),
            })
        }
    }

    fn set_metadata(path: &str, key: &str, value: &str) -> Result<(), String> {
        #[cfg(not(oiio_found))]
        {
            let _ = (path, key, value);
            Ok(())
        }
        #[cfg(oiio_found)]
        {
            let c_path =
                CString::new(path).map_err(|_| "invalid path: contains null byte".to_string())?;
            let c_key =
                CString::new(key).map_err(|_| "invalid key: contains null byte".to_string())?;
            let c_value =
                CString::new(value).map_err(|_| "invalid value: contains null byte".to_string())?;

            let mut buf = vec![0u8; 4096];
            let ret = unsafe {
                buf[..c_value.as_bytes().len().min(4095)].copy_from_slice(c_value.as_bytes());
                ffi::OIIO_image_metadata(
                    c_path.as_ptr(),
                    c_key.as_ptr(),
                    buf.as_mut_ptr() as *mut c_char,
                    4096,
                )
            };
            if ret != 0 {
                return Err(format!("OIIO failed to set metadata: {}={}", key, value));
            }
            Ok(())
        }
    }
}
