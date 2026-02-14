//! Tournament bracket — single elimination with TRNG seeding.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::rng::Rng;

const MAX_PLAYERS: usize = 16;

#[derive(Debug, Clone)]
pub struct Tournament {
    pub players: Vec<String>,
    pub matches: Vec<Match>,
    pub current_match: usize,
    pub rounds: usize,
    pub champion: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Match {
    pub player_a: Option<String>,  // None = BYE
    pub player_b: Option<String>,
    pub winner: Option<String>,
    pub round: usize,
}

impl Tournament {
    /// Create a new tournament with seeded players.
    pub fn new(mut names: Vec<String>, rng: &Rng) -> Option<Self> {
        if names.len() < 2 || names.len() > MAX_PLAYERS {
            return None;
        }

        // Shuffle for random seeding
        rng.shuffle(&mut names);

        // Round up to next power of 2
        let bracket_size = names.len().next_power_of_two();
        let rounds = (bracket_size as f32).log2() as usize;

        // Create first-round matches, padding with BYEs
        let mut matches = Vec::new();
        let first_round_matches = bracket_size / 2;

        for i in 0..first_round_matches {
            let a = names.get(i * 2).cloned();
            let b = names.get(i * 2 + 1).cloned();

            let mut m = Match {
                player_a: a.clone(),
                player_b: b.clone(),
                winner: None,
                round: 0,
            };

            // Auto-advance BYEs
            if a.is_some() && b.is_none() {
                m.winner = a;
            } else if a.is_none() && b.is_some() {
                m.winner = b;
            }

            matches.push(m);
        }

        // Create placeholder matches for later rounds
        let mut remaining = first_round_matches / 2;
        let mut round = 1;
        while remaining > 0 {
            for _ in 0..remaining {
                matches.push(Match {
                    player_a: None,
                    player_b: None,
                    winner: None,
                    round,
                });
            }
            remaining /= 2;
            round += 1;
        }

        // Find first unresolved match
        let current = matches.iter().position(|m| m.winner.is_none())
            .unwrap_or(0);

        let mut tourney = Self {
            players: names,
            matches,
            current_match: current,
            rounds,
            champion: None,
        };

        // Propagate any BYE winners
        tourney.propagate_winners();

        Some(tourney)
    }

    /// Advance a winner for the current match.
    pub fn advance_winner(&mut self, pick: usize) -> bool {
        if self.champion.is_some() {
            return false;
        }

        let current = self.current_match;
        if current >= self.matches.len() {
            return false;
        }

        let winner = match pick {
            1 => self.matches[current].player_a.clone(),
            2 => self.matches[current].player_b.clone(),
            _ => return false,
        };

        if let Some(w) = winner {
            self.matches[current].winner = Some(w);
            self.propagate_winners();

            // Check if tournament is over
            if let Some(last) = self.matches.last() {
                if last.winner.is_some() {
                    self.champion = last.winner.clone();
                }
            }

            // Move to next unresolved match
            self.current_match = self.matches.iter()
                .position(|m| m.winner.is_none() && m.player_a.is_some() && m.player_b.is_some())
                .unwrap_or(current);

            true
        } else {
            false
        }
    }

    /// Randomly decide the current match.
    pub fn random_result(&mut self, rng: &Rng) -> bool {
        let pick = if rng.flip() { 1 } else { 2 };
        self.advance_winner(pick)
    }

    /// Propagate winners into the next round's match slots.
    fn propagate_winners(&mut self) {
        // For each round, feed winners into the next round
        let first_round_size = self.matches.iter().filter(|m| m.round == 0).count();
        let mut offset = 0;
        let mut round_size = first_round_size;

        while round_size >= 2 {
            let next_offset = offset + round_size;
            let next_round_size = round_size / 2;

            for i in 0..next_round_size {
                let match_a = offset + i * 2;
                let match_b = offset + i * 2 + 1;
                let dest = next_offset + i;

                if dest < self.matches.len() {
                    if let Some(ref w) = self.matches[match_a].winner {
                        self.matches[dest].player_a = Some(w.clone());
                    }
                    if let Some(ref w) = self.matches[match_b].winner {
                        self.matches[dest].player_b = Some(w.clone());
                    }

                    // Auto-advance if only one player (BYE in later rounds)
                    let m = &self.matches[dest];
                    if m.winner.is_none() {
                        if m.player_a.is_some() && m.player_b.is_none() {
                            self.matches[dest].winner = self.matches[dest].player_a.clone();
                        } else if m.player_a.is_none() && m.player_b.is_some() {
                            self.matches[dest].winner = self.matches[dest].player_b.clone();
                        }
                    }
                }
            }

            offset = next_offset;
            round_size = next_round_size;
        }
    }
}

pub fn format_bracket(tourney: &Tournament) -> String {
    let mut s = String::new();

    if let Some(ref champ) = tourney.champion {
        s += &format!("CHAMPION: {}\n\n", champ);
    }

    let mut round = 0;
    let mut match_num = 0;

    for (i, m) in tourney.matches.iter().enumerate() {
        if m.round != round {
            round = m.round;
            match_num = 0;
            s += &format!("\n--- Round {} ---\n", round + 1);
        }

        match_num += 1;
        let marker = if i == tourney.current_match && tourney.champion.is_none() {
            ">>"
        } else {
            "  "
        };

        let a = m.player_a.as_deref().unwrap_or("...");
        let b = m.player_b.as_deref().unwrap_or("...");
        let w = m.winner.as_deref().unwrap_or("");

        if !w.is_empty() {
            s += &format!("{} M{}: {} vs {} -> {}\n", marker, match_num, a, b, w);
        } else {
            s += &format!("{} M{}: {} vs {}\n", marker, match_num, a, b);
        }
    }

    s
}
