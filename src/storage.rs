//! PDDB storage for Decision Engine.
//!
//! Dictionary: decide.state
//! Keys: config, scoreboard, spinner, deck, roll_history, custom_coin

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::scoreboard::Scoreboard;
use crate::spinner::SpinnerConfig;

const DICT: &str = "decide.state";
const KEY_CONFIG: &str = "config";
const KEY_SCOREBOARD: &str = "scoreboard";
const KEY_SPINNER: &str = "spinner_segments";
const KEY_COIN: &str = "custom_coin";

pub struct Storage {
    pddb: pddb::Pddb,
}

impl Storage {
    pub fn new() -> Result<Self, ()> {
        let pddb = pddb::Pddb::new();
        pddb.is_mounted_blocking();
        Ok(Self { pddb })
    }

    fn read_key(&mut self, key: &str) -> Option<Vec<u8>> {
        let mut handle = self
            .pddb
            .get(DICT, key, None, false, false, None, None::<fn()>)
            .ok()?;
        let mut buf = Vec::new();
        use std::io::Read;
        handle.read_to_end(&mut buf).ok()?;
        Some(buf)
    }

    fn write_key(&mut self, key: &str, data: &[u8]) {
        if let Ok(mut handle) = self.pddb.get(
            DICT,
            key,
            None,
            true,
            true,
            Some(data.len()),
            None::<fn()>,
        ) {
            use std::io::{Seek, Write};
            handle.seek(std::io::SeekFrom::Start(0)).ok();
            handle.write_all(data).ok();
            handle.set_len(data.len() as u64).ok();
        }
        self.pddb.sync().ok();
    }

    // ── Config ──

    pub fn load_last_tool(&mut self) -> Option<String> {
        let buf = self.read_key(KEY_CONFIG)?;
        let json: serde_json::Value = serde_json::from_slice(&buf).ok()?;
        json.get("last_tool").and_then(|v| v.as_str()).map(String::from)
    }

    pub fn save_last_tool(&mut self, tool: &str) {
        let json = serde_json::json!({ "last_tool": tool });
        let data = serde_json::to_vec(&json).unwrap_or_default();
        self.write_key(KEY_CONFIG, &data);
    }

    // ── Scoreboard ──

    pub fn load_scoreboard(&mut self) -> Option<Scoreboard> {
        let buf = self.read_key(KEY_SCOREBOARD)?;
        serde_json::from_slice(&buf).ok()
    }

    pub fn save_scoreboard(&mut self, board: &Scoreboard) {
        let data = serde_json::to_vec(board).unwrap_or_default();
        self.write_key(KEY_SCOREBOARD, &data);
    }

    // ── Spinner segments ──

    pub fn load_spinner_segments(&mut self) -> Option<Vec<String>> {
        let buf = self.read_key(KEY_SPINNER)?;
        serde_json::from_slice(&buf).ok()
    }

    pub fn save_spinner_segments(&mut self, names: &[String]) {
        let data = serde_json::to_vec(names).unwrap_or_default();
        self.write_key(KEY_SPINNER, &data);
    }

    // ── Custom coin ──

    pub fn load_custom_coin(&mut self) -> Option<(String, String)> {
        let buf = self.read_key(KEY_COIN)?;
        let json: serde_json::Value = serde_json::from_slice(&buf).ok()?;
        let a = json.get("side_a").and_then(|v| v.as_str())?;
        let b = json.get("side_b").and_then(|v| v.as_str())?;
        Some((String::from(a), String::from(b)))
    }

    pub fn save_custom_coin(&mut self, side_a: &str, side_b: &str) {
        let json = serde_json::json!({ "side_a": side_a, "side_b": side_b });
        let data = serde_json::to_vec(&json).unwrap_or_default();
        self.write_key(KEY_COIN, &data);
    }
}
