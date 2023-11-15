use std::time::Duration;

/// Describes a state machine transition to take, with an optional effect to be executed before entering the new state
pub trait StateMachineTransition: Sized {
    type S: Sized;
    type E: Sized + Copy;

    fn effect(&self) -> Option<Self::E>;
    fn new_state(&self) -> Self::S;
}

/// Describes an automatic transition to be taken after a delay, with an optional effect to be executed before entering the new state.
pub trait StateMachineDelay: Sized {
    type S: Sized;
    type E: Sized + Copy;

    fn delay(&self) -> &Delay;
    fn effect(&self) -> &Option<Self::E>;
    fn new_state(&self) -> &Self::S;
}

pub enum Delay {
    Static(Duration),
    Named(&'static str),
}

pub trait StateMachine: Sized {
    type S: Sized;
    type E: Sized + Copy;
    type I: Sized + Copy;
    type T: StateMachineTransition + Sized;
    type D: StateMachineDelay + Sized;

    fn new() -> Self;

    /// Determines the next transition to be executed given the current state and the input.
    /// Returns the transition if a valid one exists, otherwise returns None.
    fn next(&self, input: Self::I) -> Option<Self::T>;

    /// Transitions the state machine into the new state
    fn transition(self, state: Self::S) -> Self;

    /// Returns which delays should be scheduled for the current state
    fn delays(&self) -> Option<Vec<Self::D>>;

    /// The current state of the state machine
    fn state(&self) -> &Self::S;

