//! Application state machine — coordinates all 8 decision tools.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::cards::{self, Deck};
use crate::coin::{self, CoinConfig, CoinResult};
use crate::dice::{self, DiceConfig, DiceResult};
use crate::eightball::{self, EightBallResult};
use crate::rng::Rng;
use crate::scoreboard::Scoreboard;
use crate::spinner::{self, SpinnerConfig, SpinResult};
use crate::storage::Storage;
use crate::tournament::{self, Tournament};
use crate::turns::TurnTracker;

// Standard key codes (ecosystem standard)
const KEY_UP: char = '\u{2191}';
const KEY_DOWN: char = '\u{2193}';
const KEY_LEFT: char = '\u{2190}';
const KEY_RIGHT: char = '\u{2192}';
const KEY_ENTER: char = '\r';
const KEY_BACKSPACE: char = '\u{0008}';
const KEY_MENU: char = '\u{2234}';

/// Top-level app states.
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    HomeMenu,
    DiceRoller,
    CoinFlip,
    CardDrawer,
    EightBall,
    Spinner,
    ScoreboardView,
    TurnTrackerView,
    TournamentView,
    // Text input sub-states
    TextInput(TextInputContext),
}

/// Context for text-input mode.
#[derive(Debug, Clone, PartialEq)]
pub struct TextInputContext {
    pub purpose: InputPurpose,
    pub buffer: String,
    pub max_len: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputPurpose {
    CoinSideA,
    CoinSideB,
    SpinnerSegment,
    ScoreboardPlayer,
    TurnPlayer,
    TournamentPlayer,
}

/// Home menu items.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tool {
    Dice,
    Coin,
    Cards,
    EightBall,
    Spinner,
    Scoreboard,
    TurnTracker,
    Tournament,
}

impl Tool {
    pub fn label(&self) -> &'static str {
        match self {
            Tool::Dice => "Dice Roller",
            Tool::Coin => "Coin Flip",
            Tool::Cards => "Card Drawer",
            Tool::EightBall => "Magic 8-Ball",
            Tool::Spinner => "Spinner",
            Tool::Scoreboard => "Scoreboard",
            Tool::TurnTracker => "Turn Tracker",
            Tool::Tournament => "Tournament",
        }
    }

    pub fn all() -> &'static [Tool] {
        &[
            Tool::Dice,
            Tool::Coin,
            Tool::Cards,
            Tool::EightBall,
            Tool::Spinner,
            Tool::Scoreboard,
            Tool::TurnTracker,
            Tool::Tournament,
        ]
    }

    pub fn key(&self) -> &'static str {
        match self {
            Tool::Dice => "dice",
            Tool::Coin => "coin",
            Tool::Cards => "cards",
            Tool::EightBall => "8ball",
            Tool::Spinner => "spinner",
            Tool::Scoreboard => "score",
            Tool::TurnTracker => "turns",
            Tool::Tournament => "tourney",
        }
    }
}

/// Master application state.
pub struct DecideApp {
    pub state: AppState,
    pub needs_redraw: bool,

    // Home menu
    pub menu_cursor: usize,

    // Dice
    pub dice_config: DiceConfig,
    pub dice_result: Option<DiceResult>,
    pub dice_history: Vec<String>, // last 20 roll summaries

    // Coin
    pub coin_config: CoinConfig,
    pub coin_result: Option<CoinResult>,
    pub coin_streak: u32,
    pub coin_last_side: Option<bool>,

    // Cards
    pub deck: Option<Deck>,
    pub drawn_cards: Vec<u8>,
    pub draw_count: usize,

    // 8-Ball
    pub eightball_result: Option<EightBallResult>,

    // Spinner
    pub spinner_config: SpinnerConfig,
    pub spin_result: Option<SpinResult>,
    pub spinner_cursor: usize,

    // Scoreboard
    pub scoreboard: Scoreboard,
    pub score_cursor: usize,
    pub score_increment: i32,

    // Turn tracker
    pub turn_tracker: TurnTracker,

    // Tournament
    pub tournament: Option<Tournament>,
    pub tourney_names: Vec<String>,

    // Storage
    pub storage: Option<Storage>,
}

impl DecideApp {
    pub fn new() -> Self {
        Self {
            state: AppState::HomeMenu,
            needs_redraw: true,
            menu_cursor: 0,
            dice_config: DiceConfig::default(),
            dice_result: None,
            dice_history: Vec::new(),
            coin_config: CoinConfig::default(),
            coin_result: None,
            coin_streak: 0,
            coin_last_side: None,
            deck: None,
            drawn_cards: Vec::new(),
            draw_count: 1,
            eightball_result: None,
            spinner_config: SpinnerConfig::default(),
            spin_result: None,
            spinner_cursor: 0,
            scoreboard: Scoreboard::default(),
            score_cursor: 0,
            score_increment: 1,
            turn_tracker: TurnTracker::default(),
            tournament: None,
            tourney_names: Vec::new(),
            storage: None,
        }
    }

