//! Macro for including tera-templated WGSL shaders from the build output directory.

#[macro_export]
macro_rules! include_shader {
    ($path:literal) => {{
        std::borrow::Cow::Borrowed(include_str!(concat!(env!("OUT_DIR"), "/shaders/", $path)))
    }};
}
