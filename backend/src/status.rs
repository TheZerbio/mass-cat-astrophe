use axum::response::{Html, IntoResponse};
use axum::http::header;

use crate::{get_active_games, GameState, is_active};

const PAGE: &str = include_str!("../static/status.html");
const CSS: &str = include_str!("../static/status.css");

// GET /status
pub async fn status_page() -> Html<String> {
    let games = get_active_games();
    let mut html = String::new();

    for game in games.iter() {
        let class = if is_active(game) { "game" } else { "game inactive" };
        html += &format!("<div class=\"{class}\"><h2>Game {}</h2>", game.id);
        if is_active(game) {
            html += &format!("<button class=\"close\" onclick=\"closeGame({})\">close game</button>", game.id);
        }
        // average of the inputs collected so far in this tick (none yet -> 0,0)
        let avg = if game.num_inputs > 0 {
            [game.input_sum[0] / game.num_inputs as f64, game.input_sum[1] / game.num_inputs as f64]
        } else {
            [0.0, 0.0]
        };
        html += &format!(
            "<table class=\"stats\">             <tr><td>state</td><td>{}</td></tr>             <tr><td>input_sum</td><td>[{:.2}, {:.2}]</td></tr>             <tr><td>num_inputs</td><td>{}</td></tr>             <tr><td>current avg</td><td>[{:.2}, {:.2}]</td></tr>             <tr><td>last_avg_input</td><td>[{:.2}, {:.2}]</td></tr>             </table>",
            format_state(&game.state),
            game.input_sum[0], game.input_sum[1],
            game.num_inputs,
            avg[0], avg[1],
            game.last_avg_input[0], game.last_avg_input[1],
        );

        if game.players.is_empty() {
            html += "<p class=\"empty\">no players</p>";
        } else {
            html += "<ul>";
            for (player_id, p) in game.players.iter().enumerate() {
                let name = escape(&p.name);
                html += &format!(
                    "<li><span class=\"pid\">#{player_id}</span> {name} - score: {:.2} - {}",
                    p.score,
                    similarity_summary(&p.similarities)
                );
                if p.is_master {
                    html += " <span class=\"master\">master</span>";
                } else {
                    html += &format!(
                        " <button onclick=\"elevate({}, this.dataset.name)\" data-name=\"{name}\">make master</button>",
                        game.id
                    );
                }
                html += &format!(
                    " <button class=\"kick\" onclick=\"kick({}, this.dataset.name)\" data-name=\"{name}\">kick</button></li>",
                    game.id
                );
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

fn format_state(state: &GameState)-> String {
    match state {
        GameState::CREATED => "⚪".to_string(),
        GameState::STARTING =>"🔵".to_string(),
        GameState::ACTIVE => "🟢".to_string(),
        GameState::IDLE => "🟤".to_string(),
        GameState::RECONNECTING => "🟠".to_string(),
        GameState::DISCONNECTED => "🔴".to_string(),
        GameState::STOPPED => "🟥".to_string(),
    }
}

// GET /status.css
pub async fn status_css() -> impl IntoResponse {
    ([(header::CONTENT_TYPE, "text/css")], CSS)
}

// "similarity: n values, avg 0.82, last 0.91" (or "no similarities yet")
fn similarity_summary(values: &[f64]) -> String {
    match values.last() {
        None => "no similarities yet".to_string(),
        Some(last) => {
            let avg = values.iter().sum::<f64>() / values.len() as f64;
            format!("similarity: {} values, avg {avg:.2}, last {last:.2}", values.len())
        }
    }
}

// player names come from users, so stop them from injecting HTML
fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}
