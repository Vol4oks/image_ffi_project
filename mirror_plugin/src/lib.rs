use serde::Deserialize;
use std::ffi::CStr;
use std::os::raw::{c_char, c_uchar, c_uint};

#[derive(Deserialize, Debug)]
struct MirrorParams {
    #[serde(default = "default_false")]
    horizontal: bool,

    #[serde(default = "default_false")]
    vertical: bool,
}

fn default_false() -> bool {
    false
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn process_image(
    width: c_uint,
    height: c_uint,
    rgba_data: *mut c_uchar,
    params: *const c_char,
) {
    let width = width as usize;
    let height = height as usize;

    if rgba_data.is_null() || params.is_null() {
        return;
    }

    let data_len = width * height * 4;

    let data_slice = unsafe { std::slice::from_raw_parts_mut(rgba_data, data_len) };

    let params_str = unsafe {
        match CStr::from_ptr(params).to_str() {
            Ok(s) => s,
            Err(_) => {
                log::debug!("Invalid UTF-8 in parameters, using defaults");
                "{}"
            }
        }
    };

    let mirror_params: MirrorParams = match serde_json::from_str(params_str) {
        Ok(p) => p,
        Err(e) => {
            log::debug!("Failed to parse parameters: {}, using defaults", e);
            MirrorParams {
                horizontal: false,
                vertical: false,
            }
        }
    };

    log::info!(
        "Mirror plugin called: {}x{}, horizontal={}, vertical={}",
        width,
        height,
        mirror_params.horizontal,
        mirror_params.vertical
    );

    if mirror_params.horizontal {
        mirror_horizontal(data_slice, width, height);
    }

    if mirror_params.vertical {
        mirror_vertical(data_slice, width, height);
    }
}

fn mirror_horizontal(data: &mut [u8], width: usize, height: usize) {
    for y in 0..height {
        let row_start = y * width * 4;
        let row = &mut data[row_start..row_start + width * 4];

        for x in 0..width / 2 {
            let left_pixel = x * 4;
            let right_pixel = (width - 1 - x) * 4;

            for i in 0..4 {
                row.swap(left_pixel + i, right_pixel + i);
            }
        }
    }
}

fn mirror_vertical(data: &mut [u8], width: usize, height: usize) {
    let row_size = width * 4;

    for y in 0..height / 2 {
        let top_row_start = y * row_size;
        let bottom_row_start = (height - 1 - y) * row_size;

        // let top_row = top_row_start..top_row_start + row_size;
        // let bottom_row = bottom_row_start..bottom_row_start + row_size;

        for i in 0..row_size {
            data.swap(top_row_start + i, bottom_row_start + i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    static GREEN: &[u8] = &[0, 255, 0, 255];
    static RED: &[u8] = &[255, 0, 0, 255];
    static BLUE: &[u8] = &[0, 0, 255, 255];
    static WHITE: &[u8] = &[255, 255, 255, 255];
    static DATA: &[u8] = &[
        255, 0, 0, 255, // red
        0, 255, 0, 255, // green
        0, 0, 255, 255, // blue
        255, 255, 255, 255, // white
    ];

    #[test]
    fn test_mirror_horizontal() {
        let mut data: Vec<u8> = DATA.to_vec();

        mirror_horizontal(&mut data, 2, 2);

        assert_eq!(data[0..4], *GREEN);
        assert_eq!(data[4..8], *RED);
        assert_eq!(data[8..12], *WHITE);
        assert_eq!(data[12..16], *BLUE);
    }

    #[test]
    fn test_mirror_vertical() {
        let mut data: Vec<u8> = DATA.to_vec();

        mirror_vertical(&mut data, 2, 2);

        assert_eq!(data[0..4], *BLUE);
        assert_eq!(data[4..8], *WHITE);
        assert_eq!(data[8..12], *RED);
        assert_eq!(data[12..16], *GREEN);
    }
}
