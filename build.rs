// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     tonic_build::compile_protos("proto/executor.proto")?;
//     Ok(())
// }
// use std::fs;
// use std::path::Path;

use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    // let descriptor_path = out_dir.join("executor_descriptor.bin");

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("executor_descriptor.bin"))
        .compile(&["src/proto/executor.proto"], &["src/proto"])?;

    println!("cargo:rerun-if-changed=src/proto/executor.proto");
    println!("cargo:rerun-if-changed=proto");

    Ok(())
}
