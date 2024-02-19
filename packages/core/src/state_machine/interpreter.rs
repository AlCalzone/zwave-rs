use crate::state_machine::traits::*;
use crate::util::{now, MaybeSleep};

use custom_debug_derive::Debug;
use thiserror::Error;

use std::marker::{Send, Sync};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, Notify};
use tokio::task::JoinHandle;

type StateMachineInputSender<I> = mpsc::Sender<I>;
type StateMachineInputReceiver<I> = mpsc::Receiver<I>;
type StateMachineEffectSender<E> = broadcast::Sender<E>;
type StateMachineEffectListener<E> = broadcast::Receiver<E>;
type StateMachineTransitionSender<T> = broadcast::Sender<T>;
type StateMachineTransitionListener<T> = broadcast::Receiver<T>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal error")]
    Internal,
}

pub type Result<T> = std::result::Result<T, Error>;

/// An asynchronous interpreter for finite state machines. Supports automatic delayed transitions.
pub struct StateMachineInterpreter<FSM>
where
    FSM: StateMachine,
{
    // Putting the handles into an Option allows us to await it in the `result()` method
    main_task: Option<JoinHandle<FSM::S>>,
    main_task_shutdown: Arc<Notify>,

    #[allow(dead_code)]
    state_task: Option<JoinHandle<()>>,
    state_task_shutdown: Arc<Notify>,

    input_tx: StateMachineInputSender<FSM::I>,
    effect_rx: StateMachineEffectListener<FSM::E>,

    transition_rx: StateMachineTransitionListener<FSM::T>,
    current_state: Arc<std::sync::RwLock<FSM::S>>,
}

impl<FSM> StateMachineInterpreter<FSM>
where
    FSM: StateMachine,
{
    pub fn new(
        machine: FSM,
        resolve_named: Box<dyn Fn(&str) -> Duration + Sync + Send>,
        evaluate_condition: Box<dyn Fn(FSM::C) -> bool + Sync + Send>,
    ) -> Self {
        // We send inputs to the main task using a channel
        let (input_tx, input_rx) = mpsc::channel::<FSM::I>(100);
        // We receive effects from the task using a broadcast channel
        let (effect_tx, effect_rx) = broadcast::channel::<FSM::E>(100);
        // We receive updates of the current state using a broadcast channel
        let (transition_tx, transition_rx) = broadcast::channel::<FSM::T>(100);

        // And we need a way to shut down the tasks when the Interpreter is dropped
        let main_task_shutdown = Arc::new(Notify::new());
        let main_task_shutdown2 = main_task_shutdown.clone();
        let state_task_shutdown = Arc::new(Notify::new());
        let state_task_shutdown2 = state_task_shutdown.clone();

        let current_state = (*machine.state()).clone();
        let current_state = Arc::new(std::sync::RwLock::new(current_state));
        let current_state2 = current_state.clone();

        let mut transition_rx2 = transition_tx.subscribe();
        // Start the background task that listens for state changes
        let state_task = Some(tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = state_task_shutdown2.notified() => {
                        break;
                    }
                    Ok(transition) = transition_rx2.recv() => {
                        *current_state2.write().unwrap() = transition.new_state();
                    }
                }
            }
        }));

        // Start the background task for the machine logic
        let main_task = Some(tokio::spawn(main_loop(
            machine,
            resolve_named,
            evaluate_condition,
            input_rx,
            main_task_shutdown2,
            effect_tx,
            transition_tx,
        )));

        Self {
            main_task,
            main_task_shutdown,
            state_task,
            state_task_shutdown,
            input_tx,
            effect_rx,
            transition_rx,
            current_state,
        }
    }

    pub async fn result(mut self) -> Result<FSM::S> {
        if let Some(task) = self.main_task.take() {
            task.await.map_err(|_| Error::Internal)
        } else {
            Err(Error::Internal)
        }
        // No need to take care of the state_task here. Calling `result` will drop this interpreter,
        // causing the state_task to be shut down.
    }

    pub fn done(&self) -> bool {
        match &self.main_task {
            Some(task) => task.is_finished(),
            // If task is None, the `result` method is currently waiting for the task to finish
            // TODO: Is there a better way to do this?
            None => false,
        }
    }

    pub fn state(&self) -> FSM::S {
        (*self.current_state).read().unwrap().clone()
    }

    pub fn effect_listener(&self) -> StateMachineEffectListener<FSM::E> {
        self.effect_rx.resubscribe()
    }

    pub fn transition_listener(&self) -> StateMachineTransitionListener<FSM::T> {
        self.transition_rx.resubscribe()
    }

    pub fn input_sender(&self) -> StateMachineInputSender<FSM::I> {
        self.input_tx.clone()
    }

    pub async fn send(&self, input: FSM::I) -> Result<()> {
        send_machine_input(&self.input_tx, input).await
    }
}

impl<FSM> Drop for StateMachineInterpreter<FSM>
where
    FSM: StateMachine,
{
    fn drop(&mut self) {
        // We need to stop the background tasks, otherwise they will stick around until the process exits
        self.main_task_shutdown.notify_one();
        self.state_task_shutdown.notify_one();
    }
}

