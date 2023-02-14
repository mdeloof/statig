//! A calculator that is implemented as a state machine.
//!
//! Based on the [calculator example](https://github.com/QuantumLeaps/qpc/tree/master/examples/workstation/calc)
//! in the `qpc` repo.

use statig::blocking::*;

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

    let state_machine = Calculator::default().uninitialized_state_machine().init();

    eframe::run_native(
        "Calculator",
        options,
        Box::new(|_| Box::new(App(state_machine))),
    );
}
