//! UI rendering for the Decision Engine.
//!
//! Draws all screens: home menu, 8 tool screens, text input overlay.

use crate::app::{AppState, DecideApp, InputPurpose, TextInputContext, Tool};
use crate::cards;
use crate::coin;
use crate::dice;
use crate::eightball;
use crate::scoreboard;
use crate::spinner;
use crate::tournament;
use crate::turns;

use gam::*;

// Layout constants (ecosystem standard)
const SCREEN_WIDTH: isize = 336;
const HEADER_HEIGHT: isize = 30;
const FOOTER_HEIGHT: isize = 46;
const CONTENT_TOP: isize = HEADER_HEIGHT;
const CONTENT_BOTTOM: isize = 536 - FOOTER_HEIGHT;

// Font metrics
const REGULAR_HEIGHT: isize = 15;
const LINE_GAP: isize = 4;
const LINE_HEIGHT: isize = REGULAR_HEIGHT + LINE_GAP;

/// Draw the complete screen based on current app state.
pub fn draw(app: &DecideApp, gam: &Gam, canvas: graphics_server::Gid) {
    // Clear screen
    let clear_rect = graphics_server::Rectangle::new_coords_with_style(
        0, 0, SCREEN_WIDTH, 536,
        graphics_server::DrawStyle::new(
            graphics_server::PixelColor::Light,
            graphics_server::PixelColor::Light,
            0,
        ),
    );
    gam.draw_rectangle(canvas, clear_rect).ok();

    match &app.state {
        AppState::HomeMenu => draw_home_menu(app, gam, canvas),
        AppState::DiceRoller => draw_dice(app, gam, canvas),
        AppState::CoinFlip => draw_coin(app, gam, canvas),
        AppState::CardDrawer => draw_cards(app, gam, canvas),
        AppState::EightBall => draw_eightball(app, gam, canvas),
        AppState::Spinner => draw_spinner(app, gam, canvas),
        AppState::ScoreboardView => draw_scoreboard(app, gam, canvas),
        AppState::TurnTrackerView => draw_turns(app, gam, canvas),
        AppState::TournamentView => draw_tournament(app, gam, canvas),
        AppState::TextInput(ctx) => draw_text_input(app, ctx, gam, canvas),
    }

    gam.redraw().ok();
}

// ─── Header / Footer helpers ────────────────────────────────────────────────

fn draw_header(gam: &Gam, canvas: graphics_server::Gid, title: &str) {
    let bg = graphics_server::Rectangle::new_coords_with_style(
        0, 0, SCREEN_WIDTH, HEADER_HEIGHT,
        graphics_server::DrawStyle::new(
            graphics_server::PixelColor::Dark,
            graphics_server::PixelColor::Dark,
            0,
        ),
    );
    gam.draw_rectangle(canvas, bg).ok();

    let mut tv = TextView::new(
        canvas,
        TextBounds::BoundingBox(graphics_server::Rectangle::new_coords(
            8, 2, SCREEN_WIDTH - 8, HEADER_HEIGHT - 2,
        )),
    );
    tv.style = GlyphStyle::Bold;
    tv.invert = true;
    tv.draw_border = false;
    tv.margin = Point::new(0, 0);
    write!(tv, "{}", title).ok();
    gam.post_textview(&mut tv).ok();
}

fn draw_footer(gam: &Gam, canvas: graphics_server::Gid, text: &str) {
    let y = CONTENT_BOTTOM;

    let sep = graphics_server::Line::new_with_style(
        Point::new(0, y),
        Point::new(SCREEN_WIDTH, y),
        graphics_server::DrawStyle::new(
            graphics_server::PixelColor::Dark,
            graphics_server::PixelColor::Dark,
            1,
        ),
    );
    gam.draw_line(canvas, sep).ok();

    let mut tv = TextView::new(
        canvas,
        TextBounds::BoundingBox(graphics_server::Rectangle::new_coords(
            8, y + 4, SCREEN_WIDTH - 8, y + FOOTER_HEIGHT - 4,
        )),
    );
    tv.style = GlyphStyle::Small;
    tv.draw_border = false;
    tv.margin = Point::new(0, 0);
    write!(tv, "{}", text).ok();
    gam.post_textview(&mut tv).ok();
}

