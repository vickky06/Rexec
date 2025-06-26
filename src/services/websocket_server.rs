use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;

use crate::{
    models::websocket_message_model::WebSocketMessage,
    services::{
        session_cache_service::{Session, SessionCache},
        session_service::update_create_session,
    },
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
                            Ok(tungstenite::Message::Text(mut text)) => {
                                println!("Received message: {}", text);
                                match serde_json::from_str::<WebSocketMessage>(&text) {
                                    Ok(message) => {
                                        let session_cache = SessionCache::new();
                                        println!("Parsed WebSocketMessage: {:?}", message);
                                        let in_mem_session = update_create_session(&message);
                                        match in_mem_session {
                                            Ok(session) => {
                                                // Since the session already has been updated if existed, we can insert it into the cache
                                                session_cache.insert_session(session.clone());
                                                // ONLY FOR TESTING PURPOSES
                                                let session_validation_message = valid_session_process(
                                                    session,
                                                    &session_cache,
                                                );
                                                println!(
                                                    "Session processin validation: {}",
                                                    session_validation_message
                                                );

                                                
                                            
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
