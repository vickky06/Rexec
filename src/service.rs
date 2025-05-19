// use tonic::{Request, Response, Status};

// use executor::code_executor_server::CodeExecutor;
// use executor::{ExecuteRequest, ExecuteResponse};
// pub mod executor {
//     tonic::include_proto!("executor");

//     pub const FILE_DESCRIPTOR_SET: &[u8] =
//     tonic::include_file_descriptor_set!("executor_descriptor");
// }
// #[derive(Debug, Default)]
// pub struct ExecutorService;

// #[tonic::async_trait]
// impl CodeExecutor for ExecutorService {
//     async fn execute(
//         &self,
//         _request: Request<ExecuteRequest>,
//     ) -> Result<Response<ExecuteResponse>, Status> {
//         let reply = ExecuteResponse {
//             message: "Request received".into(),
//         };
//         Ok(Response::new(reply))
//     }
// }

// filepath: /Users/viveksingh/RUST/DSA-engine/dsa-engine/src/service.rs
use tonic::{Request, Response, Status};
use crate::proto::executor::code_executor_server::CodeExecutor;
use crate::proto::executor::{ExecuteRequest, ExecuteResponse};
use crate::docker::docker_manager;



#[derive(Debug, Default)]
pub struct ExecutorService;

#[tonic::async_trait]
impl CodeExecutor for ExecutorService {
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        // Extract the request data
        let request_data = request.into_inner();
        println!("Received request: {:?}", request_data);
        let language = request_data.language.to_lowercase();
        let code = request_data.code;
        let dockerfile_path = match language.as_str() {
            "python" => "./Dockerfile.python",
            "javascript" => "./Dockerfile.javascript",
            "java" => "./Dockerfile.java",
            _ => {
                return Err(Status::invalid_argument(format!(
                    "Unsupported language: {}",
                    language
                )))
            }
        }; 
        match handle_request(dockerfile_path, &language, &code).await {
            Ok(output) => println!("Execution Result: {}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
        // match build_and_run_docker_image(dockerfile_path, &language).await {
        //     Ok(container_name) => {
        //         println!("Container started: {}", container_name);

        //     }
        //     Err(e) => {
        //         eprintln!("Error during Docker operations: {}", e);
        //         return Err(Status::internal("Failed to process the request"));
        //     }
        // }
            // eprintln!("Error during Docker operations: {}", e);
            // return Err(Status::internal("Failed to process the request"));
        // }
        let reply = ExecuteResponse {
            message: "Request received".into(),
        };
        Ok(Response::new(reply))
    }
}
pub async fn handle_request(
    dockerfile_path: &str,
    language: &str,
    _code: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Handling request for language: {}", language);
    let code = r#"print("Hello, World!")"#;
    let result = docker_manager::handle_request(language, code).await?;
    // Build and run the Docker container
    // let container_name = build_and_run_container(dockerfile_path, language).await?;
    // print!("Container name: {}", container_name);
    // // Connect to Docker
    // let docker = Docker::connect_with_local_defaults()?;
    // let code = r#"print("Hello, World!")"#;
    // let result = execute_code_in_container(&docker, &container_name, code).await?;

    Ok(result)
}