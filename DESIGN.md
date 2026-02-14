# THE DECISION ENGINE
## Design Specification

```
"God does not play dice with the universe."
    — Albert Einstein

"Stop telling God what to do."
    — Niels Bohr

"Hold my TRNG."
    — Precursor
```

---

## Problem Statement

You're at the table. The board is set. Someone lost the dice. Someone else is arguing about whose turn it is. The score is written on a napkin that got wet. And nobody can agree on what to have for dinner after the game.

The Decision Engine solves all of this. It is a board game companion, a randomness oracle, a scorekeeper, a turn tracker, and an argument ender — powered by the only hardware true random number generator in anyone's pocket.

This is not pseudo-random. This is quantum noise converted to dice rolls, card draws, coin flips, and life decisions. The universe itself is choosing your fate.

---

## Precursor Fit: Born For This

| Constraint | Advantage |
|-----------|-----------|
| Hardware TRNG | TRUE randomness — quantum noise, not algorithms. The most honest dice on Earth. |
| Physical keyboard | Fast input: number of dice, sides, modifiers, player names |
| 1-bit display | Clean number display, ASCII art dice faces, card suits |
| PDDB encrypted | Save game state — nobody can peek at the scoreboard |
| Vibration motor | Haptic feedback on rolls — *feel* the randomness |
| No network | No one can accuse you of rigged results |
| Portable | Fits in your pocket, comes to every game night |

---

## Features

### DICE ROLLER (The Flagship)
- Standard dice: d4, d6, d8, d10, d12, d20, d100
- Custom sides: d(N) for any N from 2-999
- Multiple dice: 1-20 dice at once
- Roll modifiers: +N, -N (e.g., "2d6+3")
- Advantage/Disadvantage: roll twice, take higher/lower (D&D 5e)
- Show individual results AND total
- Roll history (last 50 rolls)
- Exploding dice: if max value, roll again and add
- Drop lowest: NdX drop L (e.g., "4d6 drop 1" for D&D stats)
- ASCII art for d6 faces

### COIN FLIP
- Heads or Tails with big ASCII art
- Flip streak counter
- Multi-flip: flip N coins, show count of each
- Custom coin: rename sides (Yes/No, Pizza/Sushi, etc.)

### CARD DRAWER
- Standard 52-card deck + 2 jokers
- Draw N cards at a time
- Track remaining deck (no reshuffling until reset)
- Card display with suit symbols (♠ ♥ ♦ ♣)
- Multiple deck support (for games needing 2+ decks)
- Shuffle and reset

### MAGIC 8-BALL
- Ask a yes/no question
- 20 classic responses ("It is certain", "Don't count on it", etc.)
- Vibration on shake (tap Enter to "shake")
- Big dramatic reveal

### SPINNER
- Custom spinner with 2-12 segments
- Named segments (player names, options, etc.)
- Animated selection (rapid cycling → slow → land)
- Equal or weighted probability

### SCOREBOARD
- 2-8 players with custom names
- Add/subtract points
- Running totals
- Sort by score
- Round tracking
- Save/load game to PDDB
- Reset scores

### TURN TRACKER
- 2-20 players in order
- Current player highlighted
- Next turn advances automatically
- Random turn order (TRNG shuffle)
- Skip/remove player
- Round counter

### TOURNAMENT BRACKET
- 2-16 players
- Single elimination bracket
- Random seeding via TRNG
- Advance winners
- Display bracket tree

---

## Screen Flows

```
                        ┌──────────┐
                        │   HOME   │
                        │  MENU    │
                        └────┬─────┘
                             │
       ┌──────┬──────┬──────┼──────┬──────┬──────┬──────┐
       ▼      ▼      ▼      ▼      ▼      ▼      ▼      ▼
    ┌──────┐┌────┐┌──────┐┌────┐┌──────┐┌──────┐┌────┐┌──────┐
    │ Dice ││Coin││Cards ││ 8  ││Spin- ││Score-││Turn││Tourn-│
    │Roller││Flip││Drawer││Ball││ ner  ││board ││Trkr││ament │
    └──────┘└────┘└──────┘└────┘└──────┘└──────┘└────┘└──────┘
```

