
mod proto;
mod service;
mod docker;

use service::ExecutorService;
use proto::executor::code_executor_server::CodeExecutorServer;

use tonic::transport::Server;
use tonic_reflection::server::Builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let service = ExecutorService::default();

     // Configure the reflection service
     let reflection_service = Builder::configure()
     .register_encoded_file_descriptor_set(proto::executor::FILE_DESCRIPTOR_SET)
     .build()?;

    println!("Server listening on {}", addr);

    Server::builder()
        .add_service(CodeExecutorServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}