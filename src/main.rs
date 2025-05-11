// use tonic::transport::Server;

// mod service;
// use service::executor::code_executor_server::CodeExecutorServer;
// use service::{executor, ExecutorService};



// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let addr = "[::1]:50051".parse()?;
//     println!("ExecutorServer listening on {}", addr);

//     let reflection_service = tonic_reflection::server::Builder::configure()
//     .register_encoded_file_descriptor_set(executor::FILE_DESCRIPTOR_SET)
//     .build()?;
        

//     Server::builder()
//         .add_service(CodeExecutorServer::new(ExecutorService::default()))
//         .add_service(reflection_service) // reflection here
//         .serve(addr)
//         .await?;

//     Ok(())
// }

// filepath: /Users/viveksingh/RUST/DSA-engine/dsa-engine/src/main.rs
mod proto;
mod service;

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