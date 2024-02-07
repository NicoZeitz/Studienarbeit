use std::{collections::HashMap, net::SocketAddr};

use crate::serialization::PatchworkState;
use axum::{
    extract::{
        self,
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo, Path,
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct RunningGame {
    state: PatchworkState,
    player_1: String, // TODO: reconnect for player and so on
    player_2: String,
    ply: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Options {
    seed: Option<u64>,
}

lazy_static! {
    static ref GAMES: std::sync::Mutex<HashMap<Uuid, RunningGame>> = std::sync::Mutex::new(HashMap::new());
}

pub fn api_router() -> Router {
    Router::new()
        .route("/game/:uuid", post(game_handler))
        // .route("/available_players")
        // .route("/get_valid_actions(game_id, state)")
        // .route("/is_valid_action(game_id, state, action)")
        // .route("/do_action(game_id, state)")
        // .route("/upi/:uuid", get(ws_handler)) // set_option player
        .route("/upi", get(ws_handler)) // set_option player
        .fallback_service(any(not_found))
}

async fn game_handler(Path(uuid): Path<Uuid>, payload: Option<extract::Json<Options>>) -> impl IntoResponse {
    if let Some(game) = GAMES.lock().unwrap().get(&uuid) {
        // existing game
        return Json(game.clone());
    }

    // new game
    let new_game = RunningGame {
        state: PatchworkState(Patchwork::get_initial_state(
            payload.and_then(|o| o.seed).map(|seed| GameOptions { seed }),
        )),
        player_1: "player_1".to_string(),
        player_2: "player_2".to_string(),
        ply: 0,
    };

    GAMES.lock().unwrap().insert(uuid, new_game.clone());
    Json(new_game)
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
