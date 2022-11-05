use statig::prelude::*;

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

impl StateMachine for Calculator {
    type State = State;

    type Superstate<'a> = Superstate<'a>;

    type Event = Event;

    const INIT_STATE: State = State::begin();
}

/// Calculator is a state machine.
#[state_machine]
impl Calculator {
    #[action]
    fn enter_begin(&mut self) {
        self.display = "0".to_string();
    }

    /// The initial state of the calculator.
    ///
    /// - [`Event::Operator`] => [`negated1`](Self::negated1)
    #[state(superstate = "ready", entry_action = "enter_begin")]
    fn begin(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Operator {
                operator: Operator::Sub,
            } => {
                self.display = "- 0".to_string();
                Transition(State::negated1())
            }

            _ => Super,
        }
    }

    #[action]
    fn enter_result(&mut self, result: &f32) {
        self.display = result.to_string();
    }

    /// Show the result of the calculation.
    #[state(
        superstate = "ready",
        entry_action = "enter_result",
        local_storage("result: f32")
    )]
    fn result(event: &Event) -> Response<State> {
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
    #[superstate(superstate = "on")]
    fn ready(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Digit { digit: 0 } => {
                self.display.clear();
                Transition(State::zero1())
            }

            Event::Digit { digit } => {
                self.display.clear();
                self.display.push_str(&digit.to_string());
                Transition(State::int1())
            }

            Event::Point => {
                self.display = "0.".to_string();
                Transition(State::frac1())
            }

            Event::Operator { operator } => {
                let operand1 = str::parse(&self.display).unwrap();
                Transition(State::OpEntered {
                    operand1,
                    operator: *operator,
                })
            }

            Event::Ac => Transition(State::begin()),

            Event::Ce => Transition(State::begin()),

            _ => Super,
        }
    }

    /// The display contains a single zero.
    ///
    /// - [`Event::Digit`] => [`int1`](Self::int1) | `(handled)`
    /// - [`Event::Point`] => [`frac1`](Self::frac1)
    #[state(superstate = "operand1")]
    fn zero1(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Digit { digit: 0 } => Handled,

            Event::Digit { digit } => {
                self.display.push_str(&digit.to_string());
                Transition(State::int1())
            }

            Event::Point => {
                self.display = "0.".to_string();
                Transition(State::frac1())
            }

            _ => Super,
        }
    }

    /// The integer part of the first operand is being entered.
    ///
    /// - [`Event::Point`] => [`frac1`](Self::frac1)
    /// - [`Event::Digit`] => `(handled)`
    #[state(superstate = "operand1")]
    fn int1(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Point => {
                self.display.push('.');
                Transition(State::frac1())
            }

            Event::Digit { digit } => {
                self.display.push_str(&digit.to_string());
                Handled
            }

            _ => Super,
        }
    }

    /// The fractional part of the first operand is being entered.
    ///
    /// - [`Event::Point`] => `(handled)`
    /// - [`Event::Digit`] => `(handled)`
    #[state(superstate = "operand1")]
    fn frac1(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Point => Handled,

            Event::Digit { digit } => {
                self.display.push_str(&digit.to_string());
                Handled
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
    #[state(superstate = "operand1")]
    fn negated1(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Digit { digit: digit @ 0 } => {
                self.display.clear();
                self.display.push('-');
                self.display.push_str(&digit.to_string());
                Transition(State::zero1())
            }

            Event::Digit { digit } => {
                self.display.clear();
                self.display.push('-');
                self.display.push_str(&digit.to_string());
                Transition(State::int1())
            }

            Event::Point => {
                self.display.clear();
                self.display.push_str("-0.");
                Transition(State::frac1())
            }

            Event::Operator { .. } => Handled,

            Event::Ac => {
                self.display.clear();
                Transition(State::begin())
            }

            _ => Super,
        }
    }

    /// The first operand is being entered.
    ///
    /// - [`Event::Ac`] => [`begin`](Self::begin)
    /// - [`Event::Operator`] => [`op_entered`](Self::op_entered)
    /// - [`Event::Equal`] => [`result`](Self::result)
    #[superstate(superstate = "on")]
    fn operand1(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Ac => {
                self.display.clear();
                Transition(State::begin())
            }

            Event::Ce => {
                self.display.clear();
                Handled
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
                Transition(State::result(operand1))
            }

            _ => Super,
        }
    }
    /// The operator that will be applied has been selected.
    ///
    /// - [`Event::Digit`] => [`zero2`](Self::zero2) | [`int2`](Self::int2)
    /// - [`Event::Point`] => [`frac2`](Self::frac2)
    /// - [`Event::Operator`] => [`negated2`](Self::negated2) | `(handled)`
    #[state(superstate = "on")]
    fn op_entered(
        &mut self,
        operand1: &f32,
        operator: &Operator,
        event: &Event,
    ) -> Response<State> {
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

            Event::Operator { .. } => Handled,

            _ => Super,
        }
    }

    /// The display contains a single zero.
    ///
    /// - [`Event::Digit`] => [`int2`](Self::int2) | `(handled)`
    /// - [`Event::Point`] => [`frac2`](Self::frac2)
    #[state(superstate = "operand2")]
    fn zero2(&mut self, operand1: &f32, operator: &Operator, event: &Event) -> Response<State> {
        match event {
            Event::Digit { digit: 0 } => Handled,

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
    #[state(superstate = "operand2")]
    fn int2(&mut self, operand1: &f32, operator: &Operator, event: &Event) -> Response<State> {
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
                Handled
            }

            _ => Super,
        }
    }

    /// The fractional part of the second operand is being entered.
    ///
    /// - [`Event::Point`] => `(handled)`
    /// - [`Event::Digit`] => `(handled)`
    #[state(
        superstate = "operand2",
        local_storage("operand1: f32", "operator: Operator")
    )]
    fn frac2(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Point => Handled,

            Event::Digit { digit } => {
                self.display.push_str(&digit.to_string());
                Handled
            }

            _ => Super,
        }
    }

    /// The second operand is being entered.
    ///
    /// - [`Event::Ac`] => [`op_entered`](Self::op_entered)
    /// - [`Event::Equal`] => [`result`](Self::result)
    /// - [`Event::Operator`] => [`op_entered`](Self::op_entered)
    #[superstate(superstate = "on")]
    fn operand2(&mut self, operand1: &f32, operator: &Operator, event: &Event) -> Response<State> {
        match event {
            Event::Ac => {
                self.display.clear();
                Transition(State::begin())
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
                Transition(State::result(result))
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
    #[state(superstate = "on")]
    fn negated2(&mut self, operand1: &f32, operator: &Operator, event: &Event) -> Response<State> {
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

            Event::Operator { .. } => Handled,

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
    #[superstate]
    fn on(&mut self, event: &Event) -> Response<State> {
        match event {
            Event::Ac => {
                self.display.clear();
                Transition(State::begin())
            }

            _ => Super,
        }
    }
}
