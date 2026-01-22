use std::ffi::{c_int, c_uint};

pub const OK_CODE: i32 = 0;
pub const PARSE_ERROR_CODE: i32 = -1;
pub const DATA_IMAGE_ERROR_CODE: i32 = -2;
pub const NULL_POINT_ERROR_CODE: i32 = -3;
pub const INVALID_PARAMS_CODE: i32 = -4;

pub const BYTES_PER_PIXEL: usize = 4;

pub fn check_dimensions(width: c_uint, height: c_uint) -> Result<(usize, usize), c_int> {
    if width == 0 || height == 0 {
        log::error!("Invalid dimensions: {}x{}", width, height);
        return Err(DATA_IMAGE_ERROR_CODE);
    }
    let width_usize = width as usize;
    let height_usize = height as usize;

    if width_usize as c_uint != width || height_usize as c_uint != height {
        log::error!(
            "Dimensions overflow: {}x{} cannot be represented as usize",
            width,
            height
        );
        return Err(DATA_IMAGE_ERROR_CODE);
    }

    Ok((width_usize, height_usize))
}

pub fn calculate_data_len(width: usize, height: usize) -> Result<usize, c_int> {
    width
        .checked_mul(height)
        .and_then(|wh| wh.checked_mul(BYTES_PER_PIXEL))
        .ok_or_else(|| {
            log::error!("Image dimensions cause overflow: {}x{}", width, height);
            DATA_IMAGE_ERROR_CODE
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_dimensions_valid() {
        assert!(check_dimensions(100, 100).is_ok());
    }

    #[test]
    fn test_check_dimensions_zero() {
        assert!(check_dimensions(0, 100).is_err());
        assert!(check_dimensions(100, 0).is_err());
    }

    #[test]
    fn test_calculate_data_len_valid() {
        assert_eq!(
            calculate_data_len(100, 100).unwrap(),
            100 * 100 * BYTES_PER_PIXEL
        );
    }

    #[test]
    fn test_calculate_data_len_overflow() {
        assert!(calculate_data_len(usize::MAX / 4 + 1, 100).is_err());
    }
}
