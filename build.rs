use std::borrow::Borrow;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_env_var = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_env_var);
    let shader_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets/shader");

    let files = fs::read_dir(&shader_dir).unwrap();

    let shader_files = files.filter_map(Result::ok).filter(|f| {
        f.path()
            .extension()
            .and_then(|e| Some(e == "glsl"))
            .unwrap_or(true)
    });

    for file in shader_files {
        println!("cargo:rerun-if-changed={}", file.path().to_string_lossy());

        Command::new("glslangValidator")
            .args(&[
                "-V",
                "-o",
                &out_dir
                    .join(Path::new(file.path().file_name().unwrap()).with_extension("spv"))
                    .to_string_lossy(),
                &file.path().to_string_lossy(),
            ])
            .status()
            .unwrap();
    }
}
