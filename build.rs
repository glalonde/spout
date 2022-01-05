use std::{env, error::Error, fs};

// All shaders reside in the 'src/shaders' directory
fn generate_shaders() -> std::result::Result<(), Box<dyn Error>> {
    let tera = tera::Tera::new("src/shaders/*")?;
    println!("cargo:rerun-if-changed=src/shaders/");
    let mut context = tera::Context::new();

    context.insert("inner_grid_bits", &int_grid::INNER_GRID_BITS);
    context.insert("outer_grid_bits", &int_grid::OUTER_GRID_BITS);
    context.insert("outer_grid_size", &int_grid::OUTER_GRID_SIZE);
    context.insert("half_outer_grid_size", &int_grid::HALF_OUTER_GRID_SIZE);
    context.insert("grid_anchor", &int_grid::GRID_ANCHOR);
    context.insert("grid_anchor_absolute", &int_grid::GRID_ANCHOR_ABSOLUTE);
    context.insert("high_res_mask", &int_grid::HIGH_RES_MASK);
    context.insert("inner_grid_size", &int_grid::INNER_GRID_SIZE);
    context.insert("half_inner_grid_size", &int_grid::HALF_INNER_GRID_SIZE);


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
