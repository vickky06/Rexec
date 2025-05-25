mod cleanup_service;
mod config;
mod docker;
mod proto;
mod service;
mod session_management_service;

use crate::config::{Config, GLOBAL_CONFIG};
use proto::executor::code_executor_server::CodeExecutorServer;
use service::ExecutorService;

use cleanup_service::{ActivityType, CleanupService};
use tokio::signal;
use tonic::transport::Server;
use tonic_reflection::server::Builder;

use std::env;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command-line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <run|grpcui>", args[0]);
        return Ok(());
    }

    let command = &args[1];
    println!("Command: {}", command);

    let mut config = Config::from_file("config.toml");

    let server_pod_id = Uuid::new_v4(); // Replace with actual server pod ID
    config.session_management_service = session_management_service::SessionManagementService::new();
    config.build.service_name = format!("{} {}", config.build.service_name, server_pod_id);
    GLOBAL_CONFIG.set(config).expect("Config already set");

    let addr = ("[::1]:".to_owned() + &GLOBAL_CONFIG.get().unwrap().build.service_port.to_string())
        .parse()?;
    let service = ExecutorService::default();

    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(proto::executor::FILE_DESCRIPTOR_SET)
        .build()?;

    match command.as_str() {
        cmd if cmd.contains("run") => {
            println!(
                "Server listening on {} for {} service",
                addr,
                &GLOBAL_CONFIG.get().unwrap().constants.service_name
            );
            let svc = Server::builder()
                .add_service(CodeExecutorServer::new(service))
                .add_service(reflection_service);

            // Create a shutdown signal future
            let shutdown_signal = async {
                signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C handler");
                println!("Shutdown signal received. Cleaning up...");
            };

            // Run the server and listen for shutdown signal
            tokio::select! {
                res = svc.serve(addr) => {
                    if let Err(e) = res {
                        eprintln!("Server error: {}", e);
                        return Ok(());
                    }
                }
                _ = shutdown_signal => {
                    // This branch runs when Ctrl+C is received
                }
            }
            let container =  cleanup_service::CLEANUP_ACTIVITY_CONTAINER;
            let all_tars = cleanup_service::CLEANUP_ACTIVITY_ALL_TARS;
            println!("Cleaning up resources...");
            println!("Container: {}", container);
            println!("All Tars: {}", all_tars);
            // Perform cleanup operations
            // Cleanup logic here
            let cleanup_service = CleanupService {};
            let activity = ActivityType::new(
                Some(container.to_string()),
                None,
                Some(all_tars.to_string() ),
                None,
                Some(Vec::new()),
            );
            cleanup_service.cleanup(activity).await?;

            println!("Server exited cleanly.");
        }
        cmd if cmd.contains("grpcui") => {
            println!(
                "GRPC server starting on {}",
                "127.0.0.1:".to_owned()
                    + &GLOBAL_CONFIG.get().unwrap().build.grpc_ui_port.to_string()
            );
            // Add any additional logic for grpcui mode here
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Usage: {} <run|grpcui>", args[0]);
        }
    }

    println!("Server running successfully.");
    Ok(())
}

// mod cleanup_service;
// mod config;
// mod docker;
// mod proto;
// mod service;
// mod session_management_service;

// use crate::config::{Config, GLOBAL_CONFIG};
// use proto::executor::code_executor_server::CodeExecutorServer;
// use service::ExecutorService;

// use cleanup_service::{ActivityType, CleanupService};
// use tokio::signal;
// use tonic::transport::Server;
// use tonic_reflection::server::Builder;

// use uuid::Uuid;
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let mut config = Config::from_file("config.toml");

//     let server_pod_id = Uuid::new_v4(); // Replace with actual server pod ID
//     config.session_management_service = session_management_service::SessionManagementService::new();
//     config.build.service_name = format!("{} {}", config.build.service_name, server_pod_id);
//     GLOBAL_CONFIG.set(config).expect("Config already set");

//     let addr = ("[::1]:".to_owned() + &GLOBAL_CONFIG.get().unwrap().build.service_port.to_string())
//         .parse()?;
//     let service = ExecutorService::default();

//     let reflection_service = Builder::configure()
//         .register_encoded_file_descriptor_set(proto::executor::FILE_DESCRIPTOR_SET)
//         .build()?;

//     println!(
//         "Server listening on {} for {} service",
//         addr,
//         &GLOBAL_CONFIG.get().unwrap().constants.service_name
//     );
//     let svc = Server::builder()
//         .add_service(CodeExecutorServer::new(service))
//         .add_service(reflection_service);

//     println!(
//         "GRPC server starting on {}",
//         "127.0.0.1:".to_owned() + &GLOBAL_CONFIG.get().unwrap().build.grpc_ui_port.to_string()
//     );
//     // Create a shutdown signal future
//     let shutdown_signal = async {
//         signal::ctrl_c()
//             .await
//             .expect("Failed to install Ctrl+C handler");
//         println!("Shutdown signal received. Cleaning up...");
//     };

//     // Run the server and listen for shutdown signal
//     tokio::select! {
//         res = svc.serve(addr) => {
//             if let Err(e) = res {
//                 eprintln!("Server error: {}", e);
//             }
//         }
//         _ = shutdown_signal => {
//             // This branch runs when Ctrl+C is received
//         }
//     }

//     // Cleanup logic here
//     let cleanup_service = CleanupService {};
//     let activity = ActivityType::new(
//         Some("container".to_string()),
//         None,
//         Some("all tars".to_string()),
//         None,
//     );
//     cleanup_service.cleanup(activity).await?;

//     println!("Server exited cleanly.");
//     Ok(())
// }
