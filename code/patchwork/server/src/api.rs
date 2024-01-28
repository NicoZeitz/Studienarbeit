use std::net::SocketAddr;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get},
    Router,
};
use axum_extra::TypedHeader;
use futures_util::{stream::StreamExt, SinkExt};

pub fn api_router() -> Router {
    Router::new()
        .route("/upi", get(ws_handler))
        .fallback_service(any(not_found))
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    println!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(socket: WebSocket, who: SocketAddr) {
    let (mut sender, mut receiver) = socket.split();

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(_) => {
                println!("Received message from `{}`: {:?}", who, msg);
                let _ = sender.send(Message::Text("Hello!".to_string())).await;
            }
            Message::Binary(_) => {}
            Message::Ping(_) => {}
            Message::Pong(_) => {}
            Message::Close(_) => {
                break;
            }
        }
    }
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "404 Not Found")
}
