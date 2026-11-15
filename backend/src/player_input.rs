use axum::extract::{Path, Query, WebSocketUpgrade};
use axum::extract::ws::{Message, WebSocket};
use axum::http::{header, StatusCode};
use axum::response::{Html, IntoResponse, Response};
use axum::{Json, Router};
use axum::routing::get;
use qrcode::QrCode;
use qrcode::render::svg;
use serde::Deserialize;
use std::time::Instant;
use crate::{get_active_games, is_active, validate_player_token, PlayerStatus};
use crate::game_communictaion::get_player_afk_timeout;
use crate::PlayerStatus::{AFK, PLAYING};

pub(crate) fn joystick_routing(router: Router) -> Router {
    router.route("/game/{game_id}/js_controls/{player_id}/{token}/ws", get(joy_ws_upgrader))
        .route("/qr.svg", get(qr_svg_from_str))
        .route("/player/check_integrity/{game_id}/{player_id}/{token}", get(check_player_integrity))
        .route("/stats", get(serve_leaderboard))
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
    let mut games = get_active_games();
    // no indexing here: a panic while holding the lock would hit every game
    let Some(game) = games.get_mut(game_id) else { return };
    if !is_active(game) { return; }
    let last_avg_input = game.last_avg_input;
    let Some(player) = game.players.get_mut(player_id) else { return };

    // only remember the latest input here; game_tick combines all players once per tick
    player.last_input = [x, y];
    player.last_input_at = Some(Instant::now());

    if !player.is_master {
        if x.hypot(y) <= 0.01 {
            player.status = PlayerStatus::IDLE;
            player.idle_time += 1;
            if player.idle_time > get_player_afk_timeout() as usize {
                player.status = AFK;
                return;
            }
        } else {
            player.status = PLAYING;
            player.idle_time = 0;
        }
    }

    // calculate player deviation
    let similarity: f64 = dot_prod_4values(last_avg_input[0], last_avg_input[1], x, y);
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

#[derive(Deserialize)]
struct PlayerDTO{
    game_id: usize,
    player_id: usize,
    token: u16,
}
async fn serve_leaderboard(Query(PlayerDTO{game_id, player_id, token}): Query<PlayerDTO>)->Html<String>{
    const PAGE: &str = include_str!("../static/leaderboard.html");
    return Html(PAGE.to_string());
}