use std::time::Duration;

/// Describes a state machine transition to take, with an optional effect to be executed before entering the new state

pub trait StateMachineTransition: Sized {
    type E: Sized + Copy;
    type SM: Sized;

    fn effect(&self) -> Option<Self::E>;
    fn new_state(&self) -> &Self::SM;
}

/// Describes an automatic transition to be taken after a delay, with an optional effect to be executed before entering the new state.
pub trait StateMachineDelay: Sized {
    type E: Sized + Copy;
    type SM: Sized;

    fn delay(&self) -> Duration;
    fn effect(&self) -> Option<Self::E>;
    fn new_state(&self) -> Self::SM;
}

pub trait StateMachine {
    type S: Sized;
    type E: Sized + Copy;
    type I: Sized + Copy;
    type T: StateMachineTransition + Sized;
    type D: StateMachineDelay + Sized;

    fn new() -> Self
    where
        Self: Sized;

    /// Determines the next transition to be executed given the current state and the input.
    /// Returns the transition if a valid one exists, otherwise returns None.
    fn next(&self, input: Self::I) -> Option<Self::T>;

    /// Returns which delays should be scheduled for the current state
    fn delays(&self) -> Option<Vec<Self::D>>;

    /// The current state of the state machine
    fn state(&self) -> &Self::S;

    /// Whether the state machine has reached a final state
    fn done(&self) -> bool;
}

/// Dummy implementation for state machines that don't use delays
pub struct NoneStateMachineDelay;
impl StateMachineDelay for NoneStateMachineDelay {
    type E = ();
    type SM = ();

    fn delay(&self) -> Duration {
        todo!()
    }

    fn effect(&self) -> Option<Self::E> {
        todo!()
    }

    fn new_state(&self) -> Self::SM {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::{NoneStateMachineDelay, StateMachine};

    #[derive(Debug, Clone, PartialEq)]
    enum State {
        Initial,
        WaitingForResponse,
        WaitingForCallback,
        Done,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    enum Input {
        Sent,
        Response,
        Callback,
    }

    #[derive(Debug, Copy, Clone, PartialEq)]
    enum Effect {
        Send,
    }

    struct Transition {
        effect: Option<Effect>,
        new_state: FSM,
    }

    impl super::StateMachineTransition for Transition {
        type E = Effect;
        type SM = FSM;

        fn effect(&self) -> Option<Self::E> {
            self.effect
        }

        fn new_state(&self) -> &Self::SM {
            &self.new_state
        }
    }

    struct FSM {
        state: State,
    }

    impl StateMachine for FSM {
        type S = State;
        type E = Effect;
        type I = Input;
        type T = Transition;
        type D = NoneStateMachineDelay;

        fn new() -> Self
        where
            Self: Sized,
        {
            Self {
                state: State::Initial,
            }
        }

        fn next(&self, input: Input) -> Option<Self::T> {
            match (&self.state, input) {
                (State::Initial, Input::Sent) => Some(Self::T {
                    effect: Some(Effect::Send),
                    new_state: Self {
                        state: State::WaitingForResponse,
                    },
                }),
                (State::WaitingForResponse, Input::Response) => Some(Self::T {
                    effect: None,
                    new_state: Self {
                        state: State::WaitingForCallback,
                    },
                }),
                (State::WaitingForCallback, Input::Callback) => Some(Self::T {
                    effect: None,
                    new_state: Self { state: State::Done },
                }),
                _ => None,
            }
        }

        fn delays(&self) -> Option<Vec<Self::D>> {
            None
        }

        fn state(&self) -> &Self::S {
            &self.state
        }

        fn done(&self) -> bool {
            self.state == State::Done
        }
    }

    #[test]
    fn test_fsm_nodelay() {
        let mut fsm = FSM::new();

        // Start the state machine
        let transition = fsm.next(Input::Sent);
        assert!(transition.is_some());
        let transition = transition.unwrap();
        fsm = transition.new_state;
        assert_eq!(fsm.state(), &(State::WaitingForResponse));

        // Send an unexpected input
        let transition = fsm.next(Input::Callback);
        assert!(transition.is_none());
        assert_eq!(fsm.state(), &(State::WaitingForResponse));

        // Send the expected input
        let transition = fsm.next(Input::Response);
        assert!(transition.is_some());
        let transition = transition.unwrap();
        fsm = transition.new_state;
        assert_eq!(fsm.state(), &(State::WaitingForCallback));

        // Send an unexpected input
        let transition = fsm.next(Input::Sent);
        assert!(transition.is_none());
        assert_eq!(fsm.state(), &(State::WaitingForCallback));

        // Send the expected input
        let transition = fsm.next(Input::Callback);
        assert!(transition.is_some());
        let transition = transition.unwrap();
        fsm = transition.new_state;
        assert_eq!(fsm.state(), &(State::Done));

        assert!(fsm.done());
    }
}
