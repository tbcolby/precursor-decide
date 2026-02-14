//! Coin flipper — single, multi-flip, custom side names.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::rng::Rng;

#[derive(Debug, Clone)]
pub struct CoinConfig {
    pub count: u32,           // 1-20 coins
    pub side_a: String,       // default "HEADS"
    pub side_b: String,       // default "TAILS"
}

impl Default for CoinConfig {
    fn default() -> Self {
        Self {
            count: 1,
            side_a: String::from("HEADS"),
            side_b: String::from("TAILS"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CoinResult {
    pub flips: Vec<bool>,     // true = side_a, false = side_b
    pub side_a_count: u32,
    pub side_b_count: u32,
    pub side_a_name: String,
    pub side_b_name: String,
}

pub fn flip(config: &CoinConfig, rng: &Rng) -> CoinResult {
    let mut flips = Vec::new();
    let mut a_count = 0u32;
    let mut b_count = 0u32;

    for _ in 0..config.count {
        let is_a = rng.flip();
        flips.push(is_a);
        if is_a {
            a_count += 1;
        } else {
            b_count += 1;
        }
    }

    CoinResult {
        flips,
        side_a_count: a_count,
        side_b_count: b_count,
        side_a_name: config.side_a.clone(),
        side_b_name: config.side_b.clone(),
    }
}

/// Big ASCII art for a single coin flip result.
pub fn coin_ascii(is_side_a: bool, side_a: &str, side_b: &str) -> [String; 5] {
    let label = if is_side_a { side_a } else { side_b };
    // Center the label in a 9-char field
    let padded = if label.len() >= 9 {
        label[..9].into()
    } else {
        let left = (9 - label.len()) / 2;
        let right = 9 - label.len() - left;
        format!("{}{}{}", &"         "[..left], label, &"         "[..right])
    };

    [
        String::from("+===========+"),
        String::from("|           |"),
        format!("| {} |", padded),
        String::from("|           |"),
        String::from("+===========+"),
    ]
}

pub fn format_result(result: &CoinResult) -> String {
    let mut s = String::new();

    if result.flips.len() == 1 {
        // Single flip — just show the result
        let side = if result.flips[0] {
            &result.side_a_name
        } else {
            &result.side_b_name
        };
        s += side;
    } else {
        // Multi-flip — show counts
        s += &format!("{}: {}  |  {}: {}",
            result.side_a_name, result.side_a_count,
            result.side_b_name, result.side_b_count);

        // Show individual results if <= 10
        if result.flips.len() <= 10 {
            s += "\n[";
            for (i, &is_a) in result.flips.iter().enumerate() {
                if i > 0 { s += ", "; }
                if is_a {
                    s += &result.side_a_name[..1];
                } else {
                    s += &result.side_b_name[..1];
                }
            }
            s += "]";
        }
    }

    s
}
