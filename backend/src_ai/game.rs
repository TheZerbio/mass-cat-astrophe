//! Game logic: collects joystick inputs, combines them into one shared input
//! and scores every player by how close they are to that combined result.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Inputs shorter than this count as "not touching the stick".
pub const DEADZONE: f32 = 0.1;

#[derive(Clone, Copy, Default, Debug, Serialize, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub fn len(self) -> f32 {
        self.x.hypot(self.y)
    }

    /// Replaces non-finite components with 0 and clamps into the unit circle.
    pub fn sanitized(self) -> Self {
        let x = if self.x.is_finite() { self.x } else { 0.0 };
        let y = if self.y.is_finite() { self.y } else { 0.0 };
        let v = Vec2 { x, y };
        let l = v.len();
        if l > 1.0 { Vec2 { x: x / l, y: y / l } } else { v }
    }

    fn dist(self, o: Vec2) -> f32 {
        (self.x - o.x).hypot(self.y - o.y)
    }
}

/// What a phone sends: either `[x, y]` or `{"x": .., "y": ..}`, each in -1..1.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum InputMsg {
    Array([f32; 2]),
    Object { x: f32, y: f32 },
}

impl From<InputMsg> for Vec2 {
    fn from(m: InputMsg) -> Self {
        match m {
            InputMsg::Array([x, y]) | InputMsg::Object { x, y } => Vec2 { x, y },
        }
    }
}

