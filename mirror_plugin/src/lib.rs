use serde::Deserialize;
use std::ffi::{CStr, c_char, c_int, c_uchar, c_uint};

use plugin_lib::*;

#[derive(Deserialize, Debug)]
struct MirrorParams {
    horizontal: bool,
    vertical: bool,
}

/// # Safety
/// Эта функция unsafe, так как она использует raw указатели.
///
/// # Возвращаемые коды
/// -  O: успешное выполнение
/// - -1: ошибка парсинга параметров
/// - -2: недопустимые размеры изображения
/// - -3: нулевой указатель
///
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn process_image(
    width: c_uint,
    height: c_uint,
    rgba_data: *mut c_uchar,
    params: *const c_char,
) -> c_int {
    let _ = env_logger::try_init();

    if rgba_data.is_null() || params.is_null() {
        return NULL_POINT_ERROR_CODE;
    }

    let (width_usize, height_useze) = match check_dimensions(width, height) {
        Ok((w, h)) => (w, h),
        Err(code) => return code,
    };

    let data_len = match calculate_data_len(width_usize, height_useze) {
        Ok(len) => len,
        Err(code) => return code,
    };

    let data_slice = unsafe { std::slice::from_raw_parts_mut(rgba_data, data_len) };

    let params_str = unsafe {
        match CStr::from_ptr(params).to_str() {
            Ok(s) => s,
            Err(e) => {
                log::debug!("Invalid UTF-8 in parameters: {}", e);
                return PARSE_ERROR_CODE;
            }
        }
    };

    let mirror_params: MirrorParams = match serde_json::from_str(params_str) {
        Ok(p) => p,
        Err(e) => {
            log::debug!("Failed to parse parameters: {}", e);
            return PARSE_ERROR_CODE;
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
        mirror_horizontal(data_slice, width_usize, height_useze);
    }

    if mirror_params.vertical {
        mirror_vertical(data_slice, width_usize, height_useze);
    }

    OK_CODE
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
    fn test_mirror_horizontal_small() {
        let mut data: Vec<u8> = DATA.to_vec();

        mirror_horizontal(&mut data, 2, 2);

        assert_eq!(data[0..4], *GREEN);
        assert_eq!(data[4..8], *RED);
        assert_eq!(data[8..12], *WHITE);
        assert_eq!(data[12..16], *BLUE);
    }

    #[test]
    fn test_mirror_vertical_small() {
        let mut data: Vec<u8> = DATA.to_vec();

        mirror_vertical(&mut data, 2, 2);

        assert_eq!(data[0..4], *BLUE);
        assert_eq!(data[4..8], *WHITE);
        assert_eq!(data[8..12], *RED);
        assert_eq!(data[12..16], *GREEN);
    }

    #[test]
    fn test_mirror_horizontal_large_dimensions() {
        let width = 1000;
        let height = 1000;
        let mut data = vec![0; width * height * 4];

        for (i, px) in data.iter_mut().enumerate() {
            *px = (i % 256) as u8;
        }

        let original_data = data.clone();

        mirror_horizontal(&mut data, width, height);

        assert_ne!(data, original_data)
    }

    #[test]
    fn test_mirror_vertical_large_dimensions() {
        let width = 1000;
        let height = 1000;
        let mut data = vec![0; width * height * 4];

        for (i, px) in data.iter_mut().enumerate() {
            *px = (i % 256) as u8;
        }

        let original_data = data.clone();

        mirror_vertical(&mut data, width, height);

        assert_ne!(data, original_data)
    }
}
