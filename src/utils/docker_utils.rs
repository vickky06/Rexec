use bollard::Docker;
use std::error::Error;

pub fn get_docker_instance() -> Result<Docker, Box<dyn Error>> {
    let docker = Docker::connect_with_local_defaults();
    match docker {
        Ok(docker_image) => {
            println!("Connected to Docker instance successfully.");
            Ok(docker_image)
        }
        Err(e) => {
            println!("Error while connecting docker instance {:?}", e);
            Err(Box::new(e))
        }
    }
}
