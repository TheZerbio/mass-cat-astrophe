use axum::extract::{Path, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::response::Response;
use axum::Router;
use axum::routing::get;
use serde::Deserialize;
use crate::{get_active_games, player_exists};

pub(crate) fn joystick_routing(router: Router) -> Router {
    router.route("/game/{game_id}/js_controlls/{player_id}/ws", get(joy_ws_upgrader))
}
async fn joy_ws_upgrader(ws: WebSocketUpgrade, Path((game_id, player_id)): Path<(usize, usize)>)->Response{
     ws.on_upgrade(move |socket| joy_ws_handler(socket, game_id, player_id))
}

struct PlayerInput{
    game_id: usize,
    player_id: usize,
    x: f64,
    y: f64,
}
#[derive(Deserialize)]
struct JSONBody{
    x:f64,
    y:f64,
}
async fn joy_ws_handler(mut socket: WebSocket, game_id: usize, player_id: usize) {
    if !player_exists(game_id, player_id){return;}
    while let Some(Ok(message)) = socket.recv().await{
        let Message::Text(content) = message else {continue};
        let Ok(JSONBody{x, y}) = serde_json::from_str::<JSONBody>(content.as_str()) else { continue };

        let mut input = PlayerInput{game_id, player_id, x, y};
        input = norm_inputs(input);

        on_player_input(input);
    }
}
fn on_player_input(PlayerInput{game_id, player_id, x, y}: PlayerInput){
    let mut games = get_active_games();
    let game = &mut games[game_id];
    if game.players[player_id].is_master && x+y > 0.01{
        game.input_sum = [x, y];
        game.master_input = true;
        game.num_inputs = 1;
    } else if (!game.master_input){
            game.input_sum = [game.input_sum[0] + x, game.input_sum[1] + y];
            game.num_inputs+=1;
    }

    // calculate player deviation
    let similarity: f64 = dot_prod_4values(game.last_avg_input[0], game.last_avg_input[1], x, y);
    let player = &mut game.players[player_id];
    player.similarities.push(similarity);
    player.score += similarity;
}

fn dot_prod_4values(x1: f64, x2:f64, y1: f64, y2:f64)->f64{
    return (x1 *y1) +(x2 *y2);
}
fn norm_inputs(mut input: PlayerInput)->PlayerInput {
    let distance = (input.x * input.x + input.y * input.y).sqrt();
    if distance > 1. {
        input.x /= distance;
        input.y /= distance;
    }
    input
}