mod docker;
mod models;
mod proto;
mod services;
mod utils;

use proto::executor::code_executor_server::CodeExecutorServer;

use crate::services::{
    all_session_services::{
        session_management_service::SessionManagement,
        session_service::get_session_management_service,
    },
    helper_services::{
        cleanup_service,
        config_service::{get_global_config, set_global_config},
    },
    websocket::websocket_server::run_websocket_server,
};
use models::{
    cleanup_models::{ActivityType, CleanupService},
    config_models::Config,
    executor_models::ExecutorService,
};

use crate::models::port_models::PortsService as ps;
use std::env;
use std::net::SocketAddr;
use tokio::signal;
use tokio::time::Duration;
use tonic::transport::Server;
use tonic_reflection::server::Builder;
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
    config.init();
    config.build.service_name = format!("{} {}", config.build.service_name, server_pod_id);
    set_global_config(config);
    let ports_service = ps::new().await; //ports_service::PortsService::new();
    let address = ports_service.get_grpc_server_address();
    println!("gRPC server address: {}", address); // Add this line
    let grpc_addr: SocketAddr = address
        .parse()
        .expect("Failed to parse gRPC server address");

    let websocket_addr = ports_service.get_websocket_address();

    let execution_service = ExecutorService::default();

    let reflection_service = Builder::configure()
        .register_encoded_file_descriptor_set(proto::executor::FILE_DESCRIPTOR_SET)
        .build()?;

    let ws_handle = tokio::spawn(async move {
        if let Err(e) = run_websocket_server(&websocket_addr).await {
            eprintln!("WebSocket server error: {}", e);
        }
    });

    match command.as_str() {
        // will run with cargo run -- run
        cmd if cmd.contains("run") => {
            println!(
                "Initiating port {} for {} service",
                grpc_addr,
                &get_global_config(|config| config.clone())
                    .await
                    .constants
                    .service_name
            );

            let session_management_service = get_global_config(|config| config.clone())
                .await
                .session_management_service
                .clone();

            let _ = get_session_management_service();

            tokio::spawn(async move {
                let cleanup_interval = Duration::from_secs(
                    get_global_config(|config| config.clone())
                        .await
                        .session_configs
                        .session_cleanup_interval,
                );
                loop {
                    if session_management_service.unwrap().get_last_cleanup().await
                        + cleanup_interval
                        > std::time::Instant::now()
                    {
                        println!(
                            "Skipping cleanup, last cleanup was recent. {:?}",
                            session_management_service
                                .as_ref()
                                .unwrap()
                                .get_last_cleanup()
                                .await
                        );
                        tokio::time::sleep(cleanup_interval).await;
                        continue;
                    }
                    tokio::time::sleep(cleanup_interval).await;
                    let _ = session_management_service
                        .unwrap()
                        .cleanup_expired_sessions()
                        .await;
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

            let grpc_server = Server::builder()
                .add_service(CodeExecutorServer::new(execution_service))
                .add_service(reflection_service)
                .serve_with_shutdown(grpc_addr, shutdown_signal);
            // Run the server and listen for shutdown signal
            tokio::select! {
                     _ = grpc_server => {
                println!("gRPC server exited");
            }
            _ = ws_handle => {
                println!("WebSocket server exited");
            }
                }
            let container = cleanup_service::CLEANUP_ACTIVITY_CONTAINER;
            let all_tars = cleanup_service::CLEANUP_ACTIVITY_ALL_TARS;
            println!("Cleaning up resources...");
            // println!("Container: {}", container);
            // println!("All Tars: {}", all_tars);
            // Perform cleanup operations
            // Cleanup logic here
            let cleanup_service = CleanupService {};
            let activity = ActivityType::new(
                Some(container.to_string()),
                None,
                Some(all_tars.to_string()),
                None,
                Some(ports_service.get_all_ports()),
            );
            cleanup_service.cleanup(activity).await?;

            println!("Server exited cleanly.");
        }
        cmd if cmd.contains("grpcui") => {
            println!(
                "GRPC server starting on {}",
                ports_service.get_grpc_server_address()
            );
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            eprintln!("Usage: {} <run|grpcui>", args[0]);
        }
    }

    Ok(())
}
