// Include precompiled shader bytes by specifying a path relative to the shader
// source directory.
#[macro_export]
macro_rules! include_shader {
    ($path:literal) => {
        {
            log::info!("Loading shader from {}", concat!(env!("OUT_DIR"), "/shaders/", $path));
            include_str!(concat!(env!("OUT_DIR"), "/shaders/", $path))
        }
    };
}