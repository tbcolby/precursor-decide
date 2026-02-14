//! Card drawer — standard 52-card deck with optional jokers.
//!
//! Cards are represented as u8 indices:
//! - 0-12:  Clubs (A, 2-10, J, Q, K)
//! - 13-25: Diamonds
//! - 26-38: Hearts
//! - 39-51: Spades
//! - 52:    Joker (black)
//! - 53:    Joker (red)

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::rng::Rng;

const RANKS: [&str; 13] = ["A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K"];
const SUITS: [&str; 4] = ["C", "D", "H", "S"];        // clubs, diamonds, hearts, spades
const SUIT_SYMBOLS: [&str; 4] = ["&", "#", "@", "^"];  // ASCII-safe suit symbols

#[derive(Debug, Clone)]
pub struct Deck {
    pub remaining: Vec<u8>,
    pub drawn: Vec<u8>,
    pub include_jokers: bool,
}

impl Deck {
    pub fn new(rng: &Rng, include_jokers: bool) -> Self {
        let remaining = rng.shuffled_deck(include_jokers);
        Self {
            remaining,
            drawn: Vec::new(),
            include_jokers,
        }
    }

    pub fn cards_left(&self) -> usize {
        self.remaining.len()
    }

    pub fn draw(&mut self, count: usize) -> Vec<u8> {
        let n = count.min(self.remaining.len());
        let cards: Vec<u8> = self.remaining.drain(..n).collect();
        self.drawn.extend_from_slice(&cards);
        cards
    }

    pub fn reset(&mut self, rng: &Rng) {
        self.remaining = rng.shuffled_deck(self.include_jokers);
        self.drawn.clear();
    }
}

/// Get the display name of a card (e.g., "AS", "10H", "JKR").
pub fn card_name(card: u8) -> String {
    match card {
        0..=51 => {
            let suit = (card / 13) as usize;
            let rank = (card % 13) as usize;
            format!("{}{}", RANKS[rank], SUITS[suit])
        }
        52 => String::from("JKR*"),
        53 => String::from("JKR+"),
        _ => String::from("???"),
    }
}

/// Get the display name with suit symbols.
pub fn card_display(card: u8) -> String {
    match card {
        0..=51 => {
            let suit = (card / 13) as usize;
            let rank = (card % 13) as usize;
            format!("{}{}", RANKS[rank], SUIT_SYMBOLS[suit])
        }
        52 => String::from("JOKER*"),
        53 => String::from("JOKER+"),
        _ => String::from("???"),
    }
}

/// ASCII art for a single card.
pub fn card_ascii(card: u8) -> [String; 5] {
    match card {
        0..=51 => {
            let suit = (card / 13) as usize;
            let rank = (card % 13) as usize;
            let r = RANKS[rank];
            let s = SUIT_SYMBOLS[suit];

            // Pad rank to 2 chars for alignment
            let r_left = if r.len() == 1 { format!("{} ", r) } else { r.to_string() };
            let r_right = if r.len() == 1 { format!(" {}", r) } else { r.to_string() };

            [
                String::from("+-------+"),
                format!("| {}    |", r_left),
                format!("|   {}   |", s),
                format!("|    {} |", r_right),
                String::from("+-------+"),
            ]
        }
        52 | 53 => {
            let marker = if card == 52 { "*" } else { "+" };
            [
                String::from("+-------+"),
                format!("| JKR{} |", marker),
                String::from("|  ***  |"),
                format!("| {}JKR |", marker),
                String::from("+-------+"),
            ]
        }
        _ => [
            String::from("+-------+"),
            String::from("|       |"),
            String::from("|   ?   |"),
            String::from("|       |"),
            String::from("+-------+"),
        ],
    }
}

pub fn format_draw(cards: &[u8], remaining: usize) -> String {
    let mut s = String::new();

    for (i, &c) in cards.iter().enumerate() {
        if i > 0 { s += "  "; }
        s += &card_display(c);
    }

    s += &format!("\n({} remaining)", remaining);
    s
}
