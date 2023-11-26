use std::{fmt::Debug, marker::Send, time::Duration, cmp::Ordering};

/// Describes a state machine transition to take, with an optional effect to be executed before entering the new state
pub trait StateMachineTransition: Sized + Clone + Debug + Send {
    type S: Sized + Clone + Debug + Send + Sync + 'static;
    type E: Sized + Clone + Debug + Send + 'static;

    fn effect(&self) -> Option<Self::E>;
    fn new_state(&self) -> Self::S;
}

/// Describes an automatic transition to be taken after a delay, with an optional effect to be executed before entering the new state.
pub trait StateMachineDelay: Sized + Send {
    fn delay(&self) -> &Delay;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delay {
    Static(Duration),
    Named(&'static str),
}

impl Delay {
    pub fn as_duration(
        &self,
        resolve_named: &impl Fn(&str) -> Duration,
    ) -> Duration {
        match self {
            Delay::Static(duration) => *duration,
            Delay::Named(name) => resolve_named(name),
        }
    }
}

impl PartialOrd for Delay {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Delay {
    fn cmp(&self, other: &Self) -> Ordering {
        // To be able to compare delays, we need them to be static.
        // Therefore we consider all named delays to be equal and greater than static delays.
        match (self, other) {
            (Delay::Static(a), Delay::Static(b)) => a.cmp(b),
            (Delay::Static(_), Delay::Named(_)) => Ordering::Less,
            (Delay::Named(_), Delay::Static(_)) => Ordering::Greater,
            (Delay::Named(_), Delay::Named(_)) => Ordering::Equal,
        }
    }
}

pub trait StateMachineConfig {
    fn evaluate_condition(condition: &'static str) -> bool;
}

pub trait StateMachine: Sized + Send + 'static {
    type S: Sized + Clone + Debug + Send + Sync;
    type E: Sized + Clone + Debug + Send + 'static;
    type I: Sized + Clone + Debug + Send;
    type C: Sized + Copy + Debug;
    type DT: StateMachineTransition<S = Self::S, E = Self::E> + StateMachineDelay + 'static;
    type T: StateMachineTransition<S = Self::S, E = Self::E> + From<Self::DT> + 'static;

    fn new() -> Self;

    /// Determines the next transition to be executed given the current state and the input.
    /// Returns the transition if a valid one exists, otherwise returns None.
    fn next(&self, input: Self::I, evaluate_condition: impl Fn(Self::C) -> bool)
        -> Option<Self::T>;

    /// Transitions the state machine into the new state
    fn transition(&mut self, state: Self::S);

    /// Returns which delays should be scheduled for the current state
    fn delays(&self) -> Option<Vec<Self::DT>>;

    /// The current state of the state machine
    fn state(&self) -> &Self::S;

    /// Whether the state machine is still in the initial state
    fn started(&self) -> bool;

    /// Whether the state machine has reached a final state
    fn done(&self) -> bool;
}
