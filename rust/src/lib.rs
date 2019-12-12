pub mod shader_utils {
    use log::info;
    pub fn print_output_directory() {
        info!("Output directory: {}", env!("OUT_DIR"));
    }
    pub fn list_shaders() {
        info!("Foo");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        scrub_log::init().unwrap();
        shader_utils::print_output_directory();
    }
}
