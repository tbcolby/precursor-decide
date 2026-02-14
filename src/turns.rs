//! Turn tracker — manage player order with round counting.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::rng::Rng;

const MAX_PLAYERS: usize = 20;

#[derive(Debug, Clone)]
pub struct TurnTracker {
    pub players: Vec<String>,
    pub current: usize,
    pub round: u32,
    pub skipped: Vec<bool>,  // players temporarily skipped
}

impl Default for TurnTracker {
    fn default() -> Self {
        Self {
            players: Vec::new(),
            current: 0,
            round: 1,
            skipped: Vec::new(),
        }
    }
}

impl TurnTracker {
    pub fn add_player(&mut self, name: String) -> bool {
        if self.players.len() >= MAX_PLAYERS {
            return false;
        }
        self.players.push(name);
        self.skipped.push(false);
        true
    }

    pub fn remove_player(&mut self, index: usize) -> bool {
        if index >= self.players.len() || self.players.len() <= 1 {
            return false;
        }
        self.players.remove(index);
        self.skipped.remove(index);
        if self.current >= self.players.len() {
            self.current = 0;
        }
        true
    }

    pub fn next_turn(&mut self) {
        if self.players.is_empty() {
            return;
        }

        let start = self.current;
        loop {
            self.current = (self.current + 1) % self.players.len();

            // New round when we wrap
            if self.current == 0 {
                self.round += 1;
                // Clear all skips at round boundary
                for s in &mut self.skipped {
                    *s = false;
                }
            }

            // If this player isn't skipped, they're up
            if !self.skipped[self.current] {
                break;
            }

            // Safety: if we looped all the way around, just stop
            if self.current == start {
                break;
            }
        }
    }

    pub fn skip_current(&mut self) {
        if !self.players.is_empty() {
            self.skipped[self.current] = true;
            self.next_turn();
        }
    }

    pub fn randomize_order(&mut self, rng: &Rng) {
        rng.shuffle(&mut self.players);
        self.current = 0;
        self.round = 1;
        for s in &mut self.skipped {
            *s = false;
        }
    }

    pub fn current_player(&self) -> Option<&str> {
        if self.players.is_empty() {
            None
        } else {
            Some(&self.players[self.current])
        }
    }
}

pub fn format_tracker(tracker: &TurnTracker) -> String {
    let mut s = String::new();

    s += &format!("Round {}\n", tracker.round);
    s += &format!("{}\n", "-".repeat(25));

    if tracker.players.is_empty() {
        s += "(no players — press N to add)";
        return s;
    }

    for (i, name) in tracker.players.iter().enumerate() {
        let marker = if i == tracker.current { ">>" } else { "  " };
        let skip_mark = if tracker.skipped[i] { " [SKIP]" } else { "" };
        s += &format!("{} {}{}\n", marker, name, skip_mark);
    }

    s
}