    pub fn init_storage(&mut self) {
        if let Ok(mut stor) = Storage::new() {
            // Load persisted state
            if let Some(board) = stor.load_scoreboard() {
                self.scoreboard = board;
            }
            if let Some(segments) = stor.load_spinner_segments() {
                for name in segments {
                    self.spinner_config.add_segment(name);
                }
            }
            if let Some((a, b)) = stor.load_custom_coin() {
                self.coin_config.side_a = a;
                self.coin_config.side_b = b;
            }
            self.storage = Some(stor);
        }
    }

    pub fn save_state(&mut self) {
        if let Some(ref mut stor) = self.storage {
            let tool_key = match self.state {
                AppState::DiceRoller => "dice",
                AppState::CoinFlip => "coin",
                AppState::CardDrawer => "cards",
                AppState::EightBall => "8ball",
                AppState::Spinner => "spinner",
                AppState::ScoreboardView => "score",
                AppState::TurnTrackerView => "turns",
                AppState::TournamentView => "tourney",
                _ => "dice",
            };
            stor.save_last_tool(tool_key);
            stor.save_scoreboard(&self.scoreboard);

            let segments: Vec<String> = self.spinner_config.segments
                .iter().map(|s| s.name.clone()).collect();
            stor.save_spinner_segments(&segments);

            stor.save_custom_coin(&self.coin_config.side_a, &self.coin_config.side_b);
        }
    }

    /// Handle a key press. Returns false to quit.
    pub fn handle_key(&mut self, key: char, rng: &Rng) -> bool {
        self.needs_redraw = true;

        match &self.state.clone() {
            AppState::HomeMenu => self.handle_home_key(key),
            AppState::DiceRoller => self.handle_dice_key(key, rng),
            AppState::CoinFlip => self.handle_coin_key(key, rng),
            AppState::CardDrawer => self.handle_cards_key(key, rng),
            AppState::EightBall => self.handle_eightball_key(key, rng),
            AppState::Spinner => self.handle_spinner_key(key, rng),
            AppState::ScoreboardView => self.handle_scoreboard_key(key),
            AppState::TurnTrackerView => self.handle_turns_key(key, rng),
            AppState::TournamentView => self.handle_tournament_key(key, rng),
            AppState::TextInput(ctx) => {
                let purpose = ctx.purpose.clone();
                let max_len = ctx.max_len;
                self.handle_text_input(key, purpose, max_len)
            }
        }
    }

    // ─── Home Menu ──────────────────────────────────────────────────────────

    fn handle_home_key(&mut self, key: char) -> bool {
        match key {
            KEY_UP => {
                if self.menu_cursor > 0 {
                    self.menu_cursor -= 1;
                }
            }
            KEY_DOWN => {
                let max = Tool::all().len() - 1;
                if self.menu_cursor < max {
                    self.menu_cursor += 1;
                }
            }
            KEY_ENTER | ' ' => {
                self.enter_tool(self.menu_cursor);
            }
            '1'..='8' => {
                let idx = (key as usize) - ('1' as usize);
                if idx < Tool::all().len() {
                    self.enter_tool(idx);
                }
            }
            'q' | 'Q' | KEY_MENU => return false,
            _ => {}
        }
        true
    }

    fn enter_tool(&mut self, index: usize) {
        let tools = Tool::all();
        if index >= tools.len() {
            return;
        }
        match tools[index] {
            Tool::Dice => self.state = AppState::DiceRoller,
            Tool::Coin => self.state = AppState::CoinFlip,
            Tool::Cards => self.state = AppState::CardDrawer,
            Tool::EightBall => self.state = AppState::EightBall,
            Tool::Spinner => self.state = AppState::Spinner,
            Tool::Scoreboard => self.state = AppState::ScoreboardView,
            Tool::TurnTracker => self.state = AppState::TurnTrackerView,
            Tool::Tournament => self.state = AppState::TournamentView,
        }
    }

    fn go_home(&mut self) {
        self.save_state();
        self.state = AppState::HomeMenu;
    }

    // ─── Dice Roller ────────────────────────────────────────────────────────

