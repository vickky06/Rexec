// use tonic::transport::Server;

// mod service;
// use service::executor::code_executor_server::CodeExecutorServer;
// use service::ExecutorService;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "[::1]:50051".parse()?;
//     let service = ExecutorService::default();

//     println!("ExecutorServer listening on {}", addr);

//     Server::builder()
//         .add_service(CodeExecutorServer::new(service))
//         .serve(addr)
//         .await?;

//     Ok(())
// }
use tonic::transport::Server;
// use tonic_reflection::server::Builder;
// use tonic_reflection::server::ServerReflection;

mod service;
use service::executor::code_executor_server::CodeExecutorServer;
use service::{executor, ExecutorService};



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    println!("ExecutorServer listening on {}", addr);

    let reflection_service = tonic_reflection::server::Builder::configure()
    .register_encoded_file_descriptor_set(executor::FILE_DESCRIPTOR_SET)
    .build()?;
        

    Server::builder()
        .add_service(CodeExecutorServer::new(ExecutorService::default()))
        .add_service(reflection_service) // 👈 Add reflection here
        .serve(addr)
        .await?;

    Ok(())
}