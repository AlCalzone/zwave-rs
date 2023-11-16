#[macro_use]
mod macros;

mod traits;
pub use traits::*;

mod interpreter;
pub use interpreter::*;

#[cfg(test)]
mod test {
    use super::{Delay, StateMachine};
    use std::time::Duration;

    state_machine! { FSM {
        State = {
            Initial,
            WaitingForResponse,
            WaitingForCallback,
            Done(u8),
        },
        Input = {
            Sent,
            Response,
            Callback,
        },
        Effect = {
            Send,
        },
        Transitions = [
            [Initial => [
                [Sent => ! Send => WaitingForResponse]
            ]],
            [WaitingForResponse => [
                [Response => WaitingForCallback],
            ]],
            [WaitingForCallback => [
                [Callback => Done(1)],
            ]],
        ],
        Delays = [
            [WaitingForResponse => [
                [@Custom => Done(1)],
                [Duration::from_millis(1000) => ! Send => Done(2)]
            ]],
            [WaitingForCallback => [
                [Duration::from_millis(1000) => ! Send => Done(2)]
            ]]
        ],
        Initial = Initial,
        Final = Done(_)
    } }

    #[test]
    fn test_fsm() {
        let mut fsm = FSM::default();

        // Start the state machine
        let transition = fsm.next(FSMInput::Sent);
        assert!(transition.is_some());
        let transition = transition.unwrap();
        assert_eq!(transition.effect, Some(FSMEffect::Send));
        fsm.transition(transition.new_state);
        assert_eq!(fsm.state(), &(FSMState::WaitingForResponse));
        assert_eq!(
            fsm.delays(),
            Some(vec![
                FSMDelayedTransition {
                    delay: Delay::Named("Custom"),
                    effect: None,
                    new_state: FSMState::Done(1),
                },
                FSMDelayedTransition {
                    delay: Delay::Static(Duration::from_millis(1000)),
                    effect: Some(FSMEffect::Send),
                    new_state: FSMState::Done(2),
                },
            ])
        );

        // Send an unexpected input
        let transition = fsm.next(FSMInput::Callback);
        assert!(transition.is_none());
        assert_eq!(fsm.state(), &(FSMState::WaitingForResponse));

        // Send the expected input
        let transition = fsm.next(FSMInput::Response);
        assert!(transition.is_some());
        let transition = transition.unwrap();
        fsm.transition(transition.new_state);
        assert_eq!(fsm.state(), &(FSMState::WaitingForCallback));

        // Send an unexpected input
        let transition = fsm.next(FSMInput::Sent);
        assert!(transition.is_none());
        assert_eq!(fsm.state(), &(FSMState::WaitingForCallback));

        // Send the expected input
        let transition = fsm.next(FSMInput::Callback);
        assert!(transition.is_some());
        let transition = transition.unwrap();
        fsm.transition(transition.new_state);
        assert_eq!(fsm.state(), &(FSMState::Done(1)));

        assert!(fsm.done());
    }
}
