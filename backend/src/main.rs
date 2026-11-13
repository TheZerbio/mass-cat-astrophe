use std::fmt::format;
use std::sync::{Arc, LazyLock, Mutex};
use axum::{routing::post, Json, Router};

const PORT: i64 = 8900;

struct Player{
    score: f64,
    name: str

}
struct Game {
    is_active: bool,
    id: usize,
    avg_input: [f64; 2],
    players: Vec<Player>,
}

static ACTIVE_GAMES: LazyLock<Mutex<Vec<Game>>> = LazyLock::new(|| Mutex::new(vec![]));

#[tokio::main]
async fn main(){
    let router = routes();
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{PORT}")).await.unwrap();
}
fn routes() -> Router {
    let mut router = Router::new();
    router = router.route("/game/new", post(create_new_game));
    router
}


async fn create_new_game()-> Json<usize>{
// find first inactive game
    let mut  ag = ACTIVE_GAMES.lock().expect("failed to get active games");
    let mut free_slot = ag.len();
    for game in ag.iter_mut(){
        if !game.is_active{
            free_slot = game.id as usize;
            break;
        }
    }
    let new_game = Game {is_active: true, id: free_slot, avg_input: [0.0; 2], players: vec![]};
    if free_slot >= ag.len() {ag.push(new_game);} else {ag[free_slot] = new_game;};
    return Json(free_slot);
}

async fn register_player()



