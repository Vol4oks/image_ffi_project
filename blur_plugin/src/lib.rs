use serde::Deserialize;
use std::ffi::{CStr, c_char, c_int, c_uchar, c_uint};

use plugin_lib::*;

#[derive(Deserialize, Debug)]
struct BlurParams {
    radius: f32,
    iterations: u32,
}

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

    let blur_params: BlurParams = match serde_json::from_str(params_str) {
        Ok(p) => p,
        Err(e) => {
            log::debug!("Failed to parse parameters: {}", e);
            return PARSE_ERROR_CODE;
        }
    };

    if blur_params.radius <= 0.0 || blur_params.radius > 1000.0 {
        log::error!("Invalid radius value: {}", blur_params.radius);
        return INVALID_PARAMS_CODE;
    }

    if blur_params.iterations == 0 || blur_params.iterations > 1000 {
        log::error!("Invalid iterations value: {}", blur_params.radius);
        return INVALID_PARAMS_CODE;
    }

    log::debug!(
        "Blur plugin called: {}x{}, radius={}, iterations={}",
        width,
        height,
        blur_params.radius,
        blur_params.iterations
    );

    for _ in 0..blur_params.iterations {
        apply_blur(data_slice, width_usize, height_useze, blur_params.radius);
    }

    OK_CODE
}

fn apply_blur(data: &mut [u8], width: usize, height: usize, radius: f32) {
    let radius_i32 = radius as i32;

    let mut temp = vec![0u8; data.len()];

    for y in 0..height {
        for x in 0..width {
            let mut r = 0f32;
            let mut g = 0f32;
            let mut b = 0f32;
            let mut a = 0f32;
            let mut weight_sum = 0f32;

            for ky in -radius_i32..=radius_i32 {
                for kx in -radius_i32..=radius_i32 {
                    let nx = x as i32 + kx;
                    let ny = y as i32 + ky;

                    if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                        let idx = (ny as usize * width + nx as usize) * 4;

                        let distance = (kx * kx + ky * ky) as f32;
                        let weight = 1.0 / (1.0 + distance);

                        r += data[idx] as f32 * weight;
                        g += data[idx + 1] as f32 * weight;
                        b += data[idx + 2] as f32 * weight;
                        a += data[idx + 3] as f32 * weight;
                        weight_sum += weight;
                    }
                }
            }

            let idx = (y * width + x) * 4;
            if weight_sum > 0.0 {
                temp[idx] = (r / weight_sum) as u8;
                temp[idx + 1] = (g / weight_sum) as u8;
                temp[idx + 2] = (b / weight_sum) as u8;
                temp[idx + 3] = (a / weight_sum) as u8;
            } else {
                temp[idx..idx + 4].copy_from_slice(&data[idx..idx + 4]);
            }
        }
    }

    data.copy_from_slice(&temp);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bluer_small_image() {
        let mut data = vec![
            255, 0, 0, 255, // красный
            0, 255, 0, 255, // зеленый
            0, 0, 255, 255, // синий
            255, 255, 0, 255, // желтый
            255, 0, 255, 255, // пурпурный
            0, 255, 255, 255, // голубой
            128, 128, 128, 255, // серый
            64, 64, 64, 255, // темно-серый
            192, 192, 192, 255, // светло-серый
        ];

        let original_data = data.clone();

        apply_blur(&mut data, 3, 3, 1.0);

        assert_ne!(data, original_data);
    }

    #[test]
    fn test_blur_zero_radius() {
        let mut data = vec![255; 16];
        let original_data = data.clone();

        apply_blur(&mut data, 2, 2, 0.0);
        assert_eq!(data, original_data);
    }

    #[test]
    fn test_blur_large_dimensions() {
        let width = 100;
        let height = 100;
        let mut data = vec![228; width * height * 4];

        let original_data = data.clone();

        apply_blur(&mut data, width, height, 2.0);

        assert_ne!(data, original_data);
    }

    #[test]
    fn test_blur_edge_cases() {
        let mut data = vec![255, 0, 0, 255];
        apply_blur(&mut data, 1, 1, 1.0);

        let mut empty_data: Vec<u8> = vec![];
        apply_blur(&mut empty_data, 0, 0, 1.0);
    }
}
