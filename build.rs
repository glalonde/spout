use std::{env, error::Error, fs};

// All shaders reside in the 'src/shaders' directory
fn generate_shaders() -> std::result::Result<(), Box<dyn Error>> {
    let tera = tera::Tera::new("src/shaders/*")?;
    println!("cargo:rerun-if-changed=src/shaders/");
    let mut context = tera::Context::new();

    // Workgroup size for all particle/density compute shaders.
    // Must match particle_workgroup_size in particles.rs.
    const PARTICLE_WORKGROUP_SIZE: u32 = 256;
    context.insert("particle_workgroup_size", &PARTICLE_WORKGROUP_SIZE);

    let output_path = env::var("OUT_DIR")?;
    fs::create_dir_all(format!("{}/shaders/", output_path))?;
    for file in fs::read_dir("src/shaders")? {
        let file = file?;
        // safe: all files in src/shaders/ have extensions; paths are valid UTF-8
        if file.path().extension().unwrap().to_str().unwrap() == "wgsl" {
            let file = file.file_name();
            let file_name = file.to_str().unwrap();
            let result = tera.render(file_name, &context)?;
            fs::write(format!("{}/shaders/{}", output_path, file_name), result)?;
            println!("cargo:rerun-if-changed=src/shaders/{}", file_name);
        }
    }
    Ok(())
}

fn main() {
    if let Err(err) = generate_shaders() {
        // panic here for a nicer error message, otherwise it will
        // be flattened to one line for some reason
        panic!("Unable to generate shaders\n{}", err);
    }
}
