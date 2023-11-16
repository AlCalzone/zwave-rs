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
///             [StartWorking => ! Work => Working],
///         ]],
///         [Working => [
///             [Finished => Done(true)],
///         ]],
///     ],
///     Delays = [
///         [Working => [
///             [Duration::from_millis(1000) => Done(false)],
///             [@my_named_delay => ! Sleep(500) => Done(false)],
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
///     [Pattern (input) => Expression (new state)]
///     [Pattern (input) => ! Expression (effect) => Expression (new state)]
/// ]]
/// ```
/// `Delays` are a list of
/// ```ignore
/// [Pattern (current state) => [
///     [Expression (delay) => Expression (new state)],
///     [@Literal (delay name) => Expression (new state)],
///     [Expression (delay) => ! Expression (effect) => Expression (new state)],
///     [@Literal (delay name) => ! Expression (effect) => Expression (new state)],
/// ]]
/// ```
/// Both specify a condition (input or delay) under which a specific transition to a new state is taken.
/// If a transition includes an effect, it should be executed before entering the new state.
///
/// **Note**: All states, inputs and effects must have unique names.
#[macro_export]
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

            impl $crate::state_machine::StateMachineTransition for [<$fsm_name Transition>] {
                type S = [<$fsm_name State>];
                type E = [<$fsm_name Effect>];

                fn effect(&self) -> Option<Self::E> {
                    self.effect
                }

                fn new_state(&self) -> Self::S {
                    self.new_state
                }
            }

            #[derive(Debug, Clone, PartialEq)]
            struct [<$fsm_name DelayedTransition>] {
                delay: $crate::state_machine::Delay,
                effect: Option<[<$fsm_name Effect>]>,
                new_state: [<$fsm_name State>],
            }

            impl $crate::state_machine::StateMachineTransition for [<$fsm_name DelayedTransition>] {
                type S = [<$fsm_name State>];
                type E = [<$fsm_name Effect>];

                fn effect(&self) -> Option<Self::E> {
                    self.effect
                }

                fn new_state(&self) -> Self::S {
                    self.new_state
                }
            }

            impl $crate::state_machine::StateMachineDelay for [<$fsm_name DelayedTransition>] {
                fn delay(&self) -> &$crate::state_machine::Delay {
                    &self.delay
                }
            }

            impl From<[<$fsm_name DelayedTransition>]> for [<$fsm_name Transition>] {
                fn from(t: [<$fsm_name DelayedTransition>]) -> Self {
                    Self {
                        effect: t.effect,
                        new_state: t.new_state,
                    }
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
                type DT = [<$fsm_name DelayedTransition>];

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

                fn delays(&self) -> Option<Vec<Self::DT>> {
                    use [<$fsm_name State>]::*;
                    // use [<$fsm_name Input>]::*;
                    use [<$fsm_name Effect>]::*;
                    state_machine!(@delay_match (self; $($delay)*))
                }

                fn transition(&mut self, new_state: Self::S) {
                    self.state = new_state;
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
            [$expected_input:pat => ! $effect:expr => $to:expr]
            $($others:tt)*
        ]]
        $($rest:tt)*
    ) $($arms:tt)*) => {
        state_machine!(
            @transition_match (
                $self; $input; [$from => [ $($others)* ]]
                $($rest)*
            )
            $($arms)*
            ($from, $expected_input) => Some(Self::T {
                effect: Some($effect),
                new_state: $to,
            }),
        )
    };

    // From(val) => [ Input(val) => To(val) ]
    (@transition_match (
        $self:ident; $input:ident;
        [$from:pat => [
            [$expected_input:pat => $to:expr]
            $($others:tt)*
        ]]
        $($rest:tt)*
    ) $($arms:tt)*) => {
        state_machine!(
            @transition_match (
                $self; $input; [$from => [ $($others)* ]]
                $($rest)*
            )
            $($arms)*
            ($from, $expected_input) => Some(Self::T {
                effect: None,
                new_state: $to,
            }),
        )
    };

    // Matches when one state has been fully taken care of
    (@transition_match (
        $self:ident; $input:ident;
        [$from:pat => [ $(,)? ]]
        $($rest:tt)*
    ) $($arms:tt)*) => {
        state_machine!(
            @transition_match (
                $self; $input;
                $($rest)*
            )
            $($arms)*
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
    (@delay_match (
        $self:ident;
        [$from:pat => [
            $($delay:tt),* $(,)?
        ]]
        $($rest:tt)*
    ) $($arms:tt)*) => {
        state_machine!(
            @delay_match ($self; $($rest)*)
            $($arms)*
            $from => Some(
                vec![
                    $(state_machine!( @delay_match_one($delay) )),*
                ]
            ),
        )
    };

    // @Named delay, WITH Effect
    (@delay_match_one (
        @$delay:expr => ! $effect:expr => $to:expr
    )) => {
        Self::DT {
            delay: $crate::state_machine::Delay::Named(stringify!($delay)),
            effect: Some($effect),
            new_state: $to,
        }
    };
    // @Named delay, NO Effect
    (@delay_match_one (
        [@$delay:expr => $to:expr]
    )) => {
        Self::DT {
            delay: $crate::state_machine::Delay::Named(stringify!($delay)),
            effect: None,
            new_state: $to,
        }
    };
    // Static delay specified inline, WITH Effect
    (@delay_match_one (
        [$delay:expr => ! $effect:expr => $to:expr]
    )) => {
        Self::DT {
            delay: $crate::state_machine::Delay::Static($delay),
            effect: Some($effect),
            new_state: $to,
        }
    };
    // Static delay specified inline, NO Effect
    (@delay_match_one (
        [$delay:expr => $to:expr]
    )) => {
        Self::DT {
            delay: $crate::state_machine::Delay::Static($delay),
            effect: None,
            new_state: $to,
        }
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
