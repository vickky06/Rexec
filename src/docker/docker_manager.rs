use bollard::Docker;
use bollard::container::{Config as ContainerConfig, CreateContainerOptions, StartContainerOptions};
use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::BuildImageOptions;
use bollard::models::{HostConfig, PortBinding};
use futures_util::stream::StreamExt;
use std::error::Error;
use std::fs::File;
use tar::Builder;
use tokio::io::AsyncReadExt;
use crate::config::Config;

pub async fn handle_request(
    config: &Config,
    language: &str, 
    code: &str
) -> Result<String, Box<dyn Error>> {
    let docker = Docker::connect_with_local_defaults()?;

    // Select the appropriate Dockerfile
    let dockerfile_path = match language {
        "python" => &config.dockerfiles.python,
        "javascript" => &config.dockerfiles.javascript,
        "java" => &config.dockerfiles.java,
        _ => return Err(format!("Unsupported language: {}", language).into()),
    };

    // Build and run the container
    let container_name = build_and_run_container(config,&docker, dockerfile_path,language).await?;

    // Execute the code inside the container
    let result = execute_code_in_container(&docker, &container_name, code).await?;

    Ok(result)
}

pub async fn build_and_run_container(
    config: &Config,
    docker: &Docker,
    dockerfile_path: &str,
    language: &str,
) -> Result<String, Box<dyn Error>> {
    let image_name = format!("{}_executor:latest", language);
    // Create tar archive for build context
    let tar_path = &config.paths.tar_path;
    let docker_file_name = &config.constants.dockerfile;
    let dockerfile_name = create_tar_archive(dockerfile_path, tar_path, docker_file_name)?; 
println!("Using dockerfile_name: '{}'", dockerfile_name);
    // Use a sync File, not tokio::fs::File, because bollard expects a blocking Read stream
    let mut file = tokio::fs::File::open(tar_path).await?;

    let mut contents = Vec::new();
file.read_to_end(&mut contents).await?;
    // Build image options
    let build_options = BuildImageOptions {
        dockerfile: dockerfile_name,
        t: image_name.clone(),
        rm: true,
        ..Default::default()
    };

    // Start the image build stream
    let mut build_stream = docker.build_image(build_options, None, Some(contents.into()));

    // Print docker build output logs
    while let Some(build_output) = build_stream.next().await {
        match build_output {
            Ok(output) => {
                if let Some(stream) = output.stream {
                    print!("{}", stream);
                }
            }
            Err(e) => {
                eprintln!("Error during image build: {}", e);
                return Err(Box::new(e));
            },
        }
    }

    println!("Docker image '{}' built successfully!", image_name);

    // Create container config
    let container_name = format!("{}_executor_container", language);
    let config = ContainerConfig {
        image: Some(image_name),
        host_config: Some(HostConfig {
            port_bindings: Some(
                [(
                    "5001/tcp".to_string(),
                    Some(vec![PortBinding {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some("5001".to_string()),
                    }]),
                )]
                .iter()
                .cloned()
                .collect(),
            ),
            ..Default::default()
        }),
        ..Default::default()
    };
    // Create container
    docker
        .create_container(
            Some(CreateContainerOptions {
                name: &container_name,
                platform: None,
            }),
            config,
        )
        .await?;
    println!("Container '{}' created successfully.", container_name);

    // Start container
    docker
        .start_container(&container_name, None::<StartContainerOptions<String>>)
        .await?;
    println!("Container '{}' started successfully!", container_name);

    Ok(container_name)
}

async fn execute_code_in_container(
    docker: &Docker,
    container_name: &str,
    code: &str,
) -> Result<String, Box<dyn Error>> {
    let shell_command = format!("echo '{}' > script.py && python script.py", code);
    let exec_options = CreateExecOptions {
        cmd: Some(vec!["sh", "-c", &shell_command]),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        ..Default::default()
    };

    let exec = docker.create_exec(container_name, exec_options).await?;
    let output = docker.start_exec(&exec.id, None).await?;

    match output {
        StartExecResults::Attached { mut output, .. } => {
            let mut result = String::new();
            while let Some(Ok(log)) = output.next().await {
                match log {
                    bollard::container::LogOutput::StdOut { message } => {
                        result.push_str(&String::from_utf8_lossy(&message));
                    }
                    bollard::container::LogOutput::StdErr { message } => {
                        result.push_str(&String::from_utf8_lossy(&message));
                    }
                    _ => {}
                }
            }
            Ok(result)
        }
        _ => Err("Failed to execute code in container".into()),
    }
}


fn create_tar_archive(dockerfile_path: &str, tar_path: &str, docker_file_name :&String) -> Result<String, Box<dyn Error>> {
    let tar_file = File::create(tar_path)?;
    let mut tar_builder = Builder::new(tar_file);

    // The name for the Dockerfile *inside* the tar archive.
    // Docker usually expects "Dockerfile" at the root of the build context.
    // let name_in_tar = docker_file_name;
    tar_builder.append_path_with_name(dockerfile_path, docker_file_name)?;
    tar_builder.finish()?;
    println!("Tar archive created at {} containing {} from {}", tar_path, docker_file_name, dockerfile_path);

    Ok(docker_file_name.to_string()) // Return the name that was actually used in the tar
}
