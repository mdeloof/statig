use statig::blocking::{self, *};

pub enum Event {
    Ac,
    Ce,
    Digit { digit: u8 },
    Point,
    Operator { operator: Operator },
    Equal,
}

#[derive(Copy, Clone)]
pub enum Operator {
    Div,
    Mul,
    Sub,
    Add,
}

impl Operator {
    fn eval(&self, operand1: f32, operand2: f32) -> f32 {
        match self {
            Self::Div => operand1 / operand2,
            Self::Mul => operand1 * operand2,
            Self::Sub => operand1 - operand2,
            Self::Add => operand1 + operand2,
        }
    }
}

#[derive(Default)]
pub struct Calculator {
    pub display: String,
}

pub enum State {
    Begin,
    Response { result: f32 },
    Zero1,
    Int1,
    Frac1,
    Negated1,
    OpEntered { operand1: f32, operator: Operator },
    Zero2 { operand1: f32, operator: Operator },
    Int2 { operand1: f32, operator: Operator },
    Frac2 { operand1: f32, operator: Operator },
    Negated2 { operand1: f32, operator: Operator },
}

pub enum Superstate<'sub> {
    Ready,
    Operand1,
    Operand2 {
        operand1: &'sub f32,
        operator: &'sub Operator,
    },
    On,
}

impl blocking::IntoStateMachine for Calculator {
    type State = State;

    type Superstate<'sub> = Superstate<'sub>;

    type Event<'evt> = Event;

    type Context<'ctx> = ();

    type Response = ();

    fn initial() -> Self::State {
        State::Begin
    }
}

impl blocking::State<Calculator> for State {
    fn call_handler(
        &mut self,
        calculator: &mut Calculator,
        event: &Event,
        _: &mut (),
    ) -> Outcome<Self> {
        match self {
            State::Begin => Calculator::begin(calculator, event),
            State::Response { result } => Calculator::result(result, event),
            State::Zero1 => Calculator::zero1(calculator, event),
            State::Int1 => Calculator::int1(calculator, event),
            State::Frac1 => Calculator::frac1(calculator, event),
            State::Negated1 => Calculator::negated1(calculator, event),
            State::OpEntered { operand1, operator } => {
                Calculator::op_entered(calculator, operand1, operator, event)
            }
            State::Zero2 { operand1, operator } => {
                Calculator::zero2(calculator, operand1, operator, event)
            }
            State::Int2 { operand1, operator } => {
                Calculator::int2(calculator, operand1, operator, event)
            }
            State::Frac2 { operand1, operator } => {
                Calculator::frac2(calculator, operand1, operator, event)
            }
            State::Negated2 { operand1, operator } => {
                Calculator::negated2(calculator, operand1, operator, event)
            }
        }
    }

    fn call_entry_action(&mut self, calculator: &mut Calculator, _: &mut ()) {
        match self {
            State::Begin => Calculator::enter_begin(calculator),
            State::Response { result } => Calculator::enter_result(calculator, result),
            _ => {}
        }
    }

    fn superstate(&mut self) -> Option<Superstate<'_>> {
        match self {
            State::Begin => Some(Superstate::Ready),
            State::Response { .. } => Some(Superstate::Ready),
            State::Zero1 => Some(Superstate::Operand1),
            State::Int1 => Some(Superstate::Operand1),
            State::Frac1 => Some(Superstate::Operand1),
            State::Negated1 => Some(Superstate::Operand1),
            State::OpEntered { .. } => Some(Superstate::On),
            State::Zero2 { operand1, operator } => {
                Some(Superstate::Operand2 { operand1, operator })
            }
            State::Int2 { operand1, operator } => Some(Superstate::Operand2 { operand1, operator }),
            State::Frac2 { operand1, operator } => {
                Some(Superstate::Operand2 { operand1, operator })
            }
            State::Negated2 { .. } => Some(Superstate::On),
        }
    }
}

impl<'sub> blocking::Superstate<Calculator> for Superstate<'sub> {
    fn call_handler(
        &mut self,
        calculator: &mut Calculator,
        event: &Event,
        _: &mut (),
    ) -> Outcome<State> {
        match self {
            Superstate::Ready => Calculator::ready(calculator, event),
            Superstate::Operand1 => Calculator::operand1(calculator, event),
            Superstate::Operand2 { operand1, operator } => {
                Calculator::operand2(calculator, operand1, operator, event)
            }
            Superstate::On => Calculator::on(calculator, event),
        }
    }

    fn superstate(&mut self) -> Option<Superstate<'_>> {
        match self {
            Superstate::Ready => Some(Superstate::On),
            Superstate::Operand1 => Some(Superstate::On),
            Superstate::Operand2 { .. } => Some(Superstate::On),
            Superstate::On => None,
        }
    }
}

/// Calculator is a state machine.
impl Calculator {
    fn enter_begin(&mut self) {
        self.display = "0".to_string();
    }