async fn main_loop<FSM>(
    mut machine: FSM,
    resolve_named: impl Fn(&str) -> Duration,
    evaluate_condition: impl Fn(FSM::C) -> bool,
    mut input_rx: StateMachineInputReceiver<FSM::I>,
    shutdown: Arc<Notify>,
    effect_tx: StateMachineEffectSender<FSM::E>,
    transition_tx: StateMachineTransitionSender<FSM::T>,
) -> FSM::S
where
    FSM: StateMachine,
{
    while !machine.done() {
        // If the current state has delays, find the shortest one
        let mut delayed_transition = match machine.delays() {
            Some(delays) => delays.into_iter().min_by(|a, b| {
                a.delay()
                    .as_duration(&resolve_named)
                    .cmp(&b.delay().as_duration(&resolve_named))
            }),
            _ => None,
        };

        // Optional sleep
        let sleep_duration = delayed_transition
            .as_ref()
            .map(|d| d.delay().as_duration(&resolve_named));
        let sleep = MaybeSleep::new(sleep_duration);

        // Wait for something to do
        tokio::select! {
            // We were told to stop
            _ = shutdown.notified() => {
                // Exit the task
                break;
            }

            // The shortest delay expired
            _ = sleep => {
                let delay = delayed_transition.take().unwrap();
                let new_state = delay.new_state();
                if let Some(effect) = delay.effect() {
                    effect_tx.send(effect).unwrap();
                }
                machine.transition(new_state);
                // After transitioning, inform the outside world
                transition_tx.send(delay.into()).unwrap();
            }

            // An input was received
            Some(input) = input_rx.recv() => {
                println!("{} FSM received input: {:?}", now(), input);
                if let Some(transition) = machine.next(input, &evaluate_condition) {
                    let new_state = transition.new_state();
                    if let Some(effect) = transition.effect() {
                        effect_tx.send(effect).unwrap();
                    }
                    machine.transition(new_state);
                    // After transitioning, inform the outside world
                    transition_tx.send(transition).unwrap();
                }
            }
        }
    }
    (*machine.state()).clone()
}

async fn send_machine_input<I>(input_sender: &StateMachineInputSender<I>, input: I) -> Result<()> {
    input_sender.send(input).await.map_err(|_| Error::Internal)
}

#[cfg(test)]
pub(crate) mod test {
    #![allow(clippy::upper_case_acronyms, unused_variables, unused_imports)]
    use tokio::sync::broadcast::error::RecvError;

    use crate::state_machine::StateMachine;

    use super::StateMachineInterpreter;
    use std::time::Duration;

    state_machine! { FSM {
        State = {
            Initial,
            Working,
            Done,
        },
        Input = {
            DoStuff,
            FinishStuff
        },
        Effect = {
            DoingStuff,
            FinishedStuff,
        },
        Condition = {},
        Transitions = [
            [Initial => [
                [DoStuff => ! DoingStuff => Working]
            ]],
            [Working => [
                [FinishStuff => ! FinishedStuff => Done]
            ]]
        ],
        Delays = [
            [Working => [
                [@Lazy => Done],
                [Duration::from_millis(10000) => Working]
            ]]
        ],
        Initial = Initial,
        Final = Done
    } }

    #[tokio::test]
    async fn test_interpreter_nodelay() {
        let fsm = FSM::default();

        let resolve_named = Box::new(|name: &str| match name {
            "Lazy" => Duration::from_millis(1000),
            _ => Duration::from_millis(0),
        });

        let evaluate_condition = Box::new(|_| false);

        let interpreter = StateMachineInterpreter::new(fsm, resolve_named, evaluate_condition);
        let mut listener = interpreter.effect_listener();

        let mut inputs: Vec<FSMInput> = vec![FSMInput::DoStuff, FSMInput::FinishStuff];

        let listen_task = tokio::spawn(async move {
            let mut effects: Vec<FSMEffect> = Vec::new();
            loop {
                match listener.recv().await {
                    Ok(effect) => {
                        effects.push(effect);
                    }
                    Err(RecvError::Closed) => {
                        break;
                    }
                    Err(RecvError::Lagged(_)) => {}
                }
            }
            effects
        });

        while !inputs.is_empty() {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = interpreter.send(inputs.remove(0)).await;
        }

        let effects = listen_task.await.unwrap();
        assert_eq!(
            effects,
            vec![FSMEffect::DoingStuff, FSMEffect::FinishedStuff]
        );

        let final_state = interpreter.result().await.unwrap();
        assert!(matches!(final_state, FSMState::Done));
    }

    #[tokio::test]
    async fn test_interpreter_delay() {
        let fsm = FSM::default();

        let resolve_named = Box::new(|name: &str| match name {
            "Lazy" => Duration::from_millis(1000),
            _ => Duration::from_millis(0),
        });
        let evaluate_condition = Box::new(|_| false);

        let interpreter = StateMachineInterpreter::new(fsm, resolve_named, evaluate_condition);
        let mut listener = interpreter.effect_listener();

        // We only provide an input to go into the working state
        // Then the delay takes over
        let mut inputs: Vec<FSMInput> = vec![FSMInput::DoStuff];

        let listen_task = tokio::spawn(async move {
            let mut effects: Vec<FSMEffect> = Vec::new();
            loop {
                match listener.recv().await {
                    Ok(effect) => effects.push(effect),
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => {}
                }
            }
            effects
        });

        while !inputs.is_empty() {
            tokio::time::sleep(Duration::from_millis(50)).await;
            let _ = interpreter.send(inputs.remove(0)).await;
        }

        // No more inputs, wait 150 ms for the timeout to occur
        tokio::time::sleep(Duration::from_millis(150)).await;

        let effects = listen_task.await.unwrap();
        assert!(interpreter.done());
        // We didn't get the FinishedStuff event because the timeout includes none
        assert_eq!(effects, vec![FSMEffect::DoingStuff]);

        let final_state = interpreter.result().await.unwrap();
        assert_eq!(final_state, FSMState::Done);
    }
}
