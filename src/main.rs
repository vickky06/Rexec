
mod proto;
mod service;
mod docker;

mod config;

use service::ExecutorService;
use proto::executor::code_executor_server::CodeExecutorServer;
use crate::config::{Config, GLOBAL_CONFIG};

use tonic::transport::Server;
use tonic_reflection::server::Builder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("config.toml");
    GLOBAL_CONFIG.set(config).expect("Config already set");
    println!("Config loaded: {:?}\n\n", GLOBAL_CONFIG.get().unwrap());

    let addr = ("[::1]:".to_owned() + &GLOBAL_CONFIG.get().unwrap().build.service_port.to_string()).parse()?;
    let service = ExecutorService::default();

     // Configure the reflection service
     let reflection_service = Builder::configure()
     .register_encoded_file_descriptor_set(proto::executor::FILE_DESCRIPTOR_SET)
     .build()?;

    println!("Server listening on {} for {} service", addr,&GLOBAL_CONFIG.get().unwrap().build.service_name);

    Server::builder()
        .add_service(CodeExecutorServer::new(service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}