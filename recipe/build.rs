use std::{env, path::PathBuf};

// run build script with `cargo build -vv` to see
// println output.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Building protobuf");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    println!("OUT_DIR: {:?}", out_dir);
    tonic_build::configure()
        // .file_descriptor_set_path(out_dir.join("recipe_descriptor.bin"))
        .compile(&["protos/recipe.proto"], &["protos"])?;
    Ok(())
}
