use log::debug;
use serde::Deserialize;
use std::ffi::CStr;
use std::os::raw::{c_char, c_uchar, c_uint};

#[derive(Deserialize, Debug)]
struct BlurParams {
    #[serde(default = "default_radius")]
    radius: f32,

    #[serde(default = "default_iterations")]
    iterations: u32,
}

fn default_radius() -> f32 {
    1.0
}

fn default_iterations() -> u32 {
    1
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

    let params_str = unsafe { CStr::from_ptr(params).to_str().unwrap_or("{}") };

    let blur_params: BlurParams = match serde_json::from_str(params_str) {
        Ok(p) => p,
        Err(_) => BlurParams {
            radius: default_radius(),
            iterations: default_iterations(),
        },
    };

    debug!(
        "Blur plugin called: {}x{}, radius={}, iterations={}",
        width, height, blur_params.radius, blur_params.iterations
    );

    for _ in 0..blur_params.iterations {
        apply_blur(data_slice, width, height, blur_params.radius);
    }
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
