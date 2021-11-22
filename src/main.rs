mod event_processing;
mod geometry_manager;
mod move_logic;
mod resize_logic;
mod utils;
mod wcomp;

use crate::wcomp::WComp;

fn main() {
    env_logger::init();

    let mut event_loop = calloop::EventLoop::try_new().unwrap();
    let loop_handle = event_loop.handle();
    let loop_signal = event_loop.get_signal();

    let signals_event_source = loop_handle
        .insert_source(
            calloop::signals::Signals::new(&[calloop::signals::Signal::SIGINT]).unwrap(),
            move |_event, _metadata, _data| {
                println!("\nSigint detected, closing...");
                loop_signal.stop();
            },
        )
        .unwrap();

    let mut wcomp = WComp::new();
    wcomp.run(&mut event_loop);
    loop_handle.remove(signals_event_source);
}