    /// Whether the state machine has reached a final state
    fn done(&self) -> bool;
}

/// Generates a state machine implementation from a declarative syntax, by specifying
/// the possible states, inputs, side effects, transitions and delays,
/// as well as initial and final states. This state machine can be used by a state machine
/// interpreter to drive some logic.
///
/// The syntax is as follows:
/// ```ignore
/// state_machine! { FSM {
///     State = {
///         Initial,
///         Working,
///         Done(bool),
///     },
///     Input = {
///         StartWorking,
///         Finished,
///     },
///     Effect = {
///         Work,
///         Sleep(u8),
///     },
///     Transitions = [
///         [Initial => [
///             StartWorking => ! Work => Working,
///         ]],
///         [Working => [
///             Finished => Done(true),
///         ]],
///     ],
///     Delays = [
///         [Working => [
///             Duration::from_millis(1000) => Done(false),
///             @my_named_delay => ! Sleep(500) => Done(false),
///         ]]
///     ],
///     Initial = Initial,
///     Final = Done(_)
/// } }
/// ```
///
/// `State`, `Input`, and `Effect` desugar to `enum`s. `Initial` is an expression, `Final` a pattern.
///
/// `Transitions` are a list of
/// ```ignore
/// [Pattern (current state) => [
///     Pattern (input) => Expression (new state)
///     Pattern (input) => ! Expression (effect) => Expression (new state)
/// ]]
/// ```
/// `Delays` are a list of
/// ```ignore
/// [Pattern (current state) => [
///     Expression (delay) => Expression (new state),
///     @Literal (delay name) => Expression (new state),
///     Expression (delay) => ! Expression (effect) => Expression (new state),
///     @Literal (delay name) => ! Expression (effect) => Expression (new state),
/// ]]
/// ```
/// Both specify a condition (input or delay) under which a specific transition to a new state is taken.
/// If a transition includes an effect, it should be executed before entering the new state.
///
/// **Note**: All states, inputs and effects must have unique names.
macro_rules! state_machine {
	// TODO: Config and Delays could be optional, but I won't deal with this now
	(
		$fsm_name:ident {
			State = $state_enum:tt,
			Input = $input_enum:tt,
			Effect = $effect_enum:tt,
			Transitions = [
				$( $transition:tt ),* $(,)?
			],
			Delays = [
				$( $delay:tt ),* $(,)?
			],
			Initial = $initial:expr,
			Final = $done:pat$(,)?
		}
	) => {
		paste::paste! {
			#[derive(Debug, Clone, Copy, PartialEq)]
			enum [<$fsm_name State>] $state_enum

			#[derive(Debug, Clone, Copy, PartialEq)]
			enum [<$fsm_name Input>] $input_enum

			#[derive(Debug, Clone, Copy, PartialEq)]
			enum [<$fsm_name Effect>] $effect_enum

			struct [<$fsm_name Transition>] {
				effect: Option<[<$fsm_name Effect>]>,
				new_state: [<$fsm_name State>],
			}

			impl crate::state_machine::StateMachineTransition for [<$fsm_name Transition>] {
				type S = [<$fsm_name State>];
				type E = [<$fsm_name Effect>];

				fn effect(&self) -> Option<Self::E> {
					self.effect
				}

				fn new_state(&self) -> Self::S {
					self.new_state
				}
			}

			struct [<$fsm_name Delay>] {
				delay: crate::state_machine::Delay,
				effect: Option<[<$fsm_name Effect>]>,
				new_state: [<$fsm_name State>],
			}

			impl crate::state_machine::StateMachineDelay for [<$fsm_name Delay>] {
				type S = [<$fsm_name State>];
				type E = [<$fsm_name Effect>];

				fn delay(&self) -> &crate::state_machine::Delay {
					&self.delay
				}

				fn effect(&self) -> &Option<Self::E> {
					&self.effect
				}

				fn new_state(&self) -> &Self::S {
					&self.new_state
				}
			}

			struct $fsm_name {
				state: [<$fsm_name State>],
			}

			impl Default for $fsm_name {
				fn default() -> Self {
					Self::new()
				}
			}

			impl StateMachine for $fsm_name {
				type S = [<$fsm_name State>];
				type E = [<$fsm_name Effect>];
				type I = [<$fsm_name Input>];
				type T = [<$fsm_name Transition>];
				type D = [<$fsm_name Delay>];

				fn new() -> Self {
					use [<$fsm_name State>]::*;
					Self {
						state: $initial,
					}
				}

				fn next(&self, input: Self::I) -> Option<Self::T> {
					use [<$fsm_name State>]::*;
					use [<$fsm_name Input>]::*;
					use [<$fsm_name Effect>]::*;
					state_machine!(@transition_match (self; input; $($transition)*))
				}

				fn delays(&self) -> Option<Vec<Self::D>> {
					use [<$fsm_name State>]::*;
					// use [<$fsm_name Input>]::*;
					use [<$fsm_name Effect>]::*;
					state_machine!(@delay_match (self; $($delay)*))
				}

				fn transition(self, new_state: Self::S) -> Self {
					Self { state: new_state }
				}

				fn state(&self) -> &Self::S {
					&self.state
				}

				fn done(&self) -> bool {
					use [<$fsm_name State>]::*;
					match self.state {
						$done => true,
						_ => false,
					}
				}

			}
		}
	};

	// Generate the match arms for transitions in next()

	// From(val) => [ Input(val) => ! Effect(val) => To(val) ]
	(@transition_match (
		$self:ident; $input:ident;
		[$from:pat => [
			$($expected_input:pat => ! $effect:expr => $to:expr),*$(,)?
		]]
		$($rest:tt)*
	) $($arms:tt)*) => {
		state_machine!(
			@transition_match ($self; $input; $($rest)*)
			$($arms)*
			$(
				($from, $expected_input) => Some(Self::T {
					effect: Some($effect),
					new_state: $to,
				}),
			),*
		)
	};

	// From(val) => [ Input(val) => To(val) ]
	(@transition_match (
		$self:ident; $input:ident;
		[$from:pat => [
			$($expected_input:pat => $to:expr),*$(,)?
		]]
		$($rest:tt)*
	) $($arms:tt)*) => {
		state_machine!(
			@transition_match ($self; $input; $($rest)*)
			$($arms)*
			$(
				($from, $expected_input) => Some(Self::T {
					effect: None,
					new_state: $to,
				}),
			),*
		)
	};
	
	// Matches when there is an unrecognized transition
	(@transition_match (
		$self:ident; $input:ident;
		$unknown:tt
		$($rest:tt)*
	) $($arms:tt)*) => {
		state_machine!(
			@transition_match ($self; $input; $($rest)*)
			$($arms)*
			_ => compile_error!(concat!("Invalid transition ", stringify!($unknown))),
		)
	};
	// Matches when all transitions have been taken care of
	(@transition_match (
		$self:ident; $input:ident;
		$(,)?
	) $($arms:tt)*) => {
		match (&$self.state, $input) {
			$($arms)*
			_ => None
		}
	};

	// Generate the match arms for delays in delays()
	// Static delay specified inline, WITH Effect
	(@delay_match (
		$self:ident;
		[$from:pat => [
			$($delay:expr => ! $effect:expr => $to:expr),* $(,)?
		]]
		$($rest:tt)*
	) $($arms:tt)*) => {
		state_machine!(
			@delay_match ($self; $($rest)*)
			$($arms)*
			$from => Some(
				vec![
					$(Self::D {
						delay: crate::state_machine::Delay::Static($delay),
						effect: Some($effect),
						new_state: $to,
					}),*
				]
			),
		)
	};
	// Static delay specified inline, NO Effect
	(@delay_match (
		$self:ident;
		[$from:pat => [
			$($delay:expr => $to:expr),* $(,)?
		]]
		$($rest:tt)*
	) $($arms:tt)*) => {
		state_machine!(
			@delay_match ($self; $($rest)*)
			$($arms)*
			$from => Some(
				vec![
					$(Self::D {
						delay: crate::state_machine::Delay::Static($delay),
						effect: None,
						new_state: $to,
					}),*
				]
			),
		)
	};
	// @Named delay, WITH Effect
	(@delay_match (
		$self:ident;
		[$from:pat => [
			$(@$delay:expr => ! $effect:expr => $to:expr),* $(,)?
		]]
		$($rest:tt)*
	) $($arms:tt)*) => {
		state_machine!(
			@delay_match ($self; $($rest)*)
			$($arms)*
			$from => Some(
				vec![
					$(Self::D {
						delay: crate::state_machine::Delay::Named(stringify!($delay)),
						effect: Some($effect),
						new_state: $to,
					}),*
				]
			),
		)
	};
	// @Named delay, NO Effect
	(@delay_match (
		$self:ident;
		[$from:pat => [
			$(@$delay:expr => $to:expr),* $(,)?
		]]
		$($rest:tt)*
	) $($arms:tt)*) => {
		state_machine!(
			@delay_match ($self; $($rest)*)
			$($arms)*
			$from => Some(
				vec![
					$(Self::D {
						delay: crate::state_machine::Delay::Named(stringify!($delay)),
						effect: None,
						new_state: $to,
					}),*
				]
			),
		)
	};

	// Matches when there is no recognized delay
	(@delay_match (
		$self:ident;
		$unknown:tt
		$($rest:tt)*
	) $($arms:tt)*) => {
		state_machine!(
			@delay_match ($self; $($rest)*)
			$($arms)*
			_ => compile_error!(concat!("Invalid delay ", stringify!($unknown))),
		)
	};
	// Matches when all delays have been taken care of
	(@delay_match (
		$self:ident;
		$(,)?
	) $($arms:tt)*) => {
		match &$self.state {
			$($arms)*
			_ => None
		}
	};
}

// #[cfg(test)]
mod test {
    use std::time::Duration;

