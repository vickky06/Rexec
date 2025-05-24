use crate::config::{self, GLOBAL_CONFIG};
use crate::docker::docker_manager;
use crate::proto::executor::code_executor_server::CodeExecutor;
use crate::proto::executor::{ExecuteRequest, ExecuteResponse};
use crate::session_management_service::{SessionManagement, SessionManagementService as sms};
use tonic::{Request, Response, Status};
#[derive(Debug, Default)]
pub struct ExecutorService;

#[tonic::async_trait]
impl CodeExecutor for ExecutorService {
    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        // Extract the session_id from metadata before moving request
        let session_id = request
            .metadata()
            .get("session_id")
            .and_then(|v: &tonic::metadata::MetadataValue<tonic::metadata::Ascii>| v.to_str().ok())
            .unwrap_or("anonymous")
            .to_string();
        // Now move the request
        let request_data = request.into_inner();
        println!("Received request: {:?}", request_data);
        let language = request_data.language.to_lowercase();
        let code = request_data.code;
        match session_handler(&session_id, &language, &code).await {
            Ok(output) => {
                println!("Execution Result: {}", output);
                Ok(Response::new(ExecuteResponse { message: output }))
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                Err(Status::internal(format!("Execution error: {}", e)))
            }
        }
    }
}

pub async fn session_handler(
    session_id: &str,
    language: &str,
    _code: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let code = r#"print("Hello, World!")"#;
    let result = docker_manager::handle_request(session_id, language, code).await?;
    Ok(result)
}

pub async fn _session_handler_new(
    session_id: &str,
    language: &str,
    _code: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    println!("Handling request for language: {}", language);
    let code = r#"print("Hello, World!")"#;
    match GLOBAL_CONFIG
        .get()
        .unwrap()
        .session_management_service
        .get_session_image(session_id, language)
        .await
    {
        Ok(image) => {
            println!("Session image for {}: {}", session_id, image);
            match docker_manager::execute_code_in_existing_container(&image, code).await {
                Ok(result) => {
                    println!("Execution Result: {}", result);
                    Ok(result)
                }
                Err(e) => {
                    eprintln!("Error executing code in container: {:?}", e);
                    Err(e)
                }
            }

            // let result = docker_manager::handle_request(session_id, language, code).await?;
            // Ok(result)
        }

        Err(e) => {
            eprintln!("Error retrieving session image: {:?}", e);
            let result = docker_manager::handle_request(session_id, language, code).await?;
            Ok(result)
        }
    }
}