struct Player {
    name: String,
    color: String,
    input: Vec2,
    connected: bool,
    /// Identifies the socket currently owning this player, so a stale socket
    /// closing late cannot disconnect a fresh reconnect.
    session: u64,
    last_accuracy: Option<f32>,
    /// Sum of accuracy * seconds -> "seconds of perfect agreement".
    score: f64,
    scored_seconds: f64,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PlayerInput {
    pub id: u32,
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TickSnapshot {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub tick: u64,
    /// Combined (normed) input: average of all active players, inside the unit circle.
    pub x: f32,
    pub y: f32,
    pub magnitude: f32,
    pub active_players: usize,
    pub connected_players: usize,
    pub players: Vec<PlayerInput>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScoreEntry {
    pub rank: usize,
    pub id: u32,
    pub name: String,
    pub color: String,
    pub connected: bool,
    /// Agreement with the combined input on the latest tick (0..1), null if idle.
    pub accuracy: Option<f32>,
    /// Average agreement over all scored time (0..1).
    pub average_accuracy: f32,
    /// Accumulated points (= seconds of perfect agreement).
    pub score: f64,
}

#[derive(Serialize)]
pub struct ScoresMsg {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub players: Vec<ScoreEntry>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Welcome {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub id: u32,
    pub name: String,
    pub color: String,
    #[serde(skip)]
    pub session: u64,
}

pub struct Game {
    players: BTreeMap<u32, Player>,
    next_id: u32,
    next_session: u64,
    tick: u64,
    dt: f64,
    last: TickSnapshot,
}

impl Game {
    pub fn new(tick_hz: f64) -> Self {
        Game {
            players: BTreeMap::new(),
            next_id: 0,
            next_session: 0,
            tick: 0,
            dt: 1.0 / tick_hz,
            last: TickSnapshot {
                kind: "tick",
                tick: 0,
                x: 0.0,
                y: 0.0,
                magnitude: 0.0,
                active_players: 0,
                connected_players: 0,
                players: vec![],
            },
        }
    }

    /// Registers a phone. A previously known, currently disconnected id is
    /// resumed (keeps its score); otherwise a new player is created.
    pub fn join(&mut self, requested: Option<u32>, name: Option<String>) -> Welcome {
        self.next_session += 1;
        let session = self.next_session;
        let resumable = requested.filter(|id| self.players.get(id).is_some_and(|p| !p.connected));
        let id = resumable.unwrap_or_else(|| {
            self.next_id += 1;
            self.next_id
        });
        let name = name
            .map(|n| n.trim().chars().take(16).collect::<String>())
            .filter(|n| !n.is_empty());

        let p = self.players.entry(id).or_insert_with(|| Player {
            name: format!("Cat {id}"),
            color: player_color(id),
            input: Vec2::ZERO,
            connected: false,
            session: 0,
            last_accuracy: None,
            score: 0.0,
            scored_seconds: 0.0,
        });
        if let Some(n) = name {
            p.name = n;
        }
        p.connected = true;
        p.session = session;
        p.input = Vec2::ZERO;

        Welcome { kind: "welcome", id, name: p.name.clone(), color: p.color.clone(), session }
    }

    pub fn set_input(&mut self, id: u32, session: u64, v: Vec2) {
        if let Some(p) = self.players.get_mut(&id).filter(|p| p.session == session && p.connected) {
            p.input = v.sanitized();
        }
    }

    pub fn leave(&mut self, id: u32, session: u64) {
        if let Some(p) = self.players.get_mut(&id).filter(|p| p.session == session) {
            p.connected = false;
            p.input = Vec2::ZERO;
            p.last_accuracy = None;
        }
    }

    /// Advances one tick: combines inputs and updates scores.
    pub fn tick(&mut self) -> &TickSnapshot {
        self.tick += 1;

        let connected: Vec<(u32, Vec2)> =
            self.players.iter().filter(|(_, p)| p.connected).map(|(id, p)| (*id, p.input)).collect();
        let active: Vec<Vec2> = connected.iter().map(|(_, v)| *v).filter(|v| v.len() > DEADZONE).collect();

        let combined = if active.is_empty() {
            Vec2::ZERO
        } else {
            let n = active.len() as f32;
            Vec2 {
                x: active.iter().map(|v| v.x).sum::<f32>() / n,
                y: active.iter().map(|v| v.y).sum::<f32>() / n,
            }
            .sanitized()
        };
        let group_moving = combined.len() > DEADZONE;

        for p in self.players.values_mut().filter(|p| p.connected) {
            if group_moving || p.input.len() > DEADZONE {
                // Max distance inside the unit circle is 2 -> map to 0..1.
                let acc = (1.0 - p.input.dist(combined) / 2.0).clamp(0.0, 1.0);
                p.last_accuracy = Some(acc);
                p.score += acc as f64 * self.dt;
                p.scored_seconds += self.dt;
            } else {
                p.last_accuracy = None;
            }
        }

        self.last = TickSnapshot {
            kind: "tick",
            tick: self.tick,
            x: combined.x,
            y: combined.y,
            magnitude: combined.len(),
            active_players: active.len(),
            connected_players: connected.len(),
            players: connected.into_iter().map(|(id, v)| PlayerInput { id, x: v.x, y: v.y }).collect(),
        };
        &self.last
    }

    pub fn last_tick(&self) -> &TickSnapshot {
        &self.last
    }

    pub fn scores(&self) -> ScoresMsg {
        let mut players: Vec<ScoreEntry> = self
            .players
            .iter()
            .map(|(id, p)| ScoreEntry {
                rank: 0,
                id: *id,
                name: p.name.clone(),
                color: p.color.clone(),
                connected: p.connected,
                accuracy: p.last_accuracy,
                average_accuracy: if p.scored_seconds > 0.0 { (p.score / p.scored_seconds) as f32 } else { 0.0 },
                score: p.score,
            })
            .collect();
        players.sort_by(|a, b| b.score.total_cmp(&a.score));
        for (i, p) in players.iter_mut().enumerate() {
            p.rank = i + 1;
        }
        ScoresMsg { kind: "scores", players }
    }

    pub fn reset_scores(&mut self) {
        for p in self.players.values_mut() {
            p.score = 0.0;
            p.scored_seconds = 0.0;
            p.last_accuracy = None;
        }
        // Forget players that are gone, they have nothing left to show.
        self.players.retain(|_, p| p.connected);
    }
}

/// Distinct, stable color per player id as `#rrggbb` (golden-angle hue spacing).
fn player_color(id: u32) -> String {
    let h = (id as f32 * 137.508) % 360.0;
    let (s, l) = (0.75_f32, 0.55_f32);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let to = |v: f32| ((v + m) * 255.0).round() as u8;
    format!("#{:02x}{:02x}{:02x}", to(r), to(g), to(b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combines_active_players_and_scores_agreement() {
        let mut g = Game::new(10.0);
        let a = g.join(None, None);
        let b = g.join(None, None);
        let c = g.join(None, None); // idle
        g.set_input(a.id, a.session, Vec2 { x: 1.0, y: 0.0 });
        g.set_input(b.id, b.session, Vec2 { x: 0.0, y: 1.0 });
        let t = g.tick().clone();
        assert!((t.x - 0.5).abs() < 1e-6 && (t.y - 0.5).abs() < 1e-6);
        assert_eq!(t.active_players, 2);
        assert_eq!(t.connected_players, 3);

        let s = g.scores();
        let by_id = |id| s.players.iter().find(|p| p.id == id).unwrap();
        assert!((by_id(a.id).accuracy.unwrap() - by_id(b.id).accuracy.unwrap()).abs() < 1e-6);
        assert!(by_id(c.id).accuracy.is_some()); // idle while the group moves is scored too
    }

    #[test]
    fn input_is_clamped_and_resume_keeps_score() {
        let mut g = Game::new(10.0);
        let a = g.join(None, None);
        g.set_input(a.id, a.session, Vec2 { x: 5.0, y: 0.0 });
        assert!((g.tick().x - 1.0).abs() < 1e-6);
        let score = g.scores().players[0].score;
        assert!(score > 0.0);

        g.leave(a.id, a.session);
        let again = g.join(Some(a.id), None);
        assert_eq!(again.id, a.id);
        assert_eq!(g.scores().players[0].score, score);

        // A stale session must not kick out the resumed one.
        g.leave(a.id, a.session);
        assert!(g.scores().players[0].connected);
    }
}