    fn handle_dice_key(&mut self, key: char, rng: &Rng) -> bool {
        match key {
            KEY_ENTER | ' ' => {
                let result = dice::roll(&self.dice_config, rng);
                let summary = alloc::format!("{}: {}",
                    result.notation, dice::format_result(&result));
                self.dice_history.insert(0, summary);
                if self.dice_history.len() > 20 {
                    self.dice_history.truncate(20);
                }
                self.dice_result = Some(result);
            }
            'd' | 'D' => {
                self.dice_config.cycle_die_type();
            }
            '1'..='9' => {
                self.dice_config.count = (key as u32) - ('0' as u32);
            }
            '0' => {
                // 0 after another digit means 10+
                if self.dice_config.count >= 1 && self.dice_config.count <= 2 {
                    self.dice_config.count *= 10;
                    if self.dice_config.count > 20 {
                        self.dice_config.count = 20;
                    }
                }
            }
            '+' | '=' => {
                if self.dice_config.modifier < 99 {
                    self.dice_config.modifier += 1;
                }
            }
            '-' | '_' => {
                if self.dice_config.modifier > -99 {
                    self.dice_config.modifier -= 1;
                }
            }
            'a' | 'A' => {
                self.dice_config.advantage = !self.dice_config.advantage;
                if self.dice_config.advantage {
                    self.dice_config.disadvantage = false;
                }
            }
            'v' | 'V' => {
                self.dice_config.disadvantage = !self.dice_config.disadvantage;
                if self.dice_config.disadvantage {
                    self.dice_config.advantage = false;
                }
            }
            'x' | 'X' => {
                self.dice_config.exploding = !self.dice_config.exploding;
            }
            'l' | 'L' => {
                if self.dice_config.drop_lowest > 0 {
                    self.dice_config.drop_lowest = 0;
                } else if self.dice_config.count > 1 {
                    self.dice_config.drop_lowest = 1;
                }
            }
            'q' | 'Q' | KEY_MENU | KEY_BACKSPACE => self.go_home(),
            _ => {}
        }
        true
    }

    // ─── Coin Flip ──────────────────────────────────────────────────────────

    fn handle_coin_key(&mut self, key: char, rng: &Rng) -> bool {
        match key {
            KEY_ENTER | ' ' => {
                let result = coin::flip(&self.coin_config, rng);

                // Track streak for single flips
                if result.flips.len() == 1 {
                    let side = result.flips[0];
                    if self.coin_last_side == Some(side) {
                        self.coin_streak += 1;
                    } else {
                        self.coin_streak = 1;
                        self.coin_last_side = Some(side);
                    }
                }

                self.coin_result = Some(result);
            }
            'n' | 'N' => {
                self.coin_config.count += 1;
                if self.coin_config.count > 20 {
                    self.coin_config.count = 1;
                }
            }
            'c' | 'C' => {
                // Enter text input for custom side A name
                self.state = AppState::TextInput(TextInputContext {
                    purpose: InputPurpose::CoinSideA,
                    buffer: String::new(),
                    max_len: 10,
                });
            }
            'q' | 'Q' | KEY_MENU | KEY_BACKSPACE => self.go_home(),
            _ => {}
        }
        true
    }

    // ─── Card Drawer ────────────────────────────────────────────────────────

    fn handle_cards_key(&mut self, key: char, rng: &Rng) -> bool {
        match key {
            KEY_ENTER | ' ' => {
                // Initialize deck if needed
                if self.deck.is_none() {
                    self.deck = Some(Deck::new(rng, false));
                }
                if let Some(ref mut deck) = self.deck {
                    self.drawn_cards = deck.draw(self.draw_count);
                }
            }
            '1'..='9' => {
                self.draw_count = (key as usize) - ('0' as usize);
            }
            'r' | 'R' => {
                self.deck = Some(Deck::new(rng, false));
                self.drawn_cards.clear();
            }
            'j' | 'J' => {
                // Toggle jokers by resetting deck with/without
                let include = self.deck.as_ref().map(|d| !d.include_jokers).unwrap_or(true);
                self.deck = Some(Deck::new(rng, include));
                self.drawn_cards.clear();
            }
            'q' | 'Q' | KEY_MENU | KEY_BACKSPACE => self.go_home(),
            _ => {}
        }
        true
    }

    // ─── Magic 8-Ball ───────────────────────────────────────────────────────

    fn handle_eightball_key(&mut self, key: char, rng: &Rng) -> bool {
        match key {
            KEY_ENTER | ' ' => {
                self.eightball_result = Some(eightball::shake(rng));
            }
            'q' | 'Q' | KEY_MENU | KEY_BACKSPACE => self.go_home(),
            _ => {}
        }
        true
    }

