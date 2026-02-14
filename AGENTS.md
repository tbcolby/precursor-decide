# Decision Engine — Agent Evolution Report

## Agents Used

### From xous-dev-toolkit:
1. **ideation.md** — DESIGN.md creation, feature set for 8 tools, screen flow design
2. **architecture.md** — State machine for 10 states, module independence, text input sub-state pattern
3. **graphics.md** — Screen layout, header/footer helpers, ASCII art rendering, multi-screen dispatch
4. **storage.md** — PDDB schema (1 dict, 4 keys), JSON serialization, load/save lifecycle
5. **build.md** — Cargo.toml with trng dependency, manifest integration
6. **review.md** — Standards compliance, keyboard mapping consistency
7. **encoding.md** — Referenced for ASCII art patterns (card faces, dice faces)

### Not Used:
- **networking.md** — No network features
- **system.md** — TRNG accessed via trng crate, not raw hardware
- **testing.md** — Renode capture to be added

## New Specialist Agent: `randomness.md`

**Born from this build.** The TRNG wrapper and randomness patterns are reusable for any app needing hardware random numbers.

### Agent Definition: `agents/randomness.md`

**Role**: Implement true random number generation patterns for Precursor apps, ensuring uniform distributions, proper bias elimination, and cryptographic-quality randomness.

**Expertise**:
- Hardware TRNG access via `trng::Trng`
- Rejection sampling for uniform distribution (modulo bias elimination)
- Fisher-Yates shuffle for permutation generation
- Weighted random selection from non-uniform distributions
- Dice notation parsing and evaluation
- Card deck management (draw without replacement)
- Streak/pattern detection in random sequences
- Tournament seeding and bracket generation

**Patterns**:
- `Rng` wrapper struct encapsulating TRNG connection
- `range(max)` with rejection at `u32::MAX - (u32::MAX % max)` threshold
- `range_inclusive(min, max)` = `min + range(max - min + 1)`
- `roll_die(sides)` = `range_inclusive(1, sides)`
- `flip()` = LSB of `u32()` (fastest boolean)
- `shuffle()` = Fisher-Yates with `range()` for each swap
- `pick()` = uniform selection from slice
- `weighted_pick()` = cumulative weight scan with `range(total)`
- `shuffled_deck()` = generate ordered sequence then shuffle

**Quality Criteria**:
- TRNG wrapper has no GAM/PDDB dependencies (extractable)
- Rejection sampling MUST be used for `range()` — never raw modulo
- Fisher-Yates shuffle iterates in reverse (standard algorithm)
- Weighted pick handles zero-total gracefully
- All random operations documented with distribution guarantees

**Handoffs**:
- FROM architecture.md: "App needs randomness — what type?"
- TO any tool module: "Here's an `&Rng` — use it for all random decisions"

## Ecosystem Standards Confirmed

This build validated:
- Keyboard constants — same `KEY_UP/DOWN/LEFT/RIGHT/ENTER/BACKSPACE/MENU` pattern works
- Header/footer layout — reused without modification
- PDDB naming — `decide.state` follows `appname.category` pattern
- JSON settings — serde_json works well for small structured data
- Focus change save — background/foreground lifecycle correct
- Menu navigation — highlight bar + inverted text pattern works for 8-item menu

## New Patterns Discovered

1. **Multi-tool state machine**: 10+ states manageable with match dispatch. Each tool gets its own `handle_*_key()` method. Clean separation.

2. **Text input sub-state**: `TextInputContext` with `InputPurpose` enum allows any tool to request text input. The input handler returns to the correct parent screen. Reusable pattern for all future apps needing keyboard text entry.

3. **Independent tool modules**: Each tool (dice, coin, cards, etc.) has zero knowledge of other tools. Only `app.rs` knows about all of them. This makes adding/removing tools trivial.

4. **TRNG threading**: `Rng` struct passed by reference, not stored globally. Created once in `main()`, borrowed by `app.handle_key()`. Clean ownership.

## Recommended Toolkit Updates

1. **Add `randomness.md`** to `agents/` directory
2. **Update `architecture.md`**: Add "multi-tool state machine" pattern for apps with multiple independent features
3. **Update `graphics.md`**: Add "text input overlay" pattern for in-app text entry
4. **Update STANDARDS.md**: Add text input pattern as reusable component

## Metrics

| Metric | Value |
|--------|-------|
| Source files | 12 |
| Estimated LOC | ~3,200 |
| PDDB dictionaries | 1 |
| States | 10 |
| Tool modules | 8 |
| Toolkit agents used | 7 of 11 |
| New agents proposed | 1 (randomness.md) |
