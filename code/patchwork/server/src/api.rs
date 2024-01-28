use std::{collections::HashMap, net::SocketAddr};

use crate::serialization::PatchworkState;
use axum::{
    extract::{
        self,
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get, post},
    Json, Router,
};
use futures_util::{stream::StreamExt, SinkExt};
use lazy_static::lazy_static;
use patchwork_lib::{GameOptions, Patchwork};
use uuid::Uuid;

lazy_static! {
    static ref GAMES: std::sync::Mutex<HashMap<String, Patchwork>> = std::sync::Mutex::new(HashMap::new());
}

pub fn api_router() -> Router {
    Router::new()
        // .route("/available_players")
        // .route("/get_valid_actions(game_id, state)")
        // .route("/is_valid_action(game_id, state, action)")
        // .route("/do_action(game_id, state)")
        .route("/new-game", post(new_game_handler))
        .route("/upi", get(ws_handler)) // set_option player
        .fallback_service(any(not_found))
}

async fn new_game_handler(extract::Json(payload): extract::Json<Option<GameOptions>>) -> impl IntoResponse {
    let state = Patchwork::get_initial_state(payload);
    let id = Uuid::new_v4().to_string();
    GAMES.lock().unwrap().insert(id.clone(), state.clone());
    Json(PatchworkState { state, id })
}

async fn ws_handler(ws: WebSocketUpgrade, ConnectInfo(addr): ConnectInfo<SocketAddr>) -> impl IntoResponse {
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
