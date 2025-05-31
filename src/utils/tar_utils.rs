use std::error::Error;
use std::fs::File;
use tar::Builder;

pub fn create_tar_archive(
    dockerfile_path: &str,
    tar_path: &str,
    docker_file_name: &String,
) -> Result<String, Box<dyn Error>> {
    println!(
        "Creating tar archive for Dockerfile: {}::\n{}",
        dockerfile_path, tar_path
    );
    let tar_file = File::create(tar_path)?;
    let mut tar_builder = Builder::new(tar_file);

    // The name for the Dockerfile *inside* the tar archive.
    // Docker usually expects "Dockerfile" at the root of the build context.
    // let name_in_tar = docker_file_name;
    tar_builder.append_path_with_name(dockerfile_path, docker_file_name)?;
    tar_builder.finish()?;
    println!(
        "Tar archive created at {} containing {} from {}",
        tar_path, docker_file_name, dockerfile_path
    );

    Ok(docker_file_name.to_string()) // Return the name that was actually used in the tar
}
