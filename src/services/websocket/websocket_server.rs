use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

use crate::{
    models::{docker_models::DockerSupportedLanguage, websocket_message_model::WebSocketMessage},
    services::{
        all_session_services::{
            session_cache_service::{ Session, SessionCache},
            session_service::update_create_session,
        },
        validation_services::language_validation::get_validator,
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
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(mut websocket) => {
                    println!("New WebSocket connection");

                    while let Some(msg) = websocket.next().await {
                        match msg {
                            Ok(tungstenite::Message::Text(input_text)) => {
                                println!("Received message: {}", input_text);
                                let mut text = sanitize_code_content(&input_text);
                                println!("Sanitized message: {}", text);
                                match serde_json::from_str::<WebSocketMessage>(&text) {
                                    Ok(message) => {
                                        let session_cache: &'static SessionCache = SessionCache::new();
                                        println!("Parsed WebSocketMessage: {:?}", message);
                                        let in_mem_session = update_create_session(&message);
                                        match in_mem_session {
                                            Ok(session) => {
                                                // Since the session already has been updated if existed, we can insert it into the cache
                                                session_cache.insert_session(session.clone());
                                                // ONLY FOR TESTING PURPOSES
                                                let session_validation_message =
                                                    valid_session_process(session, &session_cache);
                                                println!(
                                                    "Session processin validation: {}",
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
                                                // text.push_str(&format!(
                                                //     " (Session ID: {}, Language: {}, Code: {})",
                                                //     message.session_id,
                                                //     message.language,
                                                //     message.get_code_string()
                                                // ));
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
                                        // Optionally send an error response back
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
                                // Echo the message back (placeholder for syntax validation)
                                text.push_str(" (echoed)");
                                if let Err(e) =
                                    websocket.send(tungstenite::Message::Text(text)).await
                                {
                                    eprintln!("WebSocket send error: {}", e);
                                    break;
                                }
                            }
                            Ok(tungstenite::Message::Close(_)) => {
                                println!("WebSocket connection closed");
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
                Err(e) => eprintln!("Error during WebSocket handshake: {}", e),
            }
        });
    }

    Ok(())
}

fn valid_session_process(session: Session, session_cache: &'static SessionCache) -> String {
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

async fn close_connection(websocket: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, session: Session, session_cache: &'static SessionCache) {
    if let Err(e) = websocket.close(None).await {
        eprintln!("Error closing WebSocket connection: {}", e);
    } else {
       session_cache.remove_session(&session.session_id);
        println!("WebSocket connection closed successfully");
    }
}