---

## Keyboard Mapping

### Home Menu
| Key | Action |
|-----|--------|
| Up/Down | Navigate tools |
| Enter | Select tool |
| Q | Quit app |
| 1-8 | Quick-select tool by number |

### Dice Roller
| Key | Action |
|-----|--------|
| 1-9 | Number of dice (hold Shift for 10+) |
| D | Cycle die type (d4→d6→d8→d10→d12→d20→d100) |
| Enter/Space | ROLL! |
| +/- | Add/remove modifier |
| A | Toggle Advantage |
| X | Toggle Exploding |
| L | Toggle Drop Lowest |
| H | Show roll history |
| Q | Back to menu |

### Coin Flip
| Key | Action |
|-----|--------|
| Enter/Space | Flip! |
| N | Number of coins (1-20) |
| C | Custom side names |
| Q | Back |

### Card Drawer
| Key | Action |
|-----|--------|
| Enter/Space | Draw card(s) |
| 1-9 | Set draw count |
| R | Reset/shuffle deck |
| D | Show remaining deck count |
| Q | Back |

### Magic 8-Ball
| Key | Action |
|-----|--------|
| Enter/Space | Shake and reveal |
| Q | Back |

### Spinner
| Key | Action |
|-----|--------|
| Enter/Space | Spin! |
| N | Add segment |
| D | Delete segment |
| E | Edit segment |
| Q | Back |

### Scoreboard
| Key | Action |
|-----|--------|
| Up/Down | Select player |
| +/Right | Add point |
| -/Left | Subtract point |
| N | New player |
| D | Delete player |
| R | New round |
| S | Save game |
| O | Sort by score |
| Q | Back |

### Turn Tracker
| Key | Action |
|-----|--------|
| Enter/Space | Next turn |
| N | Add player |
| D | Remove player |
| R | Randomize order |
| S | Skip current |
| Q | Back |

### Tournament
| Key | Action |
|-----|--------|
| N | New tournament (enter names) |
| Up/Down | Select match |
| 1/2 | Advance player 1 or 2 |
| R | Random result |
| Q | Back |

---

## PDDB Schema

### Dictionary: `decide.state`
| Key | Format | Description |
|-----|--------|-------------|
| `config` | JSON | `{ "last_tool": "dice", "vibrate": true }` |
| `scoreboard` | JSON | Players, scores, rounds |
| `tournament` | JSON | Bracket state |
| `spinner` | JSON | Segment names |
| `deck` | JSON | Remaining cards |
| `roll_history` | JSON | Last 50 rolls |
| `custom_coin` | JSON | Custom side names |

---

## ASCII Art

### d6 Faces
```
┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐  ┌─────┐
│     │  │  o  │  │  o  │  │ o o │  │ o o │  │ o o │
│  o  │  │     │  │  o  │  │     │  │  o  │  │ o o │
│     │  │  o  │  │  o  │  │ o o │  │ o o │  │ o o │
└─────┘  └─────┘  └─────┘  └─────┘  └─────┘  └─────┘
   1        2        3        4        5        6
```

### Coin
```
  ╔═══════╗       ╔═══════╗
  ║       ║       ║       ║
  ║ HEADS ║       ║ TAILS ║
  ║       ║       ║       ║
  ╚═══════╝       ╚═══════╝
```

### Card
```
  ┌───────┐
  │ A     │
  │   ♠   │
  │     A │
  └───────┘
```

---

## Complexity Estimate

| Metric | Estimate |
|--------|----------|
| Total LOC | ~2,500-3,500 |
| Modules | 8+ (main, app, dice, coin, cards, spinner, scoreboard, tournament, ui, storage) |
| Threading | None (TRNG calls are fast) |
| PDDB | 1 dictionary, multiple keys |
| New dependency | trng (hardware TRNG) |
| Vibration | Yes (gam.set_vibe) |
| Key challenge | Tournament bracket layout on 336px; ASCII art rendering |
