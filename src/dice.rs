//! Dice roller — the flagship tool.
//!
//! Standard RPG dice (d4-d100), multiple dice, modifiers,
//! advantage/disadvantage, exploding, drop lowest.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::rng::Rng;

const DIE_TYPES: [u32; 7] = [4, 6, 8, 10, 12, 20, 100];

#[derive(Debug, Clone)]
pub struct DiceConfig {
    pub count: u32,          // 1-20 dice
    pub sides: u32,          // d4, d6, d8, d10, d12, d20, d100, or custom
    pub modifier: i32,       // +N or -N
    pub advantage: bool,     // roll twice, take higher
    pub disadvantage: bool,  // roll twice, take lower
    pub exploding: bool,     // if max, roll again and add
    pub drop_lowest: u32,    // drop N lowest dice
    pub sides_index: usize,  // index into DIE_TYPES
}

impl Default for DiceConfig {
    fn default() -> Self {
        Self {
            count: 1,
            sides: 20,
            modifier: 0,
            advantage: false,
            disadvantage: false,
            exploding: false,
            drop_lowest: 0,
            sides_index: 5, // d20
        }
    }
}

impl DiceConfig {
    pub fn cycle_die_type(&mut self) {
        self.sides_index = (self.sides_index + 1) % DIE_TYPES.len();
        self.sides = DIE_TYPES[self.sides_index];
    }

    pub fn notation(&self) -> String {
        let mut s = format!("{}d{}", self.count, self.sides);
        if self.drop_lowest > 0 {
            s += &format!(" drop {}", self.drop_lowest);
        }
        if self.modifier > 0 {
            s += &format!("+{}", self.modifier);
        } else if self.modifier < 0 {
            s += &format!("{}", self.modifier);
        }
        if self.advantage {
            s += " ADV";
        }
        if self.disadvantage {
            s += " DIS";
        }
        if self.exploding {
            s += " EXPLODE";
        }
        s
    }
}

#[derive(Debug, Clone)]
pub struct DiceResult {
    pub notation: String,
    pub individual: Vec<u32>,
    pub dropped: Vec<u32>,
    pub subtotal: u32,
    pub modifier: i32,
    pub total: i32,
    pub was_advantage: Option<(i32, i32)>, // (roll1_total, roll2_total)
    pub is_crit: bool,    // natural 20 on d20
    pub is_fumble: bool,  // natural 1 on d20
}

pub fn roll(config: &DiceConfig, rng: &Rng) -> DiceResult {
    if config.advantage || config.disadvantage {
        // Roll twice, take higher/lower
        let r1 = roll_once(config, rng);
        let r2 = roll_once(config, rng);
        let t1 = r1.total;
        let t2 = r2.total;

        let chosen = if config.advantage {
            if t1 >= t2 { r1 } else { r2 }
        } else {
            if t1 <= t2 { r1 } else { r2 }
        };

        DiceResult {
            was_advantage: Some((t1, t2)),
            ..chosen
        }
    } else {
        roll_once(config, rng)
    }
}

fn roll_once(config: &DiceConfig, rng: &Rng) -> DiceResult {
    let mut rolls: Vec<u32> = Vec::new();

    for _ in 0..config.count {
        let mut val = rng.roll_die(config.sides);

        if config.exploding {
            let mut total = val;
            while val == config.sides {
                val = rng.roll_die(config.sides);
                total += val;
            }
            rolls.push(total);
        } else {
            rolls.push(val);
        }
    }

    // Sort for drop-lowest
    let mut sorted = rolls.clone();
    sorted.sort();

    let drop_count = (config.drop_lowest as usize).min(sorted.len().saturating_sub(1));
    let dropped: Vec<u32> = sorted[..drop_count].to_vec();
    let kept: Vec<u32> = sorted[drop_count..].to_vec();

    let subtotal: u32 = kept.iter().sum();
    let total = subtotal as i32 + config.modifier;

    // Check for crits (d20 only, single die)
    let is_crit = config.sides == 20 && config.count == 1 && rolls[0] == 20;
    let is_fumble = config.sides == 20 && config.count == 1 && rolls[0] == 1;

    DiceResult {
        notation: config.notation(),
        individual: rolls,
        dropped,
        subtotal,
        modifier: config.modifier,
        total,
        was_advantage: None,
        is_crit,
        is_fumble,
    }
}

/// ASCII art for a d6 face.
pub fn d6_ascii(value: u32) -> [&'static str; 5] {
    match value {
        1 => [
            "+-------+",
            "|       |",
            "|   o   |",
            "|       |",
            "+-------+",
        ],
        2 => [
            "+-------+",
            "| o     |",
            "|       |",
            "|     o |",
            "+-------+",
        ],
        3 => [
            "+-------+",
            "| o     |",
            "|   o   |",
            "|     o |",
            "+-------+",
        ],
        4 => [
            "+-------+",
            "| o   o |",
            "|       |",
            "| o   o |",
            "+-------+",
        ],
        5 => [
            "+-------+",
            "| o   o |",
            "|   o   |",
            "| o   o |",
            "+-------+",
        ],
        6 => [
            "+-------+",
            "| o   o |",
            "| o   o |",
            "| o   o |",
            "+-------+",
        ],
        _ => [
            "+-------+",
            "|       |",
            "|   ?   |",
            "|       |",
            "+-------+",
        ],
    }
}

/// Format a roll result for display.
pub fn format_result(result: &DiceResult) -> String {
    let mut s = String::new();

    // Individual dice
    if result.individual.len() <= 10 {
        s += "[";
        for (i, &v) in result.individual.iter().enumerate() {
            if i > 0 { s += ", "; }
            s += &format!("{}", v);
        }
        s += "]";
    } else {
        s += &format!("[{} dice rolled]", result.individual.len());
    }

    if !result.dropped.is_empty() {
        s += &format!(" drop {:?}", result.dropped);
    }

    s += &format!(" = {}", result.subtotal);

    if result.modifier != 0 {
        if result.modifier > 0 {
            s += &format!(" + {}", result.modifier);
        } else {
            s += &format!(" - {}", result.modifier.abs());
        }
        s += &format!(" = {}", result.total);
    }

    if let Some((r1, r2)) = result.was_advantage {
        s += &format!("\n(rolls: {} vs {})", r1, r2);
    }

    if result.is_crit {
        s += "\n*** CRITICAL HIT! ***";
    } else if result.is_fumble {
        s += "\n--- fumble ---";
    }

    s
}
