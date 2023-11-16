use crate::state_machine::traits::*;

use custom_debug_derive::Debug;
use thiserror::Error;

use std::future;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Notify};
use tokio::task::JoinHandle;
use tokio::time::sleep;

type StateMachineInputSender<I> = mpsc::Sender<I>;
type StateMachineInputReceiver<I> = mpsc::Receiver<I>;
type StateMachineEffectSender<E> = broadcast::Sender<E>;
type StateMachineEffectListener<E> = broadcast::Receiver<E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Internal error")]
    Internal,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct StateMachineInterpreter<S, E, I>
where
    I: Sized + Copy,
    E: Sized + Copy + std::fmt::Debug,
    S: Sized + Copy + std::marker::Send,
{
    // Putting the handle into an Option allows us to await it in the `result()` method
    task: Option<JoinHandle<S>>,
    task_shutdown: Arc<Notify>,
    input_tx: StateMachineInputSender<I>,
    effect_rx: StateMachineEffectListener<E>,
}

impl<S, E, I> StateMachineInterpreter<S, E, I>
where
    I: Sized + Copy + std::marker::Send + 'static,
    E: Sized + Copy + std::fmt::Debug + std::marker::Send + 'static,
    S: Sized + Copy + std::marker::Send + 'static,
    // E: Sized + Copy,
    // T: StateMachineTransition<S = S, E = E>,
    // DT: StateMachineDelayedTransition<S = S, E = E> + Sized,
{
    pub fn new<FSM, C>(
        machine: FSM,
        resolve_named: (impl Fn(&str) -> std::time::Duration
             + std::marker::Sync
             + std::marker::Send
             + 'static),
        evaluate_condition: (impl Fn(C) -> bool + std::marker::Sync + std::marker::Send + 'static),
    ) -> Self
    where
        C: Sized + Copy + std::marker::Send + 'static,
        FSM: StateMachine<S = S, E = E, I = I, C = C> + std::marker::Send + 'static,
    {
        // We send inputs to the task using a channel
        let (input_tx, input_rx) = mpsc::channel::<I>(100);
        // We receive effects from the task using a broadcast channel
        let (effect_tx, effect_rx) = broadcast::channel::<E>(100);
        // And we need a way to shut down the task when the Interpreter is dropped
        let task_shutdown = Arc::new(Notify::new());
        let task_shutdown2 = task_shutdown.clone();

        // let machine = Arc::new(Mutex::new(machine));

        // Start the background task for the machine logic
        let task = Some(tokio::spawn(main_loop(
            machine,
            resolve_named,
            evaluate_condition,
            input_rx,
            task_shutdown2,
            effect_tx,
        )));

        Self {
            task,
            task_shutdown,
            input_tx,
            effect_rx,
        }
    }

    pub async fn result(mut self) -> Result<S> {
        if let Some(task) = self.task.take() {
            let result = task.await.map_err(|_| Error::Internal)?;
            Ok(result)
        } else {
            Err(Error::Internal)
        }
    }

    pub fn done(&self) -> bool {
        match &self.task {
            Some(task) => task.is_finished(),
            // If task is None, the `result` method is currently waiting for the task to finish
            // TODO: Is there a better way to do this?
            None => false,
        }
    }

    pub fn effect_listener(&self) -> StateMachineEffectListener<E> {
        self.effect_rx.resubscribe()
    }

    pub fn input_sender(&self) -> StateMachineInputSender<I> {
        self.input_tx.clone()
    }

    pub async fn send(&self, input: I) -> Result<()> {
        send_machine_input(&self.input_tx, input).await
    }
}

impl<S, E, I> Drop for StateMachineInterpreter<S, E, I>
where
    I: Sized + Copy,
    E: Sized + Copy + std::fmt::Debug,
    S: Sized + Copy + std::marker::Send,
{
    fn drop(&mut self) {
        // We need to stop the background task, otherwise it will stick around until the process exits
        self.task_shutdown.notify_one();
    }
}

async fn main_loop<FSM, S, E, I, C>(
    mut machine: FSM,
    resolve_named: impl Fn(&str) -> std::time::Duration,
    evaluate_condition: impl Fn(C) -> bool,
    mut input_rx: StateMachineInputReceiver<I>,
    shutdown: Arc<Notify>,
    effect_tx: StateMachineEffectSender<E>,
) -> S
where
    FSM: StateMachine<S = S, E = E, I = I, C = C>,
    S: Sized + Copy,
    E: Sized + Copy + std::fmt::Debug,
    C: Sized + Copy,
{
    while !machine.done() {
        // If the current state has delays, find the shortest one
        let delayed_transition = match machine.delays() {
            Some(delays) => delays.into_iter().min_by(|a, b| {
                a.delay()
                    .as_duration(&resolve_named)
                    .cmp(&b.delay().as_duration(&resolve_named))
            }),
            _ => None,
        };

        // Optional sleep
        let sleep_duration = delayed_transition.as_ref().map(|d| d.delay());
        let sleep = async {
            match sleep_duration {
                // Sleep for the specified duration
                Some(delay) => sleep(delay.as_duration(&resolve_named)).await,
                // "sleep" forever
                None => future::pending::<()>().await,
            }
        };
        tokio::pin!(sleep);

        // Wait for something to do
        tokio::select! {
            // We were told to stop
            _ = shutdown.notified() => {
                // Exit the task
                break;
            }

            // The shortest delay expired
            _ = &mut sleep => {
                let delay = delayed_transition.as_ref().unwrap();
                let new_state = delay.new_state();
                if let Some(effect) = delay.effect() {
                    effect_tx.send(effect).unwrap();
                }
                machine.transition(new_state);
            }

            // An input was received
            Some(input) = input_rx.recv() => {

                if let Some(transition) = machine.next(input, &evaluate_condition) {
                    let new_state = transition.new_state();
                    if let Some(effect) = transition.effect() {
                        effect_tx.send(effect).unwrap();
                    }
                    machine.transition(new_state);
                }
            }
        }
    }
    *machine.state()
}

async fn send_machine_input<I: Sized + Copy>(
    input_sender: &StateMachineInputSender<I>,
    input: I,
) -> Result<()> {
    input_sender.send(input).await.map_err(|_| Error::Internal)
}

#[cfg(test)]
pub(crate) mod test {
    use tokio::sync::broadcast::error::RecvError;

    use crate::state_machine::StateMachine;

    use super::StateMachineInterpreter;
    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

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

        let resolve_named = |name: &str| match name {
            "Lazy" => Duration::from_millis(1000),
            _ => Duration::from_millis(0),
        };

        let evaluate_condition = |_| false;

        let interpreter = StateMachineInterpreter::new(fsm, resolve_named, evaluate_condition);
        let sender = interpreter.input_sender();
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
        assert_eq!(final_state, FSMState::Done);
    }

    #[tokio::test]
    async fn test_interpreter_delay() {
        let fsm = FSM::default();

        let resolve_named = |name: &str| match name {
            "Lazy" => Duration::from_millis(1000),
            _ => Duration::from_millis(0),
        };
        let evaluate_condition = |_| false;

        let interpreter = StateMachineInterpreter::new(fsm, resolve_named, evaluate_condition);
        let sender = interpreter.input_sender();
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
