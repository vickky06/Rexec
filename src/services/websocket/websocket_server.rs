use std::any::Any;
use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

use crate::{
    models::{docker_models::DockerSupportedLanguage, websocket_message_model::WebSocketMessage},
    services::{
        all_session_services::{
            session_cache_service::{Session, SessionCache},
            session_service::update_create_session,
        },
        helper_services::config_service::{get_global_config, get_global_config_mut},
        validation_services::language_validation::get_validator,
        websocket::websocket_sessionpool_service::ConnectionManager,
    },
    utils::helper_utils::sanitize_code_content,
};

pub async fn run_websocket_server(
    addr: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Starting WebSocket server on {}", addr);
    let listener = TcpListener::bind(addr)
        .await
        .expect("Failed to bind to address ");
    println!("WebSocket server listening on {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        // let peer_addr: SocketAddr = stream.peer_addr().unwrap();

        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(mut websocket) => {
                    println!("New WebSocket connection");
                    let mut session_cache: Option<&'static SessionCache> = None; // initialize properly
                    let mut session: Option<Session> = None; // store Session by value
                    while let Some(msg) = websocket.next().await {
                        match msg {
                            Ok(tungstenite::Message::Text(input_text)) => {
                                println!("Received message: {}", input_text);
                                let mut text = sanitize_code_content(&input_text);
                                println!("Sanitized message: {}", text);

                                match serde_json::from_str::<WebSocketMessage>(&text) {
                                    Ok(message) => {
                                        let session_id = message.session_id.clone();
                                        let new_session_cache = session_management_add().await;
                                        session_cache = Some(new_session_cache);
                                        // websocket_connection_pool_add(peer_addr, session_id).await;
                                        println!("Parsed WebSocketMessage: {:?}", message);

                                        match update_create_session(&message) {
                                            Ok(ss) => {
                                                let new_session = ss.clone();
                                                session = Some(new_session.clone());
                                                new_session_cache
                                                    .insert_session(new_session.clone());

                                                let session_validation_message =
                                                    valid_session_process(
                                                        &new_session,
                                                        &new_session_cache,
                                                    );
                                                println!(
                                                    "Session process validation: {}",
                                                    session_validation_message
                                                );

                                                let is_syntex_valid = syntex_validation(
                                                    message.get_language().unwrap(),
                                                    message.get_code_string(),
                                                );

                                                text.push_str(&format!(
                                                    " (Session ID: {}, Language: {}, Code: {}, Syntax Valid: {})",
                                                    message.session_id,
                                                    message.language,
                                                    message.get_code_string(),
                                                    is_syntex_valid
                                                ));
                                            }
                                            Err(e) => {
                                                text.push_str(&format!(
                                                    " Error: {}, Code: {}, Type: {:?}",
                                                    e.message, e.error_code, e.error_type
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to parse WebSocketMessage: {}", e);

                                        let error_response =
                                            format!("Error parsing message: {}", e.to_string());
                                        if let Err(e) = websocket
                                            .send(tungstenite::Message::Text(error_response))
                                            .await
                                        {
                                            eprintln!("WebSocket send error: {}", e);
                                            break;
                                        }
                                    }
                                }

                                if let Err(e) =
                                    websocket.send(tungstenite::Message::Text(text)).await
                                {
                                    eprintln!("WebSocket send error: {}", e);
                                    break;
                                }
                            }
                            Ok(tungstenite::Message::Close(_)) => {
                                println!("WebSocket connection closed");
                                close_connection(&mut websocket, session.as_ref(), session_cache)
                                    .await;
                                break;
                            }
                            Err(e) => {
                                eprintln!("WebSocket error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error during WebSocket handshake: {}", e);
                }
            }
        });
    }

    Ok(())
}
fn valid_session_process(session: &Session, session_cache: &'static SessionCache) -> String {
    const TAG: &str = "DEBUG||";
    let verify_session = session_cache.get_session(&(session).session_id.as_str());
    match &verify_session {
        Some(session_ref) => {
            let v_session = session_ref.value();
            // println!("Session verified from cache: {:?}", v_session);
            if v_session.session_id != session.session_id {
                return format!("{}Session ID mismatch after insertion", TAG);
            } else {
                return format!("{}Session ID matches after insertion", TAG);
            }
        }
        None => {
            return format!("{}Session not found in cache", TAG);
        }
    }
}

fn syntex_validation(language: DockerSupportedLanguage, code: String) -> bool {
    let validator = get_validator(language.clone());
    match validator.validate(&code) {
        Ok(_) => {
            println!("✅ {:?} syntax is valid", language);
            true
        }
        Err(e) => {
            eprintln!("❌ Syntax error: {}", e);
            false
        }
    }
}

async fn close_connection(
    websocket: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    session: Option<&Session>,
    session_cache: Option<&'static SessionCache>,
) {
    // Attempt to send a close frame to the client
    if let Err(e) = websocket.send(tungstenite::Message::Close(None)).await {
        eprintln!("Error sending WebSocket close frame: {}", e);
    }

    if let (Some(session), Some(cache)) = (session, session_cache) {
        cache.remove_session(&session.session_id);
        println!("Session {} removed from cache", session.session_id);
    }
}

async fn session_management_add() -> &'static SessionCache {
    let session_cache_instance = get_global_config(|config| config.clone())
        .await
        .session_cache_service;
    match session_cache_instance {
        Some(cache) => cache,
        None => {
            let session_cache = SessionCache::new();
            let mut config = get_global_config_mut().await;
            config.set_session_cache(session_cache);
            // set_global_config(config);
            session_cache
        }
    }
}

async fn websocket_connection_pool_add(socket_addr: SocketAddr, session_id_value: String) {
    let webpool_id = format!("{:?}", socket_addr);
    let web_socket_session_pool = get_global_config(|config| config.clone())
        .await
        .websocket_seesion_pool;
    match web_socket_session_pool {
        Some(wsp) => {
            wsp.add_connection(&webpool_id, session_id_value.clone());
        }
        None => {
            let wsp = ConnectionManager::get_connection_manager();
            let mut config = get_global_config_mut().await;
            config.set_websocket_connection_manager(wsp);
            // set_global_config(config);
        }
    }
}