fn draw_text(gam: &Gam, canvas: graphics_server::Gid,
    x: isize, y: isize, w: isize, text: &str, style: GlyphStyle) {
    let mut tv = TextView::new(
        canvas,
        TextBounds::BoundingBox(graphics_server::Rectangle::new_coords(
            x, y, x + w, y + LINE_HEIGHT,
        )),
    );
    tv.style = style;
    tv.draw_border = false;
    tv.margin = Point::new(0, 0);
    write!(tv, "{}", text).ok();
    gam.post_textview(&mut tv).ok();
}

fn draw_text_block(gam: &Gam, canvas: graphics_server::Gid,
    x: isize, y: isize, w: isize, h: isize, text: &str, style: GlyphStyle) {
    let mut tv = TextView::new(
        canvas,
        TextBounds::BoundingBox(graphics_server::Rectangle::new_coords(
            x, y, x + w, y + h,
        )),
    );
    tv.style = style;
    tv.draw_border = false;
    tv.margin = Point::new(0, 0);
    write!(tv, "{}", text).ok();
    gam.post_textview(&mut tv).ok();
}

// ─── Home Menu ──────────────────────────────────────────────────────────────

fn draw_home_menu(app: &DecideApp, gam: &Gam, canvas: graphics_server::Gid) {
    draw_header(gam, canvas, "The Decision Engine");

    let tools = Tool::all();
    for (i, tool) in tools.iter().enumerate() {
        let y = CONTENT_TOP + 15 + (i as isize) * (LINE_HEIGHT + 8);
        let selected = i == app.menu_cursor;

        if selected {
            let hl = graphics_server::Rectangle::new_coords_with_style(
                8, y - 2, SCREEN_WIDTH - 8, y + LINE_HEIGHT + 2,
                graphics_server::DrawStyle::new(
                    graphics_server::PixelColor::Dark,
                    graphics_server::PixelColor::Dark,
                    0,
                ),
            );
            gam.draw_rectangle(canvas, hl).ok();
        }

        let mut tv = TextView::new(
            canvas,
            TextBounds::BoundingBox(graphics_server::Rectangle::new_coords(
                16, y, SCREEN_WIDTH - 16, y + LINE_HEIGHT,
            )),
        );
        tv.style = GlyphStyle::Regular;
        tv.invert = selected;
        tv.draw_border = false;
        tv.margin = Point::new(0, 0);
        write!(tv, "{}. {}", i + 1, tool.label()).ok();
        gam.post_textview(&mut tv).ok();
    }

    draw_footer(gam, canvas, "1-8:Select  Enter:Open  Q:Quit");
}

// ─── Dice Roller ────────────────────────────────────────────────────────────

fn draw_dice(app: &DecideApp, gam: &Gam, canvas: graphics_server::Gid) {
    draw_header(gam, canvas, "Dice Roller");

    let mut y = CONTENT_TOP + 10;

    // Config line
    let notation = app.dice_config.notation();
    draw_text(gam, canvas, 16, y, 300, &notation, GlyphStyle::Bold);
    y += LINE_HEIGHT + 8;

    // Result
    if let Some(ref result) = app.dice_result {
        let formatted = dice::format_result(result);
        draw_text_block(gam, canvas, 16, y, 300, 120, &formatted, GlyphStyle::Regular);
        y += 60;

        // Show ASCII d6 art for single d6 rolls
        if app.dice_config.sides == 6 && result.individual.len() <= 3 {
            y += 8;
            for &val in &result.individual {
                let art = dice::d6_ascii(val);
                for line in &art {
                    draw_text(gam, canvas, 40, y, 200, line, GlyphStyle::Regular);
                    y += REGULAR_HEIGHT + 2;
                }
                y += 6;
            }
        }
    } else {
        draw_text(gam, canvas, 16, y, 300,
            "Press ENTER to roll!", GlyphStyle::Regular);
        y += LINE_HEIGHT + 8;
    }

    // Config hints
    y = CONTENT_BOTTOM - 100;
    draw_text(gam, canvas, 16, y, 300,
        "D:Die type  1-9:Count  +/-:Mod", GlyphStyle::Small);
    y += LINE_HEIGHT;
    draw_text(gam, canvas, 16, y, 300,
        "A:Adv  V:Dis  X:Explode  L:Drop", GlyphStyle::Small);

    draw_footer(gam, canvas, "Enter:ROLL!  Q:Back");
}

