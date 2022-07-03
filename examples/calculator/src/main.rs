#![feature(generic_associated_types)]

use stateful::StateMachine;

mod state;
mod ui;

use state::Calculator;
use ui::App;

fn main() {
    let options = eframe::NativeOptions {
        resizable: false,
        initial_window_size: Some((300.0, 400.0).into()),
        ..Default::default()
    };
    let mut state_machine: StateMachine<Calculator> = StateMachine::default();

    state_machine.init();

    eframe::run_native(
        "Calculator",
        options,
        Box::new(|_| Box::new(App(state_machine))),
    );
}
