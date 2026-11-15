use axum::extract::{Path, Query, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Json, Router};
use axum::routing::get;
use qrcode::QrCode;
use qrcode::render::svg;
use serde::Deserialize;
use crate::{get_active_games, player_exists, validate_player_token, PlayerStatus};
use crate::game_communictaion::get_player_afk_timeout;
use crate::PlayerStatus::{AFK, PLAYING};

pub(crate) fn joystick_routing(router: Router) -> Router {
    router.route("/game/{game_id}/js_controls/{player_id}/{token}/ws", get(joy_ws_upgrader))
        .route("/qr.svg", get(qr_svg_from_str))
        .route("/player/check_integrity/{game_id}/{player_id}/{token}", get(check_player_integrity))
}
async fn joy_ws_upgrader(ws: WebSocketUpgrade, Path((game_id, player_id, token)): Path<(usize, usize, u16)>)->Response{
     ws.on_upgrade(move |socket| joy_ws_handler(socket, game_id, player_id, token))
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
    token: u16,
}
async fn joy_ws_handler(mut socket: WebSocket, game_id: usize, player_id: usize, token: u16) {
    if !validate_player_token(game_id, player_id, token){return;}
    while let Some(Ok(message)) = socket.recv().await{
        let Message::Text(content) = message else {continue};
        let Ok(JSONBody{x, y, token}) = serde_json::from_str::<JSONBody>(content.as_str()) else { continue };

        if (!validate_player_token(game_id, player_id, token)){
            return;
        }

        let mut input = PlayerInput{game_id, player_id, x, y};
        input = norm_inputs(input);

        on_player_input(input);
    }
}
fn on_player_input(PlayerInput{game_id, player_id, x, y}: PlayerInput){
    if (!player_exists(game_id, player_id)){ return;}
    let mut games = get_active_games();
    let game = &mut games[game_id];
    if game.players[player_id].is_master {
        if x.hypot(y) > 0.01{
        game.input_sum = [x, y];
        game.master_input = true;
        game.num_inputs = 1;}
    } else if (!game.master_input){
        let player = &mut game.players[player_id];
            if (x.hypot(y) <= 0.01){
                player.status = PlayerStatus::IDLE;
                player.idle_time += 1;
                if player.idle_time > get_player_afk_timeout() as usize {
                    player.status = AFK;
                    return;
                }
            }else {
                player.status = PLAYING;
                player.idle_time = 0;
            }
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
#[derive(Deserialize)]
struct QRQUERY{text:String}
async fn qr_svg_from_str(Query(QRQUERY{text}): Query<QRQUERY>) -> impl IntoResponse {
    let code = QrCode::new(text.as_bytes()).unwrap();
    let image = code.render::<svg::Color>().min_dimensions(256, 256).build();
    ([(header::CONTENT_TYPE, "image/svg+xml")], image)
}

async fn check_player_integrity(Path((game_id, player_id, token)): Path<(usize, usize, u16)>)
    ->StatusCode{
    if validate_player_token(game_id, player_id, token){StatusCode::OK} else {StatusCode::NOT_FOUND}
}