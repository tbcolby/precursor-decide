//! True random number generation via Precursor's hardware TRNG.
//!
//! Every random number in this app comes from quantum noise —
//! not algorithms, not seeds, not PRNGs. The universe decides.

extern crate alloc;
use alloc::vec::Vec;

/// Wrapper around the hardware TRNG for convenient random operations.
pub struct Rng {
    trng: trng::Trng,
}

impl Rng {
    pub fn new(xns: &xous_names::XousNames) -> Self {
        Self {
            trng: trng::Trng::new(xns).expect("can't connect to TRNG"),
        }
    }

    /// Random u32 from hardware TRNG.
    pub fn u32(&self) -> u32 {
        self.trng.get_u32().unwrap_or(0)
    }

    /// Random number in range [0, max) using rejection sampling for uniformity.
    pub fn range(&self, max: u32) -> u32 {
        if max <= 1 {
            return 0;
        }
        // Rejection sampling: find the largest multiple of max that fits in u32,
        // reject values above it to eliminate modulo bias.
        let threshold = u32::MAX - (u32::MAX % max);
        loop {
            let val = self.u32();
            if val < threshold {
                return val % max;
            }
        }
    }

    /// Random number in range [min, max] inclusive.
    pub fn range_inclusive(&self, min: u32, max: u32) -> u32 {
        if min >= max {
            return min;
        }
        min + self.range(max - min + 1)
    }

    /// Roll a die with N sides. Returns 1..=N.
    pub fn roll_die(&self, sides: u32) -> u32 {
        self.range_inclusive(1, sides)
    }

    /// Coin flip: true = heads, false = tails.
    pub fn flip(&self) -> bool {
        self.u32() & 1 != 0
    }

    /// Fisher-Yates shuffle using TRNG.
    pub fn shuffle<T>(&self, items: &mut [T]) {
        let n = items.len();
        for i in (1..n).rev() {
            let j = self.range((i + 1) as u32) as usize;
            items.swap(i, j);
        }
    }

    /// Pick one item from a slice uniformly at random.
    pub fn pick<'a, T>(&self, items: &'a [T]) -> &'a T {
        let idx = self.range(items.len() as u32) as usize;
        &items[idx]
    }

    /// Pick one index from a weighted distribution.
    /// Weights don't need to sum to anything specific.
    pub fn weighted_pick(&self, weights: &[u32]) -> usize {
        let total: u32 = weights.iter().sum();
        if total == 0 {
            return 0;
        }
        let mut roll = self.range(total);
        for (i, &w) in weights.iter().enumerate() {
            if roll < w {
                return i;
            }
            roll -= w;
        }
        weights.len() - 1
    }

    /// Generate a shuffled deck of cards (0-53, where 52-53 are jokers).
    pub fn shuffled_deck(&self, include_jokers: bool) -> Vec<u8> {
        let size = if include_jokers { 54 } else { 52 };
        let mut deck: Vec<u8> = (0..size).collect();
        self.shuffle(&mut deck);
        deck
    }
}
