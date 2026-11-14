use axum::response::{Html, IntoResponse};
use axum::http::header;

use crate::ACTIVE_GAMES;

const PAGE: &str = include_str!("../static/status.html");
const CSS: &str = include_str!("../static/status.css");

// GET /status
pub async fn status_page() -> Html<String> {
    let games = ACTIVE_GAMES.lock().unwrap();
    let mut html = String::new();

    for game in games.iter() {
        let class = if game.is_active { "game" } else { "game inactive" };
        html += &format!("<div class=\"{class}\"><h2>Game {}</h2>", game.id);
        if game.is_active {
            html += &format!("<button class=\"close\" onclick=\"closeGame({})\">close game</button>", game.id);
        }
        html += &format!(
            "<p>active: {}<br>avg_input: [{:.2}, {:.2}]</p>",
            game.is_active, game.avg_input[0], game.avg_input[1]
        );

        if game.players.is_empty() {
            html += "<p class=\"empty\">no players</p>";
        } else {
            html += "<ul>";
            for p in game.players.iter() {
                let name = escape(&p.name);
                if p.is_master {
                    html += &format!("<li>{name} - score: {:.2} <span class=\"master\">master</span> <button class=\"kick\" onclick=\"kick({}, this.dataset.name)\" data-name=\"{name}\">kick</button></li>", p.score, game.id);
                } else {
                    html += &format!(
                        "<li>{name} - score: {:.2} <button onclick=\"elevate({}, this.dataset.name)\" data-name=\"{name}\">make master</button> <button class=\"kick\" onclick=\"kick({}, this.dataset.name)\" data-name=\"{name}\">kick</button></li>",
                        p.score, game.id, game.id
                    );
                }
            }
            html += "</ul>";
        }
        html += "</div>";
    }

    if games.is_empty() {
        html = "<p class=\"empty\">no games yet</p>".into();
    }

    Html(PAGE.replace("{{GAMES}}", &html))
}

// GET /status.css
pub async fn status_css() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css")], CSS)
}

// player names come from users, so stop them from injecting HTML
fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
