//! Scoreboard — multi-player score tracking with rounds.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use serde::{Serialize, Deserialize};

const MAX_PLAYERS: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scoreboard {
    pub players: Vec<Player>,
    pub round: u32,
    pub game_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub name: String,
    pub score: i32,
}

impl Default for Scoreboard {
    fn default() -> Self {
        Self {
            players: Vec::new(),
            round: 1,
            game_name: String::from("Game"),
        }
    }
}

impl Scoreboard {
    pub fn add_player(&mut self, name: String) -> bool {
        if self.players.len() >= MAX_PLAYERS {
            return false;
        }
        self.players.push(Player { name, score: 0 });
        true
    }

    pub fn remove_player(&mut self, index: usize) -> bool {
        if index >= self.players.len() {
            return false;
        }
        self.players.remove(index);
        true
    }

    pub fn add_score(&mut self, index: usize, amount: i32) {
        if index < self.players.len() {
            self.players[index].score += amount;
        }
    }

    pub fn next_round(&mut self) {
        self.round += 1;
    }

    pub fn reset_scores(&mut self) {
        for p in &mut self.players {
            p.score = 0;
        }
        self.round = 1;
    }

    pub fn sort_by_score(&mut self) {
        self.players.sort_by(|a, b| b.score.cmp(&a.score));
    }
}

pub fn format_scoreboard(board: &Scoreboard, cursor: usize) -> String {
    let mut s = String::new();

    s += &format!("{} — Round {}\n", board.game_name, board.round);
    s += &format!("{}\n", "-".repeat(30));

    if board.players.is_empty() {
        s += "(no players — press N to add)";
        return s;
    }

    // Find longest name for alignment
    let max_name = board.players.iter().map(|p| p.name.len()).max().unwrap_or(4);

    for (i, p) in board.players.iter().enumerate() {
        let marker = if i == cursor { ">" } else { " " };
        let padding = max_name - p.name.len();
        s += &format!("{} {}{} {:>6}\n",
            marker, p.name, " ".repeat(padding), p.score);
    }

    s
}
