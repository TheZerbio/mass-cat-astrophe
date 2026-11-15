use std::sync::{Arc, LazyLock, Mutex, MutexGuard};
use axum::{routing::post, Json, Router};
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::Html;
use axum::routing::get;
use serde::{Deserialize, Serialize};
use crate::game_communictaion::game_ui_routing;
use crate::status::{status_page};

mod status;
mod player_input;
mod game_communictaion;

const PORT: i64 = 8901;

#[derive(Serialize, Deserialize)]
enum PlayerStatus{
    CREATED,
    PLAYING, IDLE, AFK, DISCONNECTED
}
#[derive(Serialize)]
struct Player{
    score: f64,
    name: String,
    is_master: bool,
    similarities: Vec<f64>,
    #[serde(skip_serializing)]
    token: u16, // used to detect stale sessions and kick players rather than a security measure
    status: PlayerStatus,
    idle_time: usize,
}

enum GameState {
    ACTIVE, STARTING, CREATED, DISCONNECTED,
    RECONNECTING,
    STOPPED, IDLE
}
struct Game {
    state: GameState,
    id: usize,
    input_sum: [f64; 2],
    num_inputs: usize,
    last_avg_input: [f64; 2], // the avg input of the last tick once that was applied
    // (to try to make it fair to the first one )
    players:  Vec<Player>,
    master_input: bool,
}

fn is_active(game: &Game)->bool{
    match game.state {
        GameState::STOPPED => false,
        GameState::DISCONNECTED => false,
        _ => true
    }
}
fn generate_player_token()->u16{
    let random_number = rand::random::<u16>();
    random_number
}
fn validate_player_token(game_id: usize, player_id: usize, token: u16) ->bool{
    with_player(game_id, player_id, |player|{
        player.token == token
    }).unwrap_or(false)
}

static ACTIVE_GAMES: LazyLock<Mutex<Vec<Game>>> = LazyLock::new(|| Mutex::new(vec![]));

#[inline]
fn get_active_games() ->  MutexGuard<'static, Vec<Game>> {
    ACTIVE_GAMES.lock().expect("failed to get active games")
}
fn with_player<R>(game_id:usize, player_id:usize, f: impl FnOnce(&mut Player)->R)->Option<R>{
    let mut active_games = get_active_games();
    let game = active_games.get_mut(game_id)?;
    let player = game.players.get_mut(player_id)?;
    Some(f(player))
}

fn with_game<R>(game_id:usize, mut f: impl FnOnce(&mut Game)->R)->Option<R>{
    let mut active_games = get_active_games();
    let game = active_games.get_mut(game_id)?;
    Some(f(game))
}
#[tokio::main]
async fn main(){
    let port: i64 = std::env::args()
        .nth(1)                                   // args[0] is the program itself, [1] the first argument
        .and_then(|p| p.parse().ok())
        .unwrap_or(PORT);
    let router = routes();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
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
        if !is_active(game) {
            free_slot = game.id as usize;
            break;
        }
    }
    let new_game = Game { state: GameState::CREATED, id: free_slot, input_sum: [0.0; 2], num_inputs: 0, last_avg_input: [0.; 2], players: vec![], master_input: false };
    if free_slot >= ag.len() {ag.push(new_game);} else {ag[free_slot] = new_game;};
    return Json(serde_json::json!({"game_id": free_slot }));
}

#[derive(Serialize)]
struct PlayerCreatedDTO{
    name: String,
    id: usize,
    token: u16,
}
async fn register_player(Path((game_id, name)): Path<(usize, String)>)
    -> Result<Json<PlayerCreatedDTO>, (StatusCode, &'static str)>{
    if!game_exists(game_id) {
        return Err((StatusCode::GONE, "no active game with that id"))}
    
    let mut ag = get_active_games();
    if ag[game_id].players.iter().any(|player| player.name == name){
        return  Err((StatusCode::CONFLICT,"player name already exists"));
    }
    let name_copy = name.clone();
    let token = generate_player_token();
    ag[game_id].players.push(
        Player{
                    score: 0.0,
                    name,
                    is_master: false,
                    similarities: vec![],
                    token,
                    status: PlayerStatus::CREATED,
                    idle_time: 0,
        });
    return Ok(Json(PlayerCreatedDTO{
        name: name_copy,
        id: ag[game_id].players.len()-1,
        token,
    }));
}

async fn elevate_player(Path((game_id, name)): Path<(usize, String)>){
    if !game_exists(game_id) {return;}
    let mut ag = get_active_games();
    ag[game_id].players.iter_mut().for_each(
        |player| {
        player.is_master = if player.name == name {!player.is_master} else {false};
        println!("Player {} is changed master to {}", name, player.is_master);
        })

}


async fn remove_player(Path((game_id, name)): Path<(usize, String)>){
    if !game_exists(game_id) {return;}
    let mut ag = get_active_games();
    ag[game_id].players.iter_mut().filter(|player| player.name == name).for_each(
        | player| {player.token = 0;/* removing would change the indices, but removing
        the token makes his session stale*/ player.status = PlayerStatus::DISCONNECTED;}
    );
    println!("kicked player {}", name);
}

async fn close_game(Path(game_id): Path<usize>){
    if !game_exists(game_id){return;}
    let mut ag = get_active_games();
    ag[game_id].state = GameState::STOPPED;
}

fn game_exists(game_id: usize)->bool{
    let ag = get_active_games();
    return game_id < ag.len() && is_active(&ag[game_id]);
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
    token: Option<u16>,
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
                    if (params.token == None ||
                        !validate_player_token(game_id, player_id, params.token.unwrap()) ){
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




