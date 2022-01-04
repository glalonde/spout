use std::{env, error::Error, fs};
// All shaders reside in the 'src/shaders' directory
fn generate_shaders() -> std::result::Result<(), Box<dyn Error>> {
    let tera = tera::Tera::new("src/shaders/*")?;
    println!("cargo:rerun-if-changed=src/shaders/");
    let context = tera::Context::new();
    let output_path = env::var("OUT_DIR")?;
    fs::create_dir_all(format!("{}/shaders/", output_path))?;
    for file in fs::read_dir("src/shaders")? {
        let file = file?;
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
