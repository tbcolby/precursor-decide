//! Magic 8-Ball — ask a question, receive cosmic guidance.

extern crate alloc;
use alloc::string::String;

use crate::rng::Rng;

/// The 20 classic Magic 8-Ball responses, categorized.
const RESPONSES: [&str; 20] = [
    // Affirmative (10)
    "It is certain.",
    "It is decidedly so.",
    "Without a doubt.",
    "Yes, definitely.",
    "You may rely on it.",
    "As I see it, yes.",
    "Most likely.",
    "Outlook good.",
    "Yes.",
    "Signs point to yes.",
    // Non-committal (5)
    "Reply hazy, try again.",
    "Ask again later.",
    "Better not tell you now.",
    "Cannot predict now.",
    "Concentrate and ask again.",
    // Negative (5)
    "Don't count on it.",
    "My reply is no.",
    "My sources say no.",
    "Outlook not so good.",
    "Very doubtful.",
];

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ResponseType {
    Affirmative,
    NonCommittal,
    Negative,
}

pub struct EightBallResult {
    pub response: &'static str,
    pub index: usize,
    pub response_type: ResponseType,
}

pub fn shake(rng: &Rng) -> EightBallResult {
    let idx = rng.range(20) as usize;
    let response_type = match idx {
        0..=9 => ResponseType::Affirmative,
        10..=14 => ResponseType::NonCommittal,
        _ => ResponseType::Negative,
    };

    EightBallResult {
        response: RESPONSES[idx],
        index: idx,
        response_type,
    }
}

/// Format with dramatic framing.
pub fn format_result(result: &EightBallResult) -> String {
    let mut s = String::new();

    s += "  +-------------------+\n";
    s += "  |    ___   ___      |\n";
    s += "  |   /   \\_/   \\     |\n";
    s += "  |  |    8    |     |\n";
    s += "  |   \\___/ \\___/     |\n";
    s += "  +-------------------+\n\n";

    // Center the response text
    s += &result.response;
    s += "\n";

    let sentiment = match result.response_type {
        ResponseType::Affirmative => "[+]",
        ResponseType::NonCommittal => "[?]",
        ResponseType::Negative => "[-]",
    };
    s += sentiment;

    s
}
