use crate::config::GLOBAL_CONFIG;
use bollard::Docker;
use bollard::container::{
    Config as ContainerConfig, CreateContainerOptions, StartContainerOptions,
};

use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::BuildImageOptions;
use bollard::models::{HostConfig, PortBinding};
use futures_util::stream::StreamExt;
use std::error::Error;
use std::fs::File;
use tar::Builder;
use tokio::io::AsyncReadExt;

use uuid::Uuid;
use crate::cleanup_service::{ActivityType, CleanupService};

pub async fn handle_request(language: &str, code: &str) -> Result<String, Box<dyn Error>> {
    let docker = Docker::connect_with_local_defaults()?;
    let config = GLOBAL_CONFIG.get().unwrap();
    // Select the appropriate Dockerfile
    let dockerfile_path = match language {
        "python" => &config.dockerfiles.python,
        "javascript" => &config.dockerfiles.javascript,
        "java" => &config.dockerfiles.java,
        _ => return Err(format!("Unsupported language: {}", language).into()),
    };

    // Build and run the container
    let container_name = build_and_run_container(&docker, dockerfile_path, language).await?;

    // Execute the code inside the container
    let result = execute_code_in_container(&docker, &container_name, code).await?;

    Ok(result)
}

pub async fn build_and_run_container(
    docker: &Docker,
    dockerfile_path: &str,
    language: &str,
) -> Result<String, Box<dyn Error>> {
    println!("Building and running container for language: {}", language);
    let config = GLOBAL_CONFIG.get().unwrap();
    let image_name = format!("{}_{}", config.constants.executor_image_name,language);
    // Create tar archive for build context
    
    let tar_path_base = &config.paths.tar_path; //returns "./docker/context/"
    // println!("tar_path_base: {}", tar_path_base);
    let ref tar_path_formatted = format!("{}{}_{}_{}", tar_path_base, Uuid::new_v4(), language, &config.constants.tar_file_name);
    let docker_file_name = &config.constants.dockerfile;
    let dockerfile_name = create_tar_archive(dockerfile_path, &tar_path_formatted, docker_file_name)?;
    println!("Using dockerfile_name: '{}'", dockerfile_name);
    // Use a sync File, not tokio::fs::File, because bollard expects a blocking Read stream
    let mut file = tokio::fs::File::open(tar_path_formatted).await?;

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
            }
        }
    }

    println!("Docker image '{}' built successfully!", image_name);

    // clear the tar async from tar_path_formatted
    let activity_to_clear_tar = ActivityType::new(
        None,
        None,
        None,
        Some(tar_path_formatted.to_string()),
    );
    let cleanup_service = CleanupService {};
    cleanup_service.cleanup(activity_to_clear_tar).await?;
    println!("Tar file '{}' removed successfully!", tar_path_formatted);

    // Create container config
    
    let container_name = format!("{}_{}",GLOBAL_CONFIG.get().unwrap().constants.executor_container_name, language);
    let created_by_tag = GLOBAL_CONFIG
        .get()
        .unwrap()
        .constants
        .docker_created_by_label
        .clone();
    let label: String = GLOBAL_CONFIG.get().unwrap().build.service_name.clone();

    let config = ContainerConfig {
        labels: Some([(created_by_tag, label)].iter().cloned().collect()),
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

fn create_tar_archive(
    dockerfile_path: &str,
    tar_path: &str,
    docker_file_name: &String,
) -> Result<String, Box<dyn Error>> {
    println!("Creating tar archive for Dockerfile: {}::\n{}", dockerfile_path, tar_path);
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
