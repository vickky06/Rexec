use bollard::Docker;
use std::error::Error;

pub fn get_docker_instance() -> Result<Docker, Box<dyn Error>> {
    let docker = Docker::connect_with_local_defaults()?;
    println!("Connected to Docker instance successfully.");
    Ok(docker)
}
