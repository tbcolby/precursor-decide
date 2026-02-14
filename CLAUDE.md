# Decision Engine — Agent Instructions

## App Identity
- **Package**: `decide`
- **SERVER_NAME**: `_Decision Engine_`
- **APP_NAME**: `Decide`
- **manifest context_name**: `Decide`

## Architecture
- **UX Type**: `UxType::Chat` with raw keyboard input
- **State Machine**: 10 states (HomeMenu + 8 tools + TextInput)
- **Threading**: None — TRNG calls are fast, all logic is synchronous
- **PDDB**: 1 dictionary (`decide.state`), 4 keys

## Module Organization
- `rng.rs` — TRNG wrapper. **Zero Xous dependencies beyond `trng` crate.**
  - Rejection sampling in `range()` eliminates modulo bias
  - Fisher-Yates shuffle for cards and turn order
  - Weighted selection for spinner
- `dice.rs` — Dice engine. Depends only on `rng.rs`.
- `coin.rs` — Coin flipper. Depends only on `rng.rs`.
- `cards.rs` — Card drawer. Depends only on `rng.rs`.
- `eightball.rs` — Magic 8-Ball. Depends only on `rng.rs`.
- `spinner.rs` — Spinner. Depends only on `rng.rs`.
- `scoreboard.rs` — Scoreboard. Uses `serde` for persistence. No `rng` dependency.
- `turns.rs` — Turn tracker. Depends on `rng.rs` for randomize.
- `tournament.rs` — Tournament bracket. Depends on `rng.rs` for seeding.
- `app.rs` — Master state machine. Coordinates all tools.
- `ui.rs` — All rendering. Only module that uses GAM.
- `storage.rs` — PDDB access. Only module that uses `pddb` crate.

## Text Input Pattern
Text input is a shared sub-state used by multiple tools:
- Coin: custom side names (A and B)
- Spinner: segment names
- Scoreboard: player names
- Turn tracker: player names
- Tournament: player names (multi-entry, empty = done)

`InputPurpose` enum tracks which tool initiated the input.

## Key Design Decisions
1. **Single PDDB dictionary** — unlike QR/barcode apps (2 dicts), this app uses one dict with multiple keys. Simpler for the variety of small data.
2. **Tool modules are independent** — each tool can be understood and modified without reading other tool code.
3. **ASCII art is per-module** — `d6_ascii()` in dice.rs, `coin_ascii()` in coin.rs, `card_ascii()` in cards.rs. Not centralized.
4. **Tournament BYE handling** — bracket rounds up to power of 2, auto-advances BYE matches.

## Patterns Established (for ecosystem)
1. **TRNG wrapper module** — reusable for any app needing randomness
2. **Text input sub-state** — `TextInputContext` pattern for multi-purpose text entry
3. **Tool coordination** — single state machine managing multiple independent tools
4. **Weighted random selection** — `weighted_pick()` pattern for non-uniform distributions

## Build
```bash
cargo build -p decide --target riscv32imac-unknown-xous-elf
cargo xtask renode-image decide
```
