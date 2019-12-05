use log::{error, info};
use phf::phf_map;
use std::error;

static SHADER_EXTENSION_MAP: phf::Map<&'static str, shaderc::ShaderKind> = phf_map! {
    "comp" => shaderc::ShaderKind::Compute,
    "frag" => shaderc::ShaderKind::Fragment,
    "vert" => shaderc::ShaderKind::Vertex,
};

// Input path in the source tree, and also the output path in the output directory.
static SHADER_PATH: &str = "shaders";

fn main() -> Result<(), Box<dyn error::Error>> {
    scrub_log::init().unwrap();
    // Tell the build script to only run again if we change our source shaders.
    // Unfortunately, if a single shader changes, it recompiles everything.
    for entry in walkdir::WalkDir::new(SHADER_PATH).into_iter() {
        let entry = entry.unwrap();
        println!("cargo:rerun-if-changed={}", entry.path().display());
    }

    let mut compiler = shaderc::Compiler::new().unwrap();
    let mut options = shaderc::CompileOptions::new().unwrap();
    let get_include_source = |include_path: &str,
                              inc_type: shaderc::IncludeType,
                              requestor_path: &str,
                              _depth: usize|
     -> Result<shaderc::ResolvedInclude, String> {
        // If this is a relative include, we set dir_name to the path containing the requestor file.
        let parent_dir = if inc_type == shaderc::IncludeType::Relative {
            std::path::Path::new(requestor_path)
                .parent()
                .unwrap_or(std::path::Path::new("."))
        } else {
            std::path::Path::new(".")
        };
        let resolved_path = parent_dir.join(std::path::Path::new(include_path));
        info!("Including resolved path: {}", resolved_path.display());

        let source = std::fs::read_to_string(resolved_path.clone());
        let source = match source {
            Ok(contents) => contents,
            Err(e) => return Err(e.to_string()),
        };
        Ok(shaderc::ResolvedInclude {
            resolved_name: String::from(resolved_path.to_str().unwrap()),
            content: source,
        })
    };
    options.set_include_callback(get_include_source);

    // Create destination path if necessary
    // let out_dir = std::env::var("OUT_DIR").unwrap();
    let mut compile_shader = |path: &std::path::Path, kind: &shaderc::ShaderKind| {
        info!("Compiling {}, with shader kind: {:?}", path.display(), kind);
        let source = std::fs::read_to_string(path).expect("Something went wrong reading the file");
        let _binary_result = compiler
            .compile_into_spirv(
                &source,
                *kind,
                &path.display().to_string(),
                "main",
                Some(&options),
            )
            .unwrap_or_else(|e: shaderc::Error| {
                error!("Error during shader compilation. Output: \n---\n{}---", e);
                panic!()
            });
    };

    // Recursively iterate through the shader directory.
    walkdir::WalkDir::new(SHADER_PATH)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .filter_map(|e| {
            let extension = e.path().extension()?.to_str()?;
            let kind = SHADER_EXTENSION_MAP.get(extension)?;
            Some((e, kind))
        })
        .filter_map(Some)
        .for_each(|e| compile_shader(e.0.path(), e.1));
    println!("Done");

    Ok(())
}
