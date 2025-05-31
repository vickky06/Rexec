mod cleanup_service;
mod config;
mod docker;
mod proto;
mod service;
mod session_management_service;
mod utils;
mod validation_service;
mod language_executor;
mod websocket_server;

use crate::config::{Config, GLOBAL_CONFIG};
use proto::executor::code_executor_server::CodeExecutorServer;
use service::ExecutorService;
use cleanup_service::{ActivityType, CleanupService};
use session_management_service::SessionManagement;
use websocket_server::run_websocket_server;


use tokio::signal;
use tokio::time::Duration;
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

    let mut config = Config::new();

    let server_pod_id = Uuid::new_v4(); // Replace with actual server pod ID
    config.session_management_service = session_management_service::SessionManagementService::new();
    config.build.service_name = format!("{} {}", config.build.service_name, server_pod_id);
    GLOBAL_CONFIG.set(config).expect("Config already set");

    let grpc_addr = ("[::1]:".to_owned() + &GLOBAL_CONFIG.get().unwrap().build.service_port.to_string())
        .parse()?;

    // let websocket_addr = "[::1]:".to_owned() + &GLOBAL_CONFIG.get().unwrap().build.web_socket_port.to_string();

    let service = ExecutorService::default();

    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(proto::executor::FILE_DESCRIPTOR_SET)
        .build()?;

    match command.as_str() {
        cmd if cmd.contains("run") => {
            println!(
                "Server listening on {} for {} service",
                grpc_addr,
                &GLOBAL_CONFIG.get().unwrap().constants.service_name
            );

            let session_management_service = GLOBAL_CONFIG
                .get()
                .unwrap()
                .session_management_service
                .clone();

            tokio::spawn(async move {
                let cleanup_interval = Duration::from_secs(
                    GLOBAL_CONFIG
                        .get()
                        .unwrap()
                        .session_configs
                        .session_cleanup_interval,
                );
                loop {
                    if session_management_service.get_last_cleanup().await + cleanup_interval
                        <= std::time::Instant::now()
                    {
                        println!("Skipping cleanup, last cleanup was recent.");
                        tokio::time::sleep(cleanup_interval).await;
                        continue;
                    }
                    tokio::time::sleep(cleanup_interval).await;
                    let _ = session_management_service.cleanup_expired_sessions();
                    println!("Periodic session cleanup completed.");
                }
            });

            

            // Create a shutdown signal future
            let shutdown_signal = async {
                signal::ctrl_c()
                    .await
                    .expect("Failed to install Ctrl+C handler");
                println!("Shutdown signal received. Cleaning up...");
            };

            let svc = Server::builder()
                .add_service(CodeExecutorServer::new(service))
                .add_service(reflection_service);
            // Run the server and listen for shutdown signal
            tokio::select! {
                res = svc.serve(grpc_addr) => {
                    if let Err(e) = res {
                        eprintln!("Server error: {}", e);
                        return Ok(());
                    }
                }
                _ = shutdown_signal => {
                    // This branch runs when Ctrl+C is received
                }
            }
            let container = cleanup_service::CLEANUP_ACTIVITY_CONTAINER;
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
                Some(all_tars.to_string()),
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

   
    Ok(())
}
