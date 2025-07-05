use rand::{Rng, SeedableRng};
use statig::prelude::*;
use std::time::Instant;

enum Event {
    E1,
    E2,
    E3,
    E4,
    E5,
    E6,
    E7,
    E8,
    E9,
    E10,
    E11,
    E12,
    E13,
    E14,
    E15,
    E16,
    E17,
    E18,
    E19,
    E20,
    E21,
    E22,
    E23,
    E24,
    E25,
    E26,
    E27,
    E28,
    E29,
    E30,
    E31,
    E32,
    E33,
    E34,
    E35,
    E36,
    E37,
    E38,
    E39,
    E40,
    E41,
    E42,
    E43,
    E44,
    E45,
    E46,
    E47,
    E48,
    E49,
    E50,
    E51,
}

struct BenchComplex;

#[state_machine(initial = State::s1(), state(derive(Debug)))]
impl BenchComplex {
    #[state]
    async fn idle(event: &Event) -> Outcome<State> {
        match event {
            Event::E1 => Transition(State::s2()),
            _ => Handled,
        }
    }

    #[state]
    async fn s1(event: &Event) -> Outcome<State> {
        match event {
            Event::E2 => Transition(State::s2()),
            _ => Handled,
        }
    }

    #[state]
    async fn s2(event: &Event) -> Outcome<State> {
        match event {
            Event::E3 => Transition(State::s3()),
            _ => Handled,
        }
    }

    #[state]
    async fn s3(event: &Event) -> Outcome<State> {
        match event {
            Event::E4 => Transition(State::s4()),
            _ => Handled,
        }
    }

    #[state]
    async fn s4(event: &Event) -> Outcome<State> {
        match event {
            Event::E5 => Transition(State::s5()),
            _ => Handled,
        }
    }

    #[state]
    async fn s5(event: &Event) -> Outcome<State> {
        match event {
            Event::E6 => Transition(State::s6()),
            _ => Handled,
        }
    }

    #[state]
    async fn s6(event: &Event) -> Outcome<State> {
        match event {
            Event::E7 => Transition(State::s7()),
            _ => Handled,
        }
    }

    #[state]
    async fn s7(event: &Event) -> Outcome<State> {
        match event {
            Event::E8 => Transition(State::s8()),
            _ => Handled,
        }
    }

    #[state]
    async fn s8(event: &Event) -> Outcome<State> {
        match event {
            Event::E9 => Transition(State::s9()),
            _ => Handled,
        }
    }

    #[state]
    async fn s9(event: &Event) -> Outcome<State> {
        match event {
            Event::E10 => Transition(State::s10()),
            _ => Handled,
        }
    }

    #[state]
    async fn s10(event: &Event) -> Outcome<State> {
        match event {
            Event::E11 => Transition(State::s11()),
            _ => Handled,
        }
    }

    #[state]
    async fn s11(event: &Event) -> Outcome<State> {
        match event {
            Event::E12 => Transition(State::s12()),
            _ => Handled,
        }
    }

    #[state]
    async fn s12(event: &Event) -> Outcome<State> {
        match event {
            Event::E13 => Transition(State::s13()),
            _ => Handled,
        }
    }

    #[state]
    async fn s13(event: &Event) -> Outcome<State> {
        match event {
            Event::E14 => Transition(State::s14()),
            _ => Handled,
        }
    }

    #[state]
    async fn s14(event: &Event) -> Outcome<State> {
        match event {
            Event::E15 => Transition(State::s15()),
            _ => Handled,
        }
    }

    #[state]
    async fn s15(event: &Event) -> Outcome<State> {
        match event {
            Event::E16 => Transition(State::s16()),
            _ => Handled,
        }
    }

    #[state]
    async fn s16(event: &Event) -> Outcome<State> {
        match event {
            Event::E17 => Transition(State::s17()),
            _ => Handled,
        }
    }

    #[state]
    async fn s17(event: &Event) -> Outcome<State> {
        match event {
            Event::E18 => Transition(State::s18()),
            _ => Handled,
        }
    }

    #[state]
    async fn s18(event: &Event) -> Outcome<State> {
        match event {
            Event::E19 => Transition(State::s19()),
            _ => Handled,
        }
    }

    #[state]
    async fn s19(event: &Event) -> Outcome<State> {
        match event {
            Event::E20 => Transition(State::s20()),
            _ => Handled,
        }
    }

    #[state]
    async fn s20(event: &Event) -> Outcome<State> {
        match event {
            Event::E21 => Transition(State::s21()),
            _ => Handled,
        }
    }

    #[state]
    async fn s21(event: &Event) -> Outcome<State> {
        match event {
            Event::E22 => Transition(State::s22()),
            _ => Handled,
        }
    }

    #[state]
    async fn s22(event: &Event) -> Outcome<State> {
        match event {
            Event::E23 => Transition(State::s23()),
            _ => Handled,
        }
    }

    #[state]
    async fn s23(event: &Event) -> Outcome<State> {
        match event {
            Event::E24 => Transition(State::s24()),
            _ => Handled,
        }
    }

    #[state]
    async fn s24(event: &Event) -> Outcome<State> {
        match event {
            Event::E25 => Transition(State::s25()),
            _ => Handled,
        }
    }

    #[state]
    async fn s25(event: &Event) -> Outcome<State> {
        match event {
            Event::E26 => Transition(State::s26()),
            _ => Handled,
        }
    }

    #[state]
    async fn s26(event: &Event) -> Outcome<State> {
        match event {
            Event::E27 => Transition(State::s27()),
            _ => Handled,
        }
    }