// ─── Coin Flip ──────────────────────────────────────────────────────────────

fn draw_coin(app: &DecideApp, gam: &Gam, canvas: graphics_server::Gid) {
    draw_header(gam, canvas, "Coin Flip");

    let mut y = CONTENT_TOP + 10;

    // Config
    if app.coin_config.count > 1 {
        let cfg = alloc::format!("Flipping {} coins", app.coin_config.count);
        draw_text(gam, canvas, 16, y, 300, &cfg, GlyphStyle::Regular);
        y += LINE_HEIGHT + 4;
    }

    if app.coin_config.side_a != "HEADS" || app.coin_config.side_b != "TAILS" {
        let custom = alloc::format!("{} / {}",
            app.coin_config.side_a, app.coin_config.side_b);
        draw_text(gam, canvas, 16, y, 300, &custom, GlyphStyle::Small);
        y += LINE_HEIGHT + 4;
    }

    // Result
    if let Some(ref result) = app.coin_result {
        y += 8;

        if result.flips.len() == 1 {
            // Single flip — show big ASCII art
            let art = coin::coin_ascii(result.flips[0],
                &result.side_a_name, &result.side_b_name);
            for line in &art {
                draw_text(gam, canvas, 60, y, 250, line, GlyphStyle::Regular);
                y += REGULAR_HEIGHT + 2;
            }

            y += 12;
            if app.coin_streak > 1 {
                let streak = alloc::format!("Streak: {} in a row!", app.coin_streak);
                draw_text(gam, canvas, 60, y, 200, &streak, GlyphStyle::Bold);
            }
        } else {
            // Multi-flip — show summary
            let formatted = coin::format_result(result);
            draw_text_block(gam, canvas, 16, y, 300, 100, &formatted, GlyphStyle::Regular);
        }
    } else {
        draw_text(gam, canvas, 16, y, 300,
            "Press ENTER to flip!", GlyphStyle::Regular);
    }

    draw_footer(gam, canvas, "Enter:Flip!  N:Count  C:Custom  Q:Back");
}

// ─── Card Drawer ────────────────────────────────────────────────────────────

fn draw_cards(app: &DecideApp, gam: &Gam, canvas: graphics_server::Gid) {
    draw_header(gam, canvas, "Card Drawer");

    let mut y = CONTENT_TOP + 10;

    // Deck status
    let remaining = app.deck.as_ref().map(|d| d.cards_left()).unwrap_or(52);
    let jokers = app.deck.as_ref().map(|d| d.include_jokers).unwrap_or(false);
    let status = alloc::format!("Deck: {} remaining{}",
        remaining, if jokers { " (with Jokers)" } else { "" });
    draw_text(gam, canvas, 16, y, 300, &status, GlyphStyle::Small);
    y += LINE_HEIGHT + 4;

    // Draw count
    let dc = alloc::format!("Drawing: {} card{}", app.draw_count,
        if app.draw_count > 1 { "s" } else { "" });
    draw_text(gam, canvas, 16, y, 300, &dc, GlyphStyle::Small);
    y += LINE_HEIGHT + 8;

    // Drawn cards
    if !app.drawn_cards.is_empty() {
        // Show ASCII art for up to 3 cards side by side
        if app.drawn_cards.len() <= 3 {
            let arts: alloc::vec::Vec<[alloc::string::String; 5]> = app.drawn_cards.iter()
                .map(|&c| cards::card_ascii(c)).collect();

            for row in 0..5 {
                let mut line = alloc::string::String::new();
                for (i, art) in arts.iter().enumerate() {
                    if i > 0 { line += "  "; }
                    line += &art[row];
                }
                draw_text(gam, canvas, 20, y, 310, &line, GlyphStyle::Regular);
                y += REGULAR_HEIGHT + 2;
            }
        } else {
            // Text list for many cards
            let names = cards::format_draw(&app.drawn_cards, remaining);
            draw_text_block(gam, canvas, 16, y, 300, 100, &names, GlyphStyle::Regular);
        }
    } else {
        draw_text(gam, canvas, 16, y, 300,
            "Press ENTER to draw!", GlyphStyle::Regular);
    }

    draw_footer(gam, canvas, "Enter:Draw  1-9:Count  R:Shuffle  Q:Back");
}

