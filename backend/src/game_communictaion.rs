use std::time::Duration;
use axum::extract::{Path, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::response::Response;
use axum::{Json, Router};
use axum::routing::{get, post};
use serde::{Serialize};

use crate::{game_exists, get_active_games, GameState};
use crate::GameState::{ACTIVE, IDLE, RECONNECTING, STARTING};

const UPDATES_PER_SECOND: u64 = 25;
const RETRYS_BEFORE_CLOSE: u64 = 30 * UPDATES_PER_SECOND;

pub(crate) fn game_ui_routing(router: Router) -> Router {
    router
        .route("/game/{game_id}/ws/controls", get(gui_ws_upgrader))
        .route("/game/{game_id}/statistics/{player_id}", get(get_player_statistics))
        .route("/game/{game_id}/start", post(start_game))


}

async fn gui_ws_upgrader(ws: WebSocketUpgrade, Path(game_id): Path<usize>)->Response{
    println!("Upgrade called for game {}", game_id);
    ws.on_upgrade(move |socket| gui_ws_handler(socket, game_id))
}
#[derive(Serialize)]
struct JSONBody{
    x:f64,
    y:f64,
    num_inputs: usize,
}
async fn gui_ws_handler(mut socket: WebSocket, game_id: usize) {
    if !game_exists(game_id){return;}
    let mut interval = tokio::time::interval(Duration::from_millis(1000/UPDATES_PER_SECOND));
    let mut retries:usize = RETRYS_BEFORE_CLOSE as usize;
    loop {
        if (!game_in_progress(game_id)) {continue}
        interval.tick().await;
        let dto = game_tick(game_id);

        if socket.send(Message::Text(serde_json::to_string(&dto).unwrap().into())).await.is_err(){
            retries -= 1;
            if (retries > 0){continue;}
            let mut ag = get_active_games();
            let game = &mut ag[game_id];
            game.state = GameState::DISCONNECTED;
            break;
        }
    }

}

fn game_tick(game_id: usize) -> JSONBody{
    let mut ag = get_active_games();
    let game = &mut ag[game_id];
    if (game.num_inputs == 0) {
        game.state = GameState::IDLE;
        return JSONBody{x:0.0, y:0.0, num_inputs: 0 }
    }
    game.last_avg_input = [game.input_sum[0] / game.num_inputs as f64, game.input_sum[1] / game.num_inputs as f64];
    game.input_sum = [0., 0.];
    let last_num_input = game.num_inputs;
    game.num_inputs = 0;
    game.master_input = false;
    let meow = false;
    JSONBody{x: game.last_avg_input[0], y: game.last_avg_input[1], num_inputs: last_num_input }
}

async fn get_player_statistics(Path((game_id, player_id )): Path<(usize, Option<usize>)>)->Json<serde_json::Value>{
    let games = get_active_games();
    let game = &games[game_id];
    match player_id {
        None => {
            Json(serde_json::to_string(&game.players).unwrap().parse().unwrap())
        }
        Some(id) => {
            let player = &game.players[id];
            Json(serde_json::to_string(&player).unwrap().parse().unwrap())
        }
    }
}

async fn start_game(Path(game_id): Path<usize>){
    println!("Starting game {}", game_id);
    let mut games = get_active_games();
    let game = &mut games[game_id];
    game.state = GameState::ACTIVE;
}

fn game_in_progress(game_id: usize)->bool{
    let mut games = get_active_games();
    let game = &mut games[game_id];
    match game.state {
        ACTIVE => true,
        STARTING => false,
        RECONNECTING => true,
        IDLE => true,
        GameState::CREATED => {game.state = STARTING; return false;},
        GameState::DISCONNECTED => false,
        GameState::STOPPED => false,
        _ => false,
    }
}