    #[state]
    async fn s27(event: &Event) -> Outcome<State> {
        match event {
            Event::E28 => Transition(State::s28()),
            _ => Handled,
        }
    }

    #[state]
    async fn s28(event: &Event) -> Outcome<State> {
        match event {
            Event::E29 => Transition(State::s29()),
            _ => Handled,
        }
    }

    #[state]
    async fn s29(event: &Event) -> Outcome<State> {
        match event {
            Event::E30 => Transition(State::s30()),
            _ => Handled,
        }
    }

    #[state]
    async fn s30(event: &Event) -> Outcome<State> {
        match event {
            Event::E31 => Transition(State::s31()),
            _ => Handled,
        }
    }

    #[state]
    async fn s31(event: &Event) -> Outcome<State> {
        match event {
            Event::E32 => Transition(State::s32()),
            _ => Handled,
        }
    }

    #[state]
    async fn s32(event: &Event) -> Outcome<State> {
        match event {
            Event::E33 => Transition(State::s33()),
            _ => Handled,
        }
    }

    #[state]
    async fn s33(event: &Event) -> Outcome<State> {
        match event {
            Event::E34 => Transition(State::s34()),
            _ => Handled,
        }
    }

    #[state]
    async fn s34(event: &Event) -> Outcome<State> {
        match event {
            Event::E35 => Transition(State::s35()),
            _ => Handled,
        }
    }

    #[state]
    async fn s35(event: &Event) -> Outcome<State> {
        match event {
            Event::E36 => Transition(State::s36()),
            _ => Handled,
        }
    }

    #[state]
    async fn s36(event: &Event) -> Outcome<State> {
        match event {
            Event::E37 => Transition(State::s37()),
            _ => Handled,
        }
    }

    #[state]
    async fn s37(event: &Event) -> Outcome<State> {
        match event {
            Event::E38 => Transition(State::s38()),
            _ => Handled,
        }
    }

    #[state]
    async fn s38(event: &Event) -> Outcome<State> {
        match event {
            Event::E39 => Transition(State::s39()),
            _ => Handled,
        }
    }

    #[state]
    async fn s39(event: &Event) -> Outcome<State> {
        match event {
            Event::E40 => Transition(State::s40()),
            _ => Handled,
        }
    }

    #[state]
    async fn s40(event: &Event) -> Outcome<State> {
        match event {
            Event::E41 => Transition(State::s41()),
            _ => Handled,
        }
    }

    #[state]
    async fn s41(event: &Event) -> Outcome<State> {
        match event {
            Event::E42 => Transition(State::s42()),
            _ => Handled,
        }
    }

    #[state]
    async fn s42(event: &Event) -> Outcome<State> {
        match event {
            Event::E43 => Transition(State::s43()),
            _ => Handled,
        }
    }

    #[state]
    async fn s43(event: &Event) -> Outcome<State> {
        match event {
            Event::E44 => Transition(State::s44()),
            _ => Handled,
        }
    }

    #[state]
    async fn s44(event: &Event) -> Outcome<State> {
        match event {
            Event::E45 => Transition(State::s45()),
            _ => Handled,
        }
    }

    #[state]
    async fn s45(event: &Event) -> Outcome<State> {
        match event {
            Event::E46 => Transition(State::s46()),
            _ => Handled,
        }
    }

    #[state]
    async fn s46(event: &Event) -> Outcome<State> {
        match event {
            Event::E47 => Transition(State::s47()),
            _ => Handled,
        }
    }

    #[state]
    async fn s47(event: &Event) -> Outcome<State> {
        match event {
            Event::E48 => Transition(State::s48()),
            _ => Handled,
        }
    }

    #[state]
    async fn s48(event: &Event) -> Outcome<State> {
        match event {
            Event::E49 => Transition(State::s49()),
            _ => Handled,
        }
    }

    #[state]
    async fn s49(event: &Event) -> Outcome<State> {
        match event {
            Event::E50 => Transition(State::s50()),
            _ => Handled,
        }
    }

    #[state]
    async fn s50(event: &Event) -> Outcome<State> {
        match event {
            Event::E51 => Transition(State::idle()),
            _ => Handled,
        }
    }
}

async fn future_main() {
    let mut state_machine = BenchComplex.uninitialized_state_machine().init().await;

    let loops: u32 = 1_000_000;

    let mut rng = rand::rngs::SmallRng::from_entropy();

    let instant = Instant::now();

    for _ in 0..loops {
        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E1).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E2).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E3).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E4).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E5).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E6).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E7).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E8).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E9).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E10).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E11).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E12).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E13).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E14).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E15).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E16).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E17).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E18).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E19).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E20).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E21).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E22).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E23).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E24).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E25).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E26).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E27).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E28).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E29).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E30).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E31).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E32).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E33).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E34).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E35).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E36).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E37).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E38).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E39).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E40).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E41).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E42).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E43).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E44).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E45).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E46).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E47).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E48).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E49).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E50).await;
        }

        if rng.gen::<usize>() % 2 == 0 {
            state_machine.handle(&Event::E51).await;
        }
    }

    let total_duration = instant.elapsed();
    let loop_duration = total_duration.div_f64(loops as f64);
    let million_loop_duration = loop_duration.mul_f64(1_000_000.0);

    println!("Total duration: {total_duration:?}");
    println!("Average loop duration: {loop_duration:?}");
    println!("Duration 1M loops: {million_loop_duration:?}");

    println!("Final state: {:?}", state_machine.state());
}

fn main() {
    futures::executor::block_on(future_main());
}