// ─── Magic 8-Ball ───────────────────────────────────────────────────────────

fn draw_eightball(app: &DecideApp, gam: &Gam, canvas: graphics_server::Gid) {
    draw_header(gam, canvas, "Magic 8-Ball");

    let mut y = CONTENT_TOP + 20;

    if let Some(ref result) = app.eightball_result {
        let formatted = eightball::format_result(result);
        draw_text_block(gam, canvas, 16, y, 300, 200, &formatted, GlyphStyle::Regular);
    } else {
        draw_text(gam, canvas, 16, y, 300,
            "Ask a yes/no question...", GlyphStyle::Regular);
        y += LINE_HEIGHT + 20;
        draw_text(gam, canvas, 16, y, 300,
            "Then press ENTER to shake!", GlyphStyle::Bold);
    }

    draw_footer(gam, canvas, "Enter:Shake!  Q:Back");
}

// ─── Spinner ────────────────────────────────────────────────────────────────

fn draw_spinner(app: &DecideApp, gam: &Gam, canvas: graphics_server::Gid) {
    draw_header(gam, canvas, "Spinner");

    let mut y = CONTENT_TOP + 10;

    // Segments list
    let seg_text = spinner::format_segments(&app.spinner_config, app.spinner_cursor);
    draw_text_block(gam, canvas, 16, y, 300, 200, &seg_text, GlyphStyle::Regular);

    // Spin result
    if let Some(ref result) = app.spin_result {
        y = CONTENT_TOP + 250;
        let result_text = spinner::format_result(&app.spinner_config, result);
        draw_text_block(gam, canvas, 16, y, 300, 120, &result_text, GlyphStyle::Bold);
    }

    draw_footer(gam, canvas, "Enter:Spin! N:Add D:Del +/-:Weight Q:Back");
}

// ─── Scoreboard ─────────────────────────────────────────────────────────────

fn draw_scoreboard(app: &DecideApp, gam: &Gam, canvas: graphics_server::Gid) {
    draw_header(gam, canvas, "Scoreboard");

    let mut y = CONTENT_TOP + 10;

    let board_text = scoreboard::format_scoreboard(&app.scoreboard, app.score_cursor);
    draw_text_block(gam, canvas, 16, y, 300, 280, &board_text, GlyphStyle::Regular);

    // Score increment indicator
    y = CONTENT_BOTTOM - 60;
    let inc = alloc::format!("Increment: {} (1-9 to change)", app.score_increment);
    draw_text(gam, canvas, 16, y, 300, &inc, GlyphStyle::Small);

    draw_footer(gam, canvas, "Arrows:+/- N:Add R:Round S:Save Q:Back");
}

// ─── Turn Tracker ───────────────────────────────────────────────────────────

