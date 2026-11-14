mod game;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{
        Query, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::header,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use serde::Deserialize;
use tokio::sync::broadcast;
use tower_http::cors::CorsLayer;

use game::{Game, InputMsg, Vec2};

struct AppState {
    game: Mutex<Game>,
    /// Pre-serialized JSON messages (ticks + scores) for all game clients.
    tx: broadcast::Sender<Arc<str>>,
    join_url: String,
    port: u16,
}

type App = Arc<AppState>;

fn env_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

/// Finds the LAN address phones can reach (no packet is actually sent).
fn lan_ip() -> IpAddr {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| s.connect("8.8.8.8:80").map(|_| s))
        .and_then(|s| s.local_addr())
        .map(|a| a.ip())
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST))
}

#[tokio::main]
async fn main() {
    let port: u16 = env_or("PORT", 8080);
    let tick_hz: f64 = env_or("TICK_HZ", 30.0_f64).clamp(1.0, 240.0);
    let public_url = std::env::var("PUBLIC_URL").unwrap_or_else(|_| format!("http://{}:{port}", lan_ip()));
    let join_url = format!("{}/controller", public_url.trim_end_matches('/'));

    let (tx, _) = broadcast::channel(64);
    let app: App = Arc::new(AppState { game: Mutex::new(Game::new(tick_hz)), tx, join_url, port });

    tokio::spawn(tick_loop(app.clone(), tick_hz));

    let router = Router::new()
        .route("/", get(display_page))
        .route("/controller", get(controller_page))
        .route("/qr.svg", get(qr_svg))
        .route("/api/info", get(info))
        .route("/api/input", get(input))
        .route("/api/scores", get(scores))
        .route("/api/scores/reset", post(reset_scores))
        .route("/ws/controller", get(ws_controller))
        .route("/ws/game", get(ws_game))
        .layer(CorsLayer::permissive())
        .with_state(app.clone());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("failed to bind port");
    println!("CATastrophe backend running");
    println!("  display (QR):  http://localhost:{}/", app.port);
    println!("  phones join:   {}", app.join_url);
    println!("  game socket:   ws://localhost:{}/ws/game   ({tick_hz} Hz)", app.port);
    axum::serve(listener, router).await.unwrap();
}

async fn tick_loop(app: App, tick_hz: f64) {
    let mut interval = tokio::time::interval(Duration::from_secs_f64(1.0 / tick_hz));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    // Push scores ~4 times per second, ticks every tick.
    let scores_every = (tick_hz / 4.0).round().max(1.0) as u64;
    loop {
        interval.tick().await;
        let (tick, scores) = {
            let mut g = app.game.lock().unwrap();
            let snap = g.tick();
            let n = snap.tick;
            let tick = serde_json::to_string(snap).unwrap();
            let scores = (n % scores_every == 0).then(|| serde_json::to_string(&g.scores()).unwrap());
            (tick, scores)
        };
        // Errors only mean "nobody is listening right now".
        let _ = app.tx.send(tick.into());
        if let Some(s) = scores {
            let _ = app.tx.send(s.into());
        }
    }
}

// ---------- pages ----------

async fn display_page() -> Html<&'static str> {
    Html(include_str!("../static/display.html"))
}

async fn controller_page() -> Html<&'static str> {
    Html(include_str!("../static/controller.html"))
}

async fn qr_svg(State(app): State<App>) -> Response {
    let code = qrcode::QrCode::new(app.join_url.as_bytes()).expect("url fits in a QR code");
    let svg = code
        .render::<qrcode::render::svg::Color>()
        .min_dimensions(320, 320)
        .quiet_zone(true)
        .build();
    ([(header::CONTENT_TYPE, "image/svg+xml")], svg).into_response()
}

// ---------- HTTP API ----------

async fn info(State(app): State<App>) -> impl IntoResponse {
    Json(serde_json::json!({
        "joinUrl": app.join_url,
        "qrUrl": "/qr.svg",
        "gameSocket": "/ws/game",
    }))
}

async fn input(State(app): State<App>) -> impl IntoResponse {
    Json(app.game.lock().unwrap().last_tick().clone())
}

async fn scores(State(app): State<App>) -> impl IntoResponse {
    Json(app.game.lock().unwrap().scores())
}

async fn reset_scores(State(app): State<App>) -> impl IntoResponse {
    app.game.lock().unwrap().reset_scores();
    Json(serde_json::json!({ "ok": true }))
}

// ---------- WebSockets ----------

#[derive(Deserialize)]
struct JoinQuery {
    id: Option<u32>,
    name: Option<String>,
}

async fn ws_controller(ws: WebSocketUpgrade, Query(q): Query<JoinQuery>, State(app): State<App>) -> Response {
    ws.on_upgrade(move |socket| controller_session(socket, q, app))
}

async fn controller_session(mut socket: WebSocket, q: JoinQuery, app: App) {
    let welcome = app.game.lock().unwrap().join(q.id, q.name);
    let (id, session) = (welcome.id, welcome.session);
    let msg = serde_json::to_string(&welcome).unwrap();

    if socket.send(Message::Text(msg.into())).await.is_ok() {
        let mut status = tokio::time::interval(Duration::from_millis(500));
        loop {
            tokio::select! {
                msg = socket.recv() => match msg {
                    Some(Ok(Message::Text(t))) => {
                        if let Ok(m) = serde_json::from_str::<InputMsg>(&t) {
                            app.game.lock().unwrap().set_input(id, session, Vec2::from(m));
                        }
                    }
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                    _ => {}
                },
                _ = status.tick() => {
                    // Tell the phone how well it is doing.
                    let s = app.game.lock().unwrap().scores();
                    let total = s.players.iter().filter(|p| p.connected).count();
                    let Some(me) = s.players.into_iter().find(|p| p.id == id) else { break };
                    let msg = serde_json::json!({
                        "type": "status",
                        "rank": me.rank,
                        "accuracy": me.accuracy,
                        "averageAccuracy": me.average_accuracy,
                        "score": me.score,
                        "connectedPlayers": total,
                    });
                    if socket.send(Message::Text(msg.to_string().into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
    app.game.lock().unwrap().leave(id, session);
}

async fn ws_game(ws: WebSocketUpgrade, State(app): State<App>) -> Response {
    ws.on_upgrade(move |socket| game_session(socket, app))
}

#[derive(Deserialize)]
struct GameCommand {
    #[serde(rename = "type")]
    kind: String,
}

async fn game_session(mut socket: WebSocket, app: App) {
    let mut rx = app.tx.subscribe();
    loop {
        tokio::select! {
            msg = rx.recv() => match msg {
                Ok(json) => {
                    if socket.send(Message::Text(json.as_ref().into())).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => break,
            },
            msg = socket.recv() => match msg {
                Some(Ok(Message::Text(t))) => {
                    if let Ok(cmd) = serde_json::from_str::<GameCommand>(&t) {
                        if cmd.kind == "resetScores" {
                            app.game.lock().unwrap().reset_scores();
                        }
                    }
                }
                Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                _ => {}
            },
        }
    }
}