    /// The initial state of the calculator.
    ///
    /// - [`Event::Operator`] => [`negated1`](Self::negated1)
    fn begin(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Operator {
                operator: Operator::Sub,
            } => {
                self.display = "- 0".to_string();
                Transition(State::Negated1)
            }

            _ => Super,
        }
    }

    fn enter_result(&mut self, result: &f32) {
        self.display = result.to_string();
    }

    /// Show the result of the calculation.
    #[allow(unused)]
    fn result(result: &f32, event: &Event) -> Outcome<State> {
        #[allow(clippy::match_single_binding)]
        match event {
            _ => Super,
        }
    }

    /// The calculator is ready to receive a new event.
    ///
    /// - [`Event::Digit`] => [`zero1`](Self::zero1) | [`int1`](Self::int1)
    /// - [`Event::Point`] => [`frac1`](Self::frac1)
    /// - [`Event::Operator`] => [`op_entered`](Self::op_entered)
    /// - [`Event::Ac`] => [`begin`](Self::begin)
    /// - [`Event::Ce`] => [`begin`](Self::begin)
    fn ready(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Digit { digit: 0 } => {
                self.display.clear();
                Transition(State::Zero1)
            }

            Event::Digit { digit } => {
                self.display.clear();
                self.display.push_str(&digit.to_string());
                Transition(State::Int1)
            }

            Event::Point => {
                self.display = "0.".to_string();
                Transition(State::Frac1)
            }

            Event::Operator { operator } => {
                let operand1 = str::parse(&self.display).unwrap();
                Transition(State::OpEntered {
                    operand1,
                    operator: *operator,
                })
            }

            Event::Ac => Transition(State::Begin),

            Event::Ce => Transition(State::Begin),

            _ => Super,
        }
    }

    /// The display contains a single zero.
    ///
    /// - [`Event::Digit`] => [`int1`](Self::int1) | `(handled)`
    /// - [`Event::Point`] => [`frac1`](Self::frac1)
    fn zero1(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Digit { digit: 0 } => Handled(()),

            Event::Digit { digit } => {
                self.display.push_str(&digit.to_string());
                Transition(State::Int1)
            }

            Event::Point => {
                self.display = "0.".to_string();
                Transition(State::Frac1)
            }

            _ => Super,
        }
    }

    /// The integer part of the first operand is being entered.
    ///
    /// - [`Event::Point`] => [`frac1`](Self::frac1)
    /// - [`Event::Digit`] => `(handled)`
    fn int1(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Point => {
                self.display.push('.');
                Transition(State::Frac1)
            }

            Event::Digit { digit } => {
                self.display.push_str(&digit.to_string());
                Handled(())
            }

            _ => Super,
        }
    }

    /// The fractional part of the first operand is being entered.
    ///
    /// - [`Event::Point`] => `(handled)`
    /// - [`Event::Digit`] => `(handled)`
    fn frac1(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Point => Handled(()),

            Event::Digit { digit } => {
                self.display.push_str(&digit.to_string());
                Handled(())
            }

            _ => Super,
        }
    }

    /// The substraction operator has been pressed before entering the first digit,
    /// so the first operand will be negated.
    ///
    /// - [`Event::Digit`] => [`zero1`](Self::zero1) | [`int1`](Self::int1)
    /// - [`Event::Point`] => [`frac1`](Self::frac1)
    /// - [`Event::Operator`] => `(handled)`
    /// - [`Event::Ac`] => [`begin`](Self::begin)
    fn negated1(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Digit { digit: digit @ 0 } => {
                self.display.clear();
                self.display.push('-');
                self.display.push_str(&digit.to_string());
                Transition(State::Zero1)
            }

            Event::Digit { digit } => {
                self.display.clear();
                self.display.push('-');
                self.display.push_str(&digit.to_string());
                Transition(State::Int1)
            }

            Event::Point => {
                self.display.clear();
                self.display.push_str("-0.");
                Transition(State::Frac1)
            }

            Event::Operator { .. } => Handled(()),

            Event::Ac => {
                self.display.clear();
                Transition(State::Begin)
            }

            _ => Super,
        }
    }

    /// The first operand is being entered.
    ///
    /// - [`Event::Ac`] => [`begin`](Self::begin)
    /// - [`Event::Operator`] => [`op_entered`](Self::op_entered)
    /// - [`Event::Equal`] => [`result`](Self::result)
    fn operand1(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Ac => {
                self.display.clear();
                Transition(State::Begin)
            }

            Event::Ce => {
                self.display.clear();
                Handled(())
            }

            Event::Operator { operator } => {
                let operand1 = str::parse(&self.display).unwrap();
                Transition(State::OpEntered {
                    operand1,
                    operator: *operator,
                })
            }

            Event::Equal => {
                let operand1 = str::parse(&self.display).unwrap();
                Transition(State::Response { result: operand1 })
            }

            _ => Super,
        }
    }
    /// The operator that will be applied has been selected.
    ///
    /// - [`Event::Digit`] => [`zero2`](Self::zero2) | [`int2`](Self::int2)
    /// - [`Event::Point`] => [`frac2`](Self::frac2)
    /// - [`Event::Operator`] => [`negated2`](Self::negated2) | `(handled)`
    fn op_entered(&mut self, operand1: &f32, operator: &Operator, event: &Event) -> Outcome<State> {
        match event {
            Event::Digit { digit: 0 } => {
                self.display = "0".to_string();
                Transition(State::Zero2 {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            Event::Digit { digit } => {
                self.display = digit.to_string();
                Transition(State::Int2 {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            Event::Point => {
                self.display = "0.".to_string();
                Transition(State::Frac2 {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            Event::Operator {
                operator: Operator::Sub,
            } => {
                self.display.clear();
                self.display.push_str("-0");
                Transition(State::Negated2 {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            Event::Operator { .. } => Handled(()),

            _ => Super,
        }
    }

    /// The display contains a single zero.
    ///
    /// - [`Event::Digit`] => [`int2`](Self::int2) | `(handled)`
    /// - [`Event::Point`] => [`frac2`](Self::frac2)
    fn zero2(&mut self, operand1: &f32, operator: &Operator, event: &Event) -> Outcome<State> {
        match event {
            Event::Digit { digit: 0 } => Handled(()),

            Event::Digit { digit } => {
                self.display = digit.to_string();
                Transition(State::Int2 {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            Event::Point => {
                self.display = "0.".to_string();
                Transition(State::Frac2 {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            _ => Super,
        }
    }

    /// The integer part of the second operand is being entered.
    ///
    /// - [`Event::Point`] => [`frac2`](Self::frac2)
    /// - [`Event::Digit`] => `(handled)`
    fn int2(&mut self, operand1: &f32, operator: &Operator, event: &Event) -> Outcome<State> {
        match event {
            Event::Point => {
                self.display.push('.');
                Transition(State::Frac2 {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            Event::Digit { digit } => {
                self.display.push_str(&digit.to_string());
                Handled(())
            }

            _ => Super,
        }
    }

    /// The fractional part of the second operand is being entered.
    ///
    /// - [`Event::Point`] => `(handled)`
    /// - [`Event::Digit`] => `(handled)`
    #[allow(unused)]
    fn frac2(&mut self, operand1: &f32, operator: &Operator, event: &Event) -> Outcome<State> {
        match event {
            Event::Point => Handled(()),

            Event::Digit { digit } => {
                self.display.push_str(&digit.to_string());
                Handled(())
            }

            _ => Super,
        }
    }

    /// The second operand is being entered.
    ///
    /// - [`Event::Ac`] => [`op_entered`](Self::op_entered)
    /// - [`Event::Equal`] => [`result`](Self::result)
    /// - [`Event::Operator`] => [`op_entered`](Self::op_entered)
    fn operand2(&mut self, operand1: &f32, operator: &Operator, event: &Event) -> Outcome<State> {
        match event {
            Event::Ac => {
                self.display.clear();
                Transition(State::Begin)
            }

            Event::Ce => {
                self.display = "0".to_string();
                Transition(State::OpEntered {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            Event::Equal => {
                let operand2 = str::parse(&self.display).unwrap();
                let result = operator.eval(*operand1, operand2);
                Transition(State::Response { result })
            }

            Event::Operator {
                operator: next_operator,
            } => {
                let operand2 = str::parse(&self.display).unwrap();
                let result = operator.eval(*operand1, operand2);
                Transition(State::OpEntered {
                    operand1: result,
                    operator: *next_operator,
                })
            }

            _ => Super,
        }
    }

    /// The substraction operator has been pressed before entering the first digit,
    /// so the second operand will be negated.
    ///
    /// - [`Event::Digit`] => [`zero2`](Self::zero2) | [`int2`](Self::int2)
    /// - [`Event::Point`] => [`frac2`](Self::frac2)
    /// - [`Event::Operator`] => `(handled)`
    /// - [`Event::Ac`] => [`op_entered`](Self::op_entered)
    fn negated2(&mut self, operand1: &f32, operator: &Operator, event: &Event) -> Outcome<State> {
        match event {
            Event::Digit { digit: digit @ 0 } => {
                self.display.clear();
                self.display.push('-');
                self.display.push_str(&digit.to_string());
                Transition(State::Zero2 {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            Event::Digit { digit } => {
                self.display.clear();
                self.display.push('-');
                self.display.push_str(&digit.to_string());
                Transition(State::Int2 {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            Event::Point => {
                self.display.clear();
                self.display.push_str("-0.");
                Transition(State::Frac2 {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            Event::Operator { .. } => Handled(()),

            Event::Ac => {
                self.display.clear();
                Transition(State::OpEntered {
                    operand1: *operand1,
                    operator: *operator,
                })
            }

            _ => Super,
        }
    }

    /// The calculator is on and ready to receive events.
    ///
    /// - [`Event::Ac`] => [`begin`](Self::begin)
    fn on(&mut self, event: &Event) -> Outcome<State> {
        match event {
            Event::Ac => {
                self.display.clear();
                Transition(State::Begin)
            }

            _ => Super,
        }
    }
}