fn draw_turns(app: &DecideApp, gam: &Gam, canvas: graphics_server::Gid) {
    draw_header(gam, canvas, "Turn Tracker");

    let tracker_text = turns::format_tracker(&app.turn_tracker);
    draw_text_block(gam, canvas, 16, CONTENT_TOP + 10, 300, 300,
        &tracker_text, GlyphStyle::Regular);

    draw_footer(gam, canvas, "Enter:Next N:Add D:Del R:Random S:Skip Q:Back");
}

// ─── Tournament ─────────────────────────────────────────────────────────────

fn draw_tournament(app: &DecideApp, gam: &Gam, canvas: graphics_server::Gid) {
    draw_header(gam, canvas, "Tournament");

    if let Some(ref tourney) = app.tournament {
        let bracket_text = tournament::format_bracket(tourney);
        draw_text_block(gam, canvas, 16, CONTENT_TOP + 10, 300, 400,
            &bracket_text, GlyphStyle::Regular);

        draw_footer(gam, canvas, "1/2:Advance R:Random X:Clear Q:Back");
    } else {
        draw_text(gam, canvas, 16, CONTENT_TOP + 20, 300,
            "No tournament active.", GlyphStyle::Regular);
        draw_text(gam, canvas, 16, CONTENT_TOP + 50, 300,
            "Press N to create one.", GlyphStyle::Bold);
        draw_text(gam, canvas, 16, CONTENT_TOP + 80, 300,
            "(Enter names, then press", GlyphStyle::Small);
        draw_text(gam, canvas, 16, CONTENT_TOP + 100, 300,
            "Enter on empty to start)", GlyphStyle::Small);

        draw_footer(gam, canvas, "N:New tournament  Q:Back");
    }
}

// ─── Text Input Overlay ─────────────────────────────────────────────────────

fn draw_text_input(_app: &DecideApp, ctx: &TextInputContext,
    gam: &Gam, canvas: graphics_server::Gid)
{
    let prompt = match ctx.purpose {
        InputPurpose::CoinSideA => "Enter side A name:",
        InputPurpose::CoinSideB => "Enter side B name:",
        InputPurpose::SpinnerSegment => "Enter segment name:",
        InputPurpose::ScoreboardPlayer => "Enter player name:",
        InputPurpose::TurnPlayer => "Enter player name:",
        InputPurpose::TournamentPlayer => "Enter player name (empty=done):",
    };

    draw_header(gam, canvas, "Input");

    let mut y = CONTENT_TOP + 30;
    draw_text(gam, canvas, 16, y, 300, prompt, GlyphStyle::Regular);
    y += LINE_HEIGHT + 12;

    // Input box
    let box_rect = graphics_server::Rectangle::new_coords_with_style(
        16, y, SCREEN_WIDTH - 16, y + LINE_HEIGHT + 8,
        graphics_server::DrawStyle::new(
            graphics_server::PixelColor::Dark,
            graphics_server::PixelColor::Light,
            1,
        ),
    );
    gam.draw_rectangle(canvas, box_rect).ok();

    // Buffer text with cursor
    let display = alloc::format!("{}|", ctx.buffer);
    draw_text(gam, canvas, 20, y + 4, 290, &display, GlyphStyle::Regular);

    // Tournament: show names entered so far
    if ctx.purpose == InputPurpose::TournamentPlayer {
        y += LINE_HEIGHT + 30;
        if !_app.tourney_names.is_empty() {
            let count = alloc::format!("Players added: {}", _app.tourney_names.len());
            draw_text(gam, canvas, 16, y, 300, &count, GlyphStyle::Small);
            y += LINE_HEIGHT + 4;
            for (i, name) in _app.tourney_names.iter().enumerate() {
                let entry = alloc::format!("{}. {}", i + 1, name);
                draw_text(gam, canvas, 24, y, 280, &entry, GlyphStyle::Small);
                y += LINE_HEIGHT;
                if y > CONTENT_BOTTOM - 60 {
                    break;
                }
            }
        }
    }

    draw_footer(gam, canvas, "Enter:Submit  Backspace:Delete  Menu:Cancel");
}
