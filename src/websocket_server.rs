// websocket_server.rs

use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};

pub async fn run_websocket_server(addr: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind(addr).await?;
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

                                // Echo the message back (placeholder for syntax validation)
                                text.push_str(" (echoed)");
                                if let Err(e) = websocket.send(tungstenite::Message::Text(text)).await {
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
