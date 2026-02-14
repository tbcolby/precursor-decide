//! The Decision Engine — Precursor's Board Game Companion
//!
//! TRNG-powered dice, cards, coins, spinners, scoreboard, turns, tournaments.
//! Every random number comes from quantum noise — the universe decides.

#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

extern crate alloc;

mod app;
mod cards;
mod coin;
mod dice;
mod eightball;
mod rng;
mod scoreboard;
mod spinner;
mod storage;
mod tournament;
mod turns;
mod ui;

use app::DecideApp;
use rng::Rng;
use num_traits::FromPrimitive;
use num_traits::ToPrimitive;

// Server name for xous names registration (underscored)
const SERVER_NAME: &str = "_Decision Engine_";
// App name for GAM registration (must match manifest context_name)
const APP_NAME: &str = "Decide";

/// Opcodes for the application main loop.
#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
enum AppOp {
    // GAM callbacks: 0-2
    Redraw = 0,
    Rawkeys = 1,
    FocusChange = 2,

    // Control: 255
    Quit = 255,
}

fn main() -> ! {
    // Initialize logging
    log_server::init_wait().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("Decision Engine starting, PID {}", xous::process::id());

    // Connect to name server and register
    let xns = xous_names::XousNames::new().unwrap();
    let sid = xns
        .register_name(SERVER_NAME, None)
        .expect("can't register server");

    // Connect to GAM for graphics
    let gam = gam::Gam::new(&xns).expect("can't connect to GAM");

    // Initialize hardware TRNG
    let rng = Rng::new(&xns);

    // Register UX with GAM
    let token = gam
        .register_ux(gam::UxRegistration {
            app_name: alloc::string::String::from(APP_NAME),
            ux_type: gam::UxType::Chat,
            predictor: None,
            listener: sid.to_array(),
            redraw_id: AppOp::Redraw.to_u32().unwrap(),
            gotinput_id: None,
            audioframe_id: None,
            rawkeys_id: Some(AppOp::Rawkeys.to_u32().unwrap()),
            focuschange_id: Some(AppOp::FocusChange.to_u32().unwrap()),
        })
        .expect("couldn't register UX")
        .unwrap();

    // Get drawing canvas
    let content = gam
        .request_content_canvas(token)
        .expect("couldn't get canvas");
    let screensize = gam
        .get_canvas_bounds(content)
        .expect("couldn't get dimensions");
    log::info!("Canvas size: {:?}", screensize);

    // Initialize app
    let mut app = DecideApp::new();
    app.init_storage();
    let mut allow_redraw = true;

    // Initial draw
    ui::draw(&app, &gam, content);

    // Main event loop
    loop {
        let msg = xous::receive_message(sid).unwrap();
        match FromPrimitive::from_usize(msg.body.id()) {
            Some(AppOp::Redraw) => {
                if allow_redraw {
                    app.needs_redraw = true;
                    ui::draw(&app, &gam, content);
                }
            }
            Some(AppOp::Rawkeys) => xous::msg_scalar_unpack!(msg, k1, k2, k3, k4, {
                let keys = [
                    core::char::from_u32(k1 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k2 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k3 as u32).unwrap_or('\u{0000}'),
                    core::char::from_u32(k4 as u32).unwrap_or('\u{0000}'),
                ];

                let mut should_quit = false;

                for &key in keys.iter() {
                    if key != '\u{0000}' {
                        log::debug!("Key: {:?} (0x{:04X})", key, key as u32);

                        if !app.handle_key(key, &rng) {
                            should_quit = true;
                            break;
                        }

                        // Finalize tournament if names were collected
                        app.finalize_tournament(&rng);
                    }
                }

                if should_quit {
                    break;
                }

                if app.needs_redraw && allow_redraw {
                    ui::draw(&app, &gam, content);
                    app.needs_redraw = false;
                }
            }),
            Some(AppOp::FocusChange) => xous::msg_scalar_unpack!(msg, state_code, _, _, _, {
                match gam::FocusState::convert_focus_change(state_code) {
                    gam::FocusState::Background => {
                        allow_redraw = false;
                        app.save_state();
                    }
                    gam::FocusState::Foreground => {
                        allow_redraw = true;
                        ui::draw(&app, &gam, content);
                    }
                }
            }),
            Some(AppOp::Quit) => break,
            _ => log::warn!("unknown opcode: {:?}", msg.body.id()),
        }
    }

    // Save state before exit
    app.save_state();

    // Cleanup
    xns.unregister_server(sid).unwrap();
    xous::destroy_server(sid).unwrap();
    xous::terminate_process(0)
}
