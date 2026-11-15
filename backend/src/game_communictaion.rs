use std::time::{Duration, Instant};
use axum::extract::{Path, WebSocketUpgrade};
use axum::extract::ws::{CloseFrame, Message, WebSocket};
use axum::http::StatusCode;
use axum::response::Response;
use axum::{Json, Router};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::{GameState, with_game, Game, Player, PlayerStatus};
use crate::GameState::{ACTIVE, IDLE, RECONNECTING, STARTING};

const UPDATES_PER_SECOND: u64 = 25;
/// how long a game waits for its display (Unity) to reconnect before it counts as disconnected
const RECONNECT_GRACE: Duration = Duration::from_secs(60);
/// while the game isn't running, ping the display every this many ticks (= once per second)
const KEEPALIVE_TICKS: u64 = UPDATES_PER_SECOND;
/// a player's last input counts for this long; older = the phone stopped sending
const INPUT_MAX_AGE: Duration = Duration::from_millis(250);
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

/// What the handler should do on a tick, decided while holding the games lock.
enum TickStep {
    Send(JSONBody),
    KeepAlive,
    Replaced, // a newer connection took over this game
    Stopped,  // game was closed or doesn't exist anymore
}

async fn gui_ws_handler(mut socket: WebSocket, game_id: usize) {
    // Claim the game for this connection. Every new connection gets a higher generation number,
    // so the handler of an older (dead) connection notices it was replaced and stops touching the game.
    let claimed = with_game(game_id, |game| {
        match game.state {
            GameState::STOPPED => None,
            _ => {
                game.connection_gen += 1;
                // (re)connect: continue where the game was
                game.state = if game.started { ACTIVE } else { STARTING };
                Some(game.connection_gen)
            }
        }
    }).flatten();
    let Some(my_gen) = claimed else {
        // 4000-4999 are the close codes an application may define itself
        let _ = socket.send(Message::Close(Some(CloseFrame {
            code: 4404, reason: "game not found".into() }))).await;
        return;
    };
    println!("game {game_id}: display connected (connection {my_gen})");

    let mut interval = tokio::time::interval(Duration::from_millis(1000 / UPDATES_PER_SECOND));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut ticks_since_send: u64 = 0;
    let mut close_reason: Option<(u16, &'static str)> = None;

    loop {
        tokio::select! {
            // reading is the only way to notice that the client closed the connection
            message = socket.recv() => match message {
                None | Some(Err(_)) | Some(Ok(Message::Close(_))) => break,
                Some(Ok(_)) => {} // text from the game and pongs are ignored
            },
            _ = interval.tick() => {
                let step = with_game(game_id, |game| {
                    if game.connection_gen != my_gen { return TickStep::Replaced; }
                    match game.state {
                        GameState::STOPPED => TickStep::Stopped,
                        ACTIVE | IDLE => TickStep::Send(game_tick(game)),
                        _ => TickStep::KeepAlive,
                    }
                }).unwrap_or(TickStep::Stopped);

                let message = match step {
                    TickStep::Send(dto) => Message::Text(serde_json::to_string(&dto).unwrap().into()),
                    TickStep::KeepAlive => {
                        // nothing to send before the game starts: ping once per second, so proxies
                        // don't cut the idle connection and a dead client is noticed
                        ticks_since_send += 1;
                        if ticks_since_send < KEEPALIVE_TICKS { continue; }
                        Message::Ping(Vec::new().into())
                    }
                    TickStep::Replaced => { close_reason = Some((4409, "replaced by a newer connection")); break; }
                    TickStep::Stopped => { close_reason = Some((1000, "game stopped")); break; }
                };
                ticks_since_send = 0;
                if let Err(error) = socket.send(message).await {
                    // a failed send means the connection is gone, retrying on it can't succeed
                    println!("game {game_id}: connection {my_gen} lost: {error:?}");
                    break;
                }
            }
        }
    }

    if let Some((code, reason)) = close_reason {
        let _ = socket.send(Message::Close(Some(CloseFrame { code, reason: reason.into() }))).await;
    }

    // Only the newest connection may change the game state.
    let waiting_for_reconnect = with_game(game_id, |game| {
        if game.connection_gen != my_gen || matches!(game.state, GameState::STOPPED) { return false; }
        game.state = RECONNECTING;
        true
    }).unwrap_or(false);
    println!("game {game_id}: display connection {my_gen} ended");

    if waiting_for_reconnect {
        // give the game some time to reconnect before it counts as disconnected (its slot gets reusable then)
        tokio::spawn(async move {
            tokio::time::sleep(RECONNECT_GRACE).await;
            with_game(game_id, |game| {
                if game.connection_gen == my_gen && matches!(game.state, RECONNECTING) {
                    game.state = GameState::DISCONNECTED;
                    println!("game {game_id}: no reconnect within {RECONNECT_GRACE:?}, disconnected");
                }
            });
        });
    }
}

/// Combined input of this tick: the average of the latest input of every player that is still sending.
/// Uses the last known input per player instead of "messages since the last tick", because phone messages
/// don't arrive evenly: a tick without a message must not look like "nobody is steering".
fn game_tick(game: &mut Game) -> JSONBody {
    let now = Instant::now();
    let is_fresh = |player: &Player| player.last_input_at
        .is_some_and(|at| now.duration_since(at) <= INPUT_MAX_AGE);

    let master = game.players.iter()
        .find(|p| p.is_master && is_fresh(p) && p.last_input[0].hypot(p.last_input[1]) > 0.01)
        .map(|p| p.last_input);

    let (sum, num_inputs) = match master {
        Some(input) => (input, 1), // the master overrides everyone else
        None => game.players.iter()
            .filter(|p| !p.is_master && is_fresh(p))
            .filter(|p| !matches!(p.status, PlayerStatus::AFK | PlayerStatus::DISCONNECTED))
            .fold(([0.0, 0.0], 0), |(sum, n), p|
                ([sum[0] + p.last_input[0], sum[1] + p.last_input[1]], n + 1)),
    };
    game.master_input = master.is_some();
    game.input_sum = sum;       // kept for the status page
    game.num_inputs = num_inputs;

    if num_inputs == 0 {
        game.state = IDLE;
        return JSONBody { x: 0.0, y: 0.0, num_inputs: 0 };
    }
    game.state = ACTIVE;
    game.last_avg_input = [sum[0] / num_inputs as f64, sum[1] / num_inputs as f64];
    JSONBody { x: game.last_avg_input[0], y: game.last_avg_input[1], num_inputs }
}

#[derive(Deserialize)]
struct StatisticsRequestDto{
    game_id: usize,
    player_id: Option<usize>,
}
async fn get_player_statistics(Path(StatisticsRequestDto{game_id, player_id}): Path<StatisticsRequestDto>)
    -> Result<Json<serde_json::Value>, StatusCode> {
    with_game(game_id, |game|{
        aggregate_player_similarities(game, UPDATES_PER_SECOND);
        match player_id {
            None => Ok(Json(json!({ "players": game.players }))),
            Some(id) => {
                let player = game.players.get(id).ok_or(StatusCode::NOT_FOUND)?;
                Ok(Json(json!(player)))
            }
        }
    }).unwrap_or(Err(StatusCode::NOT_FOUND))
}

async fn start_game(Path(game_id): Path<usize>) -> StatusCode {
    println!("Starting game {}", game_id);
    let started = with_game(game_id, |game| {
        if matches!(game.state, GameState::STOPPED) { return false; }
        game.started = true;
        // without a connected display the state stays as it is; the reconnect then continues the game
        if matches!(game.state, STARTING | GameState::CREATED | ACTIVE | IDLE) { game.state = ACTIVE; }
        true
    }).unwrap_or(false);
    if started { StatusCode::OK } else { StatusCode::NOT_FOUND }
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
