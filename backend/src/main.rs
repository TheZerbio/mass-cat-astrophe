use std::fmt::format;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use axum::{routing::post, Json, Router};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use crate::game_communictaion::game_ui_routing;
use crate::status::{status_css, status_page};

mod status;
mod player_input;
mod game_communictaion;

const PORT: i64 = 8900;

#[derive(Serialize)]
struct Player{
    score: f64,
    name: String,
    is_master: bool,
    similarities: Vec<f64>,
}
struct Game {
    is_active: bool,
    id: usize,
    input_sum: [f64; 2],
    num_inputs: usize,
    last_avg_input: [f64; 2], // the avg input of the last tick once that was applied
    // (to try to make it fair to the first one )
    players:  Vec<Player>,
    master_input: bool,
}

static ACTIVE_GAMES: LazyLock<Mutex<Vec<Game>>> = LazyLock::new(|| Mutex::new(vec![]));

#[inline]
fn get_active_games() ->  MutexGuard<'static, Vec<Game>> {
    ACTIVE_GAMES.lock().expect("failed to get active games")
}

#[tokio::main]
async fn main(){
    let router = routes();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT}")).await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
fn routes() -> Router {
    let mut router = Router::new();
    router = router.route("/game/new", post(create_new_game))
        .route("/status", get(status_page))
        .route("/game/{game_id}/join/{name}", post(register_player))
        .route("/game/{game_id}/elevate/{name}", post(elevate_player))
        .route("/game/{game-id}/close", post(close_game))
        .route("/game/{game_id}/kick/{name}", post(remove_player))
        .route("/game/play", get(serve_js_pages));
    router = player_input::joystick_routing(router);
    router = game_ui_routing(router);




    // css files loaded from html
    router = router.route("/status.css", get(status::status_css));
    router
}


async fn create_new_game()-> Json<serde_json::Value> {
// find first inactive game
    let mut  ag = get_active_games();
    let mut free_slot = ag.len();
    for game in ag.iter_mut(){
        if !game.is_active{
            free_slot = game.id as usize;
            break;
        }
    }
    let new_game = Game {is_active: true, id: free_slot, input_sum: [0.0; 2], num_inputs: 0, last_avg_input: [0.; 2], players: vec![], master_input: false };
    if free_slot >= ag.len() {ag.push(new_game);} else {ag[free_slot] = new_game;};
    return Json(serde_json::json!({"game_id": free_slot }));
}

async fn register_player(Path((game_id, name)): Path<(usize, String)>)
    -> Result<Json<usize>, (StatusCode, &'static str)>{
    if!game_exists(game_id) {
        return Err((StatusCode::GONE, "no active game with that id"))}
    
    let mut ag = get_active_games();
    if ag[game_id].players.iter().any(|player| player.name == name){
        return  Err((StatusCode::CONFLICT,"player name already exists"));
    }
    ag[game_id].players.push(Player{score: 0.0, name, is_master: false, similarities: vec![] });
    return Ok(Json(ag[game_id].players.len()-1));
}

async fn elevate_player(Path((game_id, name)): Path<(usize, String)>){
    if !game_exists(game_id) {return;}
    let mut ag = get_active_games();
    ag[game_id].players.iter_mut().for_each(|mut player| {player.is_master = player.name == name;})
}


async fn remove_player(Path((game_id, name)): Path<(usize, String)>){
    if !game_exists(game_id) {return;}
    let mut ag = get_active_games();
    ag[game_id].players.retain(|player| player.name != name);
}

async fn close_game(Path(game_id): Path<usize>){
    if !game_exists(game_id){return;}
    let mut ag = get_active_games();
    ag[game_id].is_active = false;
}

fn game_exists(game_id: usize)->bool{
    let ag = get_active_games();
    return game_id < ag.len() && ag[game_id].is_active;
}
fn player_exists(game_id:usize, player_id: usize )->bool{
    if (game_exists(game_id)) {
        let ag = get_active_games();
        if ag[game_id].players.len() > player_id {return true;}
    }
    return false;

}

/// struct as params doesn't understand simple order logic
#[derive(Deserialize)]
struct QueryParameters{
    game_id: Option<usize>,
    player_id: Option<usize>,
}
async fn serve_js_pages(Query(params):Query<QueryParameters>)->Html<String>{
    match params.game_id {
        Some(game_id) => {
            match params.player_id {
                None => {
                    if (!game_exists(game_id)) {
                        return Html("<h1>This game doesn't exist (anymore)</h1>".to_string());
                    }
                    const PAGE: &str = include_str!("../static/register_players.html");
                    Html(PAGE.to_string())
                }
                Some(player_id) => {
                    if !player_exists(game_id, player_id) {
                        return Html(format!(
                            "<h1>Invalid session: Game or player doesn't exist</h1>\
                        <script>window.location.href='/game/play?game_id={}'</script>",
                            game_id).to_string())
                    }
                    const PAGE: &str = include_str!("../static/joystick.html");
                    return Html(PAGE.to_string());
                }
            }
        }
        None => { return Html("<h1>Link doesn't belong to any Game</h1>".to_string()); }
    }
}




