use std::fmt::format;
use std::time::Duration;
use axum::extract::{Path, WebSocketUpgrade};
use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum::response::Response;
use axum::{Json, Router};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{game_exists, get_active_games, GameState, with_game, Game, Player};
use crate::GameState::{ACTIVE, IDLE, RECONNECTING, STARTING};

const UPDATES_PER_SECOND: u64 = 25;
const RETRYS_BEFORE_CLOSE: u64 = 60 * UPDATES_PER_SECOND;
pub(crate) const PLAYER_AFK_TIMEOUT: u64 = 3 * UPDATES_PER_SECOND;

pub(crate) fn game_ui_routing(router: Router) -> Router {
    router
        .route("/game/{game_id}/ws/controls", get(gui_ws_upgrader))
        .route("/game/{game_id}/statistics/{player_id}", get(get_player_statistics))
        .route("/game/{game_id}/statistics", get(get_player_statistics))
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
    if !game_exists(game_id){

        // check if it might be a valid rebuild of the connection
        if !with_game(game_id, |game|{match game.state {GameState::DISCONNECTED=> true, _ => false} }).unwrap_or(false)
        {let _ =socket.send
        (Message::Close((Some(CloseFrame{
            code: 404, reason: "game not found".into()})))
        ).await;
            return;};

    }
    if (game_in_progress(game_id)) {return;} // already running
    let mut interval = tokio::time::interval(Duration::from_millis(1000/UPDATES_PER_SECOND));
    let mut retries:u64 = RETRYS_BEFORE_CLOSE ;
    loop {
        interval.tick().await;
        if (!game_in_progress(game_id)) {continue}

        let dto = game_tick(game_id);
        let send_result = socket.send(Message::Text(serde_json::to_string(&dto).unwrap().into())).await;
        if send_result.is_err(){
            println!("connection lost {retries} for {game_id} with error: {send_result:?}");
            set_reconnecting(game_id);
            retries -= 1;
            if (retries > 0){
                continue;}
            set_disconnected(game_id);
            let _ = socket.send
            (Message::Close((Some(CloseFrame{
                code: 1000, reason: format!("Could not connect to client after {} tries", RETRYS_BEFORE_CLOSE).into()})))
            ).await;
            println!("connection closed for {game_id}");
            return;
        } else {
            set_active(game_id);
            retries = RETRYS_BEFORE_CLOSE;
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
    JSONBody{x: game.last_avg_input[0], y: game.last_avg_input[1], num_inputs: last_num_input }
}

#[derive(Deserialize)]
struct StatisticsRequestDto{
    game_id: usize,
    player_id: Option<usize>,
}
async fn get_player_statistics(Path(StatisticsRequestDto{game_id, player_id}): Path<StatisticsRequestDto>)->Json<serde_json::Value>{
    with_game(game_id, |game|{
        aggregate_player_similarities(game, UPDATES_PER_SECOND);
        match player_id {
            None => {
                Json(json!({ "players": game.players }))
            }
            Some(id) => {
                let player = &game.players[id];
                Json(serde_json::to_string(&player).unwrap().parse().unwrap())
            }
        }
    }).unwrap()


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

fn set_reconnecting(game_id: usize){
    with_game(game_id, |game| {game.state =GameState::RECONNECTING}).unwrap();
}
fn set_active(game_id: usize){
    with_game(game_id, |game| {game.state =GameState::ACTIVE}).unwrap();
}
fn set_disconnected(game_id: usize){
    with_game(game_id, |game| {game.state =GameState::DISCONNECTED}).unwrap();
}
pub(crate) fn get_player_afk_timeout()->u64{PLAYER_AFK_TIMEOUT}

fn aggregate_player_similarities(game: &mut Game, rate: u64)
{
    game.players.iter_mut().for_each(|player| {
        let mut sum = 0.;
        let mut num = 0;
        player.similarities.iter_mut().for_each(|sim| {
            sum += *sim;
            num += 1;
            if (num % rate) == 0 {
                player.similarities_agg.push(sum / num as f64);
                num = 0;
            }
        });
        if (num > 0) {
            player.similarities_agg.push(sum / num as f64); // push the remaining ones
        }
        player.similarities = vec![];
    })

}