    // ─── Spinner ────────────────────────────────────────────────────────────

    fn handle_spinner_key(&mut self, key: char, rng: &Rng) -> bool {
        match key {
            KEY_ENTER | ' ' => {
                if let Some(result) = spinner::spin(&self.spinner_config, rng) {
                    self.spin_result = Some(result);
                }
            }
            KEY_UP => {
                if self.spinner_cursor > 0 {
                    self.spinner_cursor -= 1;
                }
            }
            KEY_DOWN => {
                let max = self.spinner_config.segments.len().saturating_sub(1);
                if self.spinner_cursor < max {
                    self.spinner_cursor += 1;
                }
            }
            'n' | 'N' => {
                // Add segment via text input
                self.state = AppState::TextInput(TextInputContext {
                    purpose: InputPurpose::SpinnerSegment,
                    buffer: String::new(),
                    max_len: 20,
                });
            }
            'd' | 'D' => {
                self.spinner_config.remove_segment(self.spinner_cursor);
                if self.spinner_cursor > 0
                    && self.spinner_cursor >= self.spinner_config.segments.len()
                {
                    self.spinner_cursor -= 1;
                }
            }
            '+' | '=' => {
                self.spinner_config.set_weight(self.spinner_cursor,
                    self.spinner_config.segments.get(self.spinner_cursor)
                        .map(|s| s.weight + 1).unwrap_or(1));
            }
            '-' | '_' => {
                let current = self.spinner_config.segments.get(self.spinner_cursor)
                    .map(|s| s.weight).unwrap_or(1);
                if current > 1 {
                    self.spinner_config.set_weight(self.spinner_cursor, current - 1);
                }
            }
            'q' | 'Q' | KEY_MENU | KEY_BACKSPACE => self.go_home(),
            _ => {}
        }
        true
    }

    // ─── Scoreboard ─────────────────────────────────────────────────────────

    fn handle_scoreboard_key(&mut self, key: char) -> bool {
        match key {
            KEY_UP => {
                if self.score_cursor > 0 {
                    self.score_cursor -= 1;
                }
            }
            KEY_DOWN => {
                let max = self.scoreboard.players.len().saturating_sub(1);
                if self.score_cursor < max {
                    self.score_cursor += 1;
                }
            }
            KEY_RIGHT | '+' | '=' => {
                self.scoreboard.add_score(self.score_cursor, self.score_increment);
            }
            KEY_LEFT | '-' | '_' => {
                self.scoreboard.add_score(self.score_cursor, -self.score_increment);
            }
            'n' | 'N' => {
                self.state = AppState::TextInput(TextInputContext {
                    purpose: InputPurpose::ScoreboardPlayer,
                    buffer: String::new(),
                    max_len: 15,
                });
            }
            'd' | 'D' => {
                self.scoreboard.remove_player(self.score_cursor);
                if self.score_cursor > 0
                    && self.score_cursor >= self.scoreboard.players.len()
                {
                    self.score_cursor -= 1;
                }
            }
            'r' | 'R' => {
                self.scoreboard.next_round();
            }
            'o' | 'O' => {
                self.scoreboard.sort_by_score();
            }
            's' | 'S' => {
                self.save_state();
            }
            'x' | 'X' => {
                self.scoreboard.reset_scores();
            }
            '1'..='9' => {
                self.score_increment = (key as i32) - ('0' as i32);
            }
            'q' | 'Q' | KEY_MENU | KEY_BACKSPACE => self.go_home(),
            _ => {}
        }
        true
    }

    // ─── Turn Tracker ───────────────────────────────────────────────────────

    fn handle_turns_key(&mut self, key: char, rng: &Rng) -> bool {
        match key {
            KEY_ENTER | ' ' => {
                self.turn_tracker.next_turn();
            }
            'n' | 'N' => {
                self.state = AppState::TextInput(TextInputContext {
                    purpose: InputPurpose::TurnPlayer,
                    buffer: String::new(),
                    max_len: 15,
                });
            }
            'd' | 'D' => {
                let idx = self.turn_tracker.current;
                self.turn_tracker.remove_player(idx);
            }
            'r' | 'R' => {
                self.turn_tracker.randomize_order(rng);
            }
            's' | 'S' => {
                self.turn_tracker.skip_current();
            }
            'q' | 'Q' | KEY_MENU | KEY_BACKSPACE => self.go_home(),
            _ => {}
        }
        true
    }

    // ─── Tournament ─────────────────────────────────────────────────────────

