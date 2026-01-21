use std::ffi::{c_char, c_uchar, c_uint};

use libloading::{Library, Symbol};

use crate::error::{self, ImageProcessorError};

pub struct PluginInterface<'a> {
    pub process_image: Symbol<
        'a,
        extern "C" fn(
            width: c_uint,
            height: c_uint,
            rgba_data: *mut c_uchar,
            params: *const c_char,
        ),
    >,
}

pub struct Plugin {
    plugin: Library,
}

impl Plugin {
    pub fn new(filename: &std::path::PathBuf) -> Result<Self, ImageProcessorError> {
        Ok(Plugin {
            plugin: unsafe { Library::new(filename) }?,
        })
    }

    pub fn inteface(&self) -> Result<PluginInterface<'_>, ImageProcessorError> {
        Ok(PluginInterface {
            process_image: unsafe { self.plugin.get("process_image") }?,
        })
    }

    /// Загрузка плагина из указанной деректории
    pub fn load(
        plugin_dir: &std::path::Path,
        plugin_name: &str,
    ) -> Result<Self, ImageProcessorError> {
        let lib_name = format!("{}{}", plugin_name, get_library_extension());
        let lib_path = plugin_dir.join(&lib_name);
        log::debug!("Loading plugin from {:?}", lib_path);

        if !lib_path.exists() {
            return Err(error::ImageProcessorError::InvalidPlaginPath(lib_path));
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
