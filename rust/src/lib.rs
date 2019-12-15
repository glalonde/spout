pub mod shader_utils {
    use lazy_static::lazy_static;
    use log::info;
    // Input path in the source tree, and also the output path in the output directory.
    // This needs to match the path in build.rs
    // TODO(glalonde) Factor this into a library.
    static SHADER_PATH: &str = "shaders";
    lazy_static! {
        pub static ref SHADER_OUTPUT_DIR: std::path::PathBuf =
            std::path::Path::new(env!("OUT_DIR")).join(std::path::Path::new(SHADER_PATH));
    }
    pub fn list_shaders() {
        // Tell the build script to only run again if we change our source shaders.
        // Unfortunately, if a single shader changes, it recompiles everything.
        for entry in walkdir::WalkDir::new(SHADER_OUTPUT_DIR.as_path())
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| !e.file_type().is_dir())
        {
            info!("Found shader: {}", entry.path().display());
        }
    }

    #[macro_export]
    macro_rules! include_shader {
        ( $shader_name:expr ) => {
            include_bytes!(concat!(env!("OUT_DIR"), "/", "shaders", "/", $shader_name))
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        scrub_log::init().unwrap();
        shader_utils::list_shaders();
        let _test_bytes = include_shader!("collatz.comp.spv");
    }
}
