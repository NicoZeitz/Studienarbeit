use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use dashmap::DashMap;
use lazy_static::lazy_static;
use patchwork_core::{Patchwork, Player};
use uuid::Uuid;

pub enum ActivePlayer {
    External,
    Internal(Arc<Mutex<dyn Player + Send>>),
}

pub struct ActiveGame {
    state: Patchwork,
    player_1: ActivePlayer,
    player_2: ActivePlayer,
}

lazy_static! {
    static ref GAMES: DashMap<Uuid, ActiveGame> = DashMap::new();
}

// pub fn get_or_start_game(uuid: Uuid)

// Player
// -> options (deserializer)
// -> get_options

// option

// {
//     type: number,
//     min: 0,
//     max: 0,
// }
// {
//     type: enum
//     values: [a, b, c]
// }
// {
//     type: bool
// }
// {
//     type: end_condition
//     values: [iterations(number), time(ms), flag]
// }
