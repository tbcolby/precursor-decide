//! Spinner — custom segments with optional weights.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

use crate::rng::Rng;

const MAX_SEGMENTS: usize = 12;
const MIN_SEGMENTS: usize = 2;

#[derive(Debug, Clone)]
pub struct SpinnerConfig {
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub name: String,
    pub weight: u32,
}

impl Default for SpinnerConfig {
    fn default() -> Self {
        Self {
            segments: Vec::new(),
        }
    }
}

impl SpinnerConfig {
    pub fn add_segment(&mut self, name: String) -> bool {
        if self.segments.len() >= MAX_SEGMENTS {
            return false;
        }
        self.segments.push(Segment { name, weight: 1 });
        true
    }

    pub fn remove_segment(&mut self, index: usize) -> bool {
        if self.segments.len() <= MIN_SEGMENTS || index >= self.segments.len() {
            return false;
        }
        self.segments.remove(index);
        true
    }

    pub fn can_spin(&self) -> bool {
        self.segments.len() >= MIN_SEGMENTS
    }

    pub fn set_weight(&mut self, index: usize, weight: u32) {
        if index < self.segments.len() && weight > 0 {
            self.segments[index].weight = weight;
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpinResult {
    pub selected_index: usize,
    pub selected_name: String,
    pub total_segments: usize,
}

pub fn spin(config: &SpinnerConfig, rng: &Rng) -> Option<SpinResult> {
    if !config.can_spin() {
        return None;
    }

    let weights: Vec<u32> = config.segments.iter().map(|s| s.weight).collect();
    let idx = rng.weighted_pick(&weights);

    Some(SpinResult {
        selected_index: idx,
        selected_name: config.segments[idx].name.clone(),
        total_segments: config.segments.len(),
    })
}

pub fn format_result(config: &SpinnerConfig, result: &SpinResult) -> String {
    let mut s = String::new();

    // Show all segments with arrow pointing to winner
    for (i, seg) in config.segments.iter().enumerate() {
        if i == result.selected_index {
            s += &format!(" >> {} <<\n", seg.name);
        } else {
            s += &format!("    {}\n", seg.name);
        }
    }

    s += &format!("\nResult: {}", result.selected_name);
    s
}

/// Format the spinner config for display (editing mode).
pub fn format_segments(config: &SpinnerConfig, cursor: usize) -> String {
    let mut s = String::new();

    if config.segments.is_empty() {
        s += "(no segments — press N to add)";
        return s;
    }

    for (i, seg) in config.segments.iter().enumerate() {
        let marker = if i == cursor { ">" } else { " " };
        if seg.weight != 1 {
            s += &format!("{} {}. {} (x{})\n", marker, i + 1, seg.name, seg.weight);
        } else {
            s += &format!("{} {}. {}\n", marker, i + 1, seg.name);
        }
    }

    s
}
