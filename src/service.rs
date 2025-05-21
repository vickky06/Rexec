
use tonic::{Request, Response, Status};
use crate::proto::executor::code_executor_server::CodeExecutor;
use crate::proto::executor::{ExecuteRequest, ExecuteResponse};
use crate::docker::docker_manager;

use crate::config::Config;


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
        match handle_request( &language, &code).await {
            Ok(output) => println!("Execution Result: {}", output),
            Err(e) => eprintln!("Error: {}", e),
        }
        let reply = ExecuteResponse {
            message: "Request received".into(),
        };
        Ok(Response::new(reply))
    }
}
pub async fn handle_request(
    language: &str,
    _code: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let config = Config::from_file("config.toml");

    println!("Handling request for language: {}", language);
    let code = r#"print("Hello, World!")"#;
    let result = docker_manager::handle_request(
        &config,
        language,
        code).await?;
    Ok(result)
}