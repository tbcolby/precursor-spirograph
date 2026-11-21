#![cfg_attr(target_os = "none", no_std)]
#![cfg_attr(target_os = "none", no_main)]

//! Spirograph shell: registers with GAM, routes keys and ticks to `app::App`, and
//! runs the animation pump only while the app is in the foreground *and* has
//! something to animate (it is stopped on Background and when idle). All state
//! and rules live in the host-tested `spirograph-core` crate.

mod app;

use num_traits::{FromPrimitive, ToPrimitive};
use precursor_appkit::{app as kit, keys, pump};

/// xous-names server name. Any unique string; NOT the GAM app name.
const SERVER_NAME: &str = "_Spirograph_";

#[derive(Debug, num_derive::FromPrimitive, num_derive::ToPrimitive)]
pub(crate) enum AppOp {
    Redraw = 0,
    Rawkeys,
    FocusChange,
    Tick,
    Quit,
}

fn main() -> ! {
    log_server::init_wait().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    log::info!("spirograph PID is {}", xous::process::id());

    let xns = xous_names::XousNames::new().unwrap();
    let sid = xns.register_name(SERVER_NAME, None).expect("can't register server");

    // GAM app name comes from the apps/manifest.json key "spirograph".
    let reg = kit::register(
        &xns,
        sid,
        gam::APP_NAME_SPIROGRAPH,
        gam::UxType::Framebuffer,
        kit::Opcodes {
            redraw: AppOp::Redraw.to_u32().unwrap(),
            rawkeys: Some(AppOp::Rawkeys.to_u32().unwrap()),
            focus_change: Some(AppOp::FocusChange.to_u32().unwrap()),
            got_input: None,
        },
    );

    let pump = pump::Pump::spawn(xous::connect(sid).unwrap(), AppOp::Tick.to_usize().unwrap(), 100);
    let mut app = app::App::new(reg.screen);
    let mut foreground = true;

    loop {
        let msg = xous::receive_message(sid).unwrap();
        match FromPrimitive::from_usize(msg.body.id()) {
            Some(AppOp::Redraw) => app.draw(),
            Some(AppOp::Rawkeys) => xous::msg_scalar_unpack!(msg, k1, k2, k3, k4, {
                for key in keys::decode_rawkeys([k1, k2, k3, k4]) {
                    if app.on_key(key) == app::Action::Leave {
                        pump.stop();
                        kit::leave(app.screen());
                    }
                }
                app.draw();
            }),
            Some(AppOp::FocusChange) => xous::msg_scalar_unpack!(msg, code, _, _, _, {
                foreground = matches!(kit::focus_state(code), gam::FocusState::Foreground);
                if foreground {
                    app.on_foreground();
                }
            }),
            Some(AppOp::Tick) => {
                app.on_tick();
                pump::ack(&msg);
            }
            Some(AppOp::Quit) => break,
            None => log::warn!("unknown opcode {:?}", msg.body.id()),
        }
        // Idempotent: the pump ticks only while foreground and animating.
        if foreground && app.animating() {
            pump.run();
        } else {
            pump.stop();
        }
    }
    pump.quit();
    kit::shutdown(&xns, sid)
}
