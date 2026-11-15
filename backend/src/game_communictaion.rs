use std::time::Duration;
use axum::extract::{Path, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::response::Response;
use axum::{Json, Router};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{game_exists, get_active_games, GameState, with_game, Game, Player};
use crate::GameState::{ACTIVE, IDLE, RECONNECTING, STARTING};

const UPDATES_PER_SECOND: u64 = 25;
/// how long a running game waits for its display (Unity) to reconnect before it counts as DISCONNECTED
const RECONNECT_GRACE: Duration = Duration::from_secs(60);
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
    // Claim the game: every connection gets a new id, so an older handler notices it was replaced.
    let claimed = with_game(game_id, |game| {
        if matches!(game.state, GameState::STOPPED) { return None; }
        game.active_handler_id += 1;
        match game.state {
            GameState::CREATED | STARTING => game.state = STARTING,         // lobby: wait for /start
            RECONNECTING | GameState::DISCONNECTED => game.state = ACTIVE,  // only after the start -> continue
            _ => {}                                                         // ACTIVE / IDLE: leave as is
        }
        Some(game.active_handler_id)
    }).flatten();
    let Some(my_id) = claimed else { return; }; // game doesn't exist or was stopped
    println!("game {game_id}: display connected (handler {my_id})");

    let mut interval = tokio::time::interval(Duration::from_millis(1000 / UPDATES_PER_SECOND));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            // reading is the only way to notice that Unity closed the connection
            message = socket.recv() => match message {
                None | Some(Err(_)) | Some(Ok(Message::Close(_))) => break,
                Some(Ok(_)) => {} // anything Unity sends is ignored
            },
            _ = interval.tick() => {
                // decide inside the lock: None = stop, Some(None) = nothing to send, Some(Some(dto)) = send
                let Some(step) = with_game(game_id, |game| {
                    if game.active_handler_id != my_id || matches!(game.state, GameState::STOPPED) {
                        return None; // replaced by a newer connection, or closed
                    }
                    match game.state {
                        ACTIVE | IDLE => Some(Some(game_tick(game))),
                        _ => Some(None), // lobby: nothing to send yet
                    }
                }).flatten() else { break };
                let Some(dto) = step else { continue };

                // send outside the lock (never .await while holding the mutex)
                if let Err(error) = socket.send(Message::Text(serde_json::to_string(&dto).unwrap().into())).await {
                    println!("game {game_id}: handler {my_id} lost connection: {error:?}");
                    break; // the socket is dead for good, no retries
                }
            }
        }
    }

    // Only the newest handler may change the state, and only a running game counts as reconnecting.
    let waiting = with_game(game_id, |game| {
        if game.active_handler_id != my_id { return false; }
        if matches!(game.state, ACTIVE | IDLE) {
            game.state = RECONNECTING;
            return true;
        }
        false
    }).unwrap_or(false);
    println!("game {game_id}: handler {my_id} ended");

    if waiting {
        // no reconnect within the grace time -> DISCONNECTED, so the slot can be reused
        tokio::spawn(async move {
            tokio::time::sleep(RECONNECT_GRACE).await;
            with_game(game_id, |game| {
                if game.active_handler_id == my_id && matches!(game.state, RECONNECTING) {
                    game.state = GameState::DISCONNECTED;
                    println!("game {game_id}: no reconnect within {RECONNECT_GRACE:?}, disconnected");
                }
            });
        });
    }
}

fn game_tick(game: &mut Game) -> JSONBody{
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
    with_game(game_id, |game| {game.state = GameState::ACTIVE;});
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
                sum = 0.0;
            }
        });
        if (num > 0) {
            player.similarities_agg.push(sum / num as f64); // push the remaining ones
        }
        player.similarities = vec![];
    })

}