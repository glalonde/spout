// Include precompiled shader bytes by specifying a path relative to the shader
// source directory.
#[macro_export]
macro_rules! include_shader {
    ($path:literal) => {{
        let path = concat!(env!("OUT_DIR"), "/shaders/", $path);
        log::info!("Loading shader from {}", path);
        std::borrow::Cow::Borrowed(include_str!(concat!(env!("OUT_DIR"), "/shaders/", $path)))
        // if cfg!(debug_assertions) {
        // let maybe_string = std::fs::read_to_string(path);
        // std::borrow::Cow::Owned(maybe_string.unwrap().to_string())
        /*
        } else {
            std::borrow::Cow::Borrowed(include_str!(concat!(env!("OUT_DIR"), "/shaders/", $path)))
        }
        */
    }};
}
