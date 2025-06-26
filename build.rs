use std::env;
use std::path::PathBuf;

/// Build script for generating gRPC code and file descriptor set for the executor service.
///
/// This script uses `tonic_build` to compile the Protocol Buffers definition located at
/// `src/proto/executor.proto`. It outputs the generated Rust code and a binary file descriptor
/// set (`executor_descriptor.bin`) to the build output directory (`OUT_DIR`). This descriptor
/// can be used for reflection or dynamic message handling.
///
/// The script also instructs Cargo to rerun the build script if the proto file or the proto
/// directory changes, ensuring that code generation stays up to date.
///
/// # Errors
/// Returns an error if code generation fails or if environment variables are missing.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("executor_descriptor.bin"))
        .compile(&["src/proto/executor.proto"], &["src/proto"])?;

    println!("cargo:rerun-if-changed=src/proto/executor.proto");
    println!("cargo:rerun-if-changed=proto");

    Ok(())
}
