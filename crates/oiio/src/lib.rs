#[cfg(feature = "oiio")]
extern "C" {
    fn OIIO_read_image(path: *const std::os::raw::c_char) -> *mut std::os::raw::c_void;
    fn OIIO_write_image(
        path: *const std::os::raw::c_char,
        data: *const u8,
        w: i32,
        h: i32,
        ch: i32,
    ) -> i32;
    fn OIIO_free_image(ptr: *mut std::os::raw::c_void);
}