    fn handle_tournament_key(&mut self, key: char, rng: &Rng) -> bool {
        match key {
            'n' | 'N' => {
                if self.tournament.is_none() {
                    // Start collecting names
                    self.tourney_names.clear();
                    self.state = AppState::TextInput(TextInputContext {
                        purpose: InputPurpose::TournamentPlayer,
                        buffer: String::new(),
                        max_len: 15,
                    });
                }
            }
            '1' => {
                if let Some(ref mut t) = self.tournament {
                    t.advance_winner(1);
                }
            }
            '2' => {
                if let Some(ref mut t) = self.tournament {
                    t.advance_winner(2);
                }
            }
            'r' | 'R' => {
                if let Some(ref mut t) = self.tournament {
                    t.random_result(rng);
                }
            }
            'x' | 'X' => {
                // Clear tournament
                self.tournament = None;
                self.tourney_names.clear();
            }
            'q' | 'Q' | KEY_MENU | KEY_BACKSPACE => self.go_home(),
            _ => {}
        }
        true
    }

    // ─── Text Input ─────────────────────────────────────────────────────────

    fn handle_text_input(&mut self, key: char, purpose: InputPurpose, max_len: usize) -> bool {
        match key {
            KEY_ENTER => {
                // Extract buffer from current state
                let buffer = if let AppState::TextInput(ref ctx) = self.state {
                    ctx.buffer.clone()
                } else {
                    return true;
                };

                if buffer.is_empty() {
                    // Empty submit = done (for tournament multi-entry)
                    match purpose {
                        InputPurpose::TournamentPlayer => {
                            // Mark for finalization — rng will be passed in main loop
                            self.state = AppState::TournamentView;
                        }
                        _ => {
                            // Go back to parent screen
                            self.return_from_input();
                        }
                    }
                } else {
                    match purpose {
                        InputPurpose::CoinSideA => {
                            self.coin_config.side_a = buffer;
                            self.state = AppState::TextInput(TextInputContext {
                                purpose: InputPurpose::CoinSideB,
                                buffer: String::new(),
                                max_len,
                            });
                        }
                        InputPurpose::CoinSideB => {
                            self.coin_config.side_b = buffer;
                            self.state = AppState::CoinFlip;
                        }
                        InputPurpose::SpinnerSegment => {
                            self.spinner_config.add_segment(buffer);
                            self.state = AppState::Spinner;
                        }
                        InputPurpose::ScoreboardPlayer => {
                            self.scoreboard.add_player(buffer);
                            self.state = AppState::ScoreboardView;
                        }
                        InputPurpose::TurnPlayer => {
                            self.turn_tracker.add_player(buffer);
                            self.state = AppState::TurnTrackerView;
                        }
                        InputPurpose::TournamentPlayer => {
                            self.tourney_names.push(buffer);
                            // Prompt for another name
                            self.state = AppState::TextInput(TextInputContext {
                                purpose: InputPurpose::TournamentPlayer,
                                buffer: String::new(),
                                max_len,
                            });
                        }
                    }
                }
            }
            KEY_BACKSPACE => {
                if let AppState::TextInput(ref mut ctx) = self.state {
                    ctx.buffer.pop();
                }
            }
            KEY_MENU => {
                self.return_from_input();
            }
            c if !c.is_control() && c != KEY_UP && c != KEY_DOWN
                && c != KEY_LEFT && c != KEY_RIGHT => {
                if let AppState::TextInput(ref mut ctx) = self.state {
                    if ctx.buffer.len() < ctx.max_len {
                        ctx.buffer.push(c);
                    }
                }
            }
            _ => {}
        }
        true
    }

    fn return_from_input(&mut self) {
        if let AppState::TextInput(ref ctx) = self.state {
            match ctx.purpose {
                InputPurpose::CoinSideA | InputPurpose::CoinSideB => {
                    self.state = AppState::CoinFlip;
                }
                InputPurpose::SpinnerSegment => {
                    self.state = AppState::Spinner;
                }
                InputPurpose::ScoreboardPlayer => {
                    self.state = AppState::ScoreboardView;
                }
                InputPurpose::TurnPlayer => {
                    self.state = AppState::TurnTrackerView;
                }
                InputPurpose::TournamentPlayer => {
                    self.state = AppState::TournamentView;
                }
            }
        }
    }

    /// Handle tournament creation with rng (called from main loop).
    pub fn finalize_tournament(&mut self, rng: &Rng) {
        if !self.tourney_names.is_empty() && self.tourney_names.len() >= 2
            && self.tournament.is_none()
        {
            let names = core::mem::take(&mut self.tourney_names);
            self.tournament = Tournament::new(names, rng);
        }
    }
}
