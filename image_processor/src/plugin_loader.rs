use std::ffi::{c_char, c_int, c_uchar, c_uint};

use libloading::{Library, Symbol};

use crate::error::ImageProcessorError;

pub struct PluginInterface<'a> {
    pub process_image: Symbol<
        'a,
        extern "C" fn(
            width: c_uint,
            height: c_uint,
            rgba_data: *mut c_uchar,
            params: *const c_char,
        ) -> c_int,
    >,
}

pub struct Plugin {
    plugin: Library,
}

impl Plugin {
    /// SAFETY: Загружаем динамическую библиотеку
    /// Гарантируем что:
    /// - библиотека существует
    /// - путь к ней корректен
    pub fn new(filename: &std::path::PathBuf) -> Result<Self, ImageProcessorError> {
        let lib = unsafe { Library::new(filename) }?;

        Ok(Plugin { plugin: lib })
    }

    /// SAFETY: Получаем указатель на функцию.
    ///  
    /// Гарантируем что:
    /// - функция существует
    /// - сигнатура корректна
    pub fn inteface(&self) -> Result<PluginInterface<'_>, ImageProcessorError> {
        Ok(PluginInterface {
            process_image: unsafe { self.plugin.get("process_image") }?,
        })
    }

    /// Загрузка плагина из указанной деректории
    ///
    /// SAFETY: Загружаем динамическую библиотеку.
    ///
    /// Гарантируем что:
    /// - библиотека существует
    /// - путь к ней корректен
    pub fn load(
        plugin_dir: &std::path::Path,
        plugin_name: &str,
    ) -> Result<Self, ImageProcessorError> {
        let lib_name = format!("{}{}", plugin_name, get_library_extension());
        let lib_path = plugin_dir.join(&lib_name);
        log::debug!("Loading plugin from {:?}", lib_path);

        if !lib_path.exists() {
            return Err(ImageProcessorError::InvalidPlaginPath(lib_path));
        }

        Self::new(&lib_path)
    }
}

fn get_library_extension() -> &'static str {
    if cfg!(target_os = "windows") {
        ".dll"
    } else {
        ".so"
    }
}