    use super::StateMachine;

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
                Sent => ! Send => WaitingForResponse,
            ]],
            [WaitingForResponse => [
                Response => WaitingForCallback,
            ]],
            [WaitingForCallback => [
                Callback => Done(1),
            ]],
        ],
        Delays = [
            [WaitingForCallback => [
                Duration::from_millis(1000) => !Send => Done(2),
            ]]
        ],
        Initial = Initial,
        Final = Done(_)
    } }

    #[test]
    fn test_fsm_nodelay() {
        let mut fsm = FSM::default();

        // Start the state machine
        let transition = fsm.next(FSMInput::Sent);
        assert!(transition.is_some());
        let transition = transition.unwrap();
        fsm = fsm.transition(transition.new_state);
        assert_eq!(fsm.state(), &(FSMState::WaitingForResponse));

        // Send an unexpected input
        let transition = fsm.next(FSMInput::Callback);
        assert!(transition.is_none());
        assert_eq!(fsm.state(), &(FSMState::WaitingForResponse));

        // Send the expected input
        let transition = fsm.next(FSMInput::Response);
        assert!(transition.is_some());
        let transition = transition.unwrap();
        fsm = fsm.transition(transition.new_state);
        assert_eq!(fsm.state(), &(FSMState::WaitingForCallback));

        // Send an unexpected input
        let transition = fsm.next(FSMInput::Sent);
        assert!(transition.is_none());
        assert_eq!(fsm.state(), &(FSMState::WaitingForCallback));

        // Send the expected input
        let transition = fsm.next(FSMInput::Callback);
        assert!(transition.is_some());
        let transition = transition.unwrap();
        fsm = fsm.transition(transition.new_state);
        assert_eq!(fsm.state(), &(FSMState::Done(1)));

        assert!(fsm.done());
    }
}
