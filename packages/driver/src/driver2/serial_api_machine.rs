use zwave_core::state_machine;
use zwave_core::state_machine::StateMachine;
use zwave_serial::prelude::*;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq)]
pub enum SerialApiMachineResult {
    Success(Option<Command>),
    ACKTimeout,
    CAN,
    NAK,
    ResponseTimeout,
    ResponseNOK(Command),
    CallbackTimeout,
    CallbackNOK(Command),
}

state_machine! { SerialApiMachine {
    State = {
        Initial,
        Sending,
        WaitingForACK,
        WaitingForResponse,
        WaitingForCallback, // TODO: needs another state for callback aborted
        Done(SerialApiMachineResult),
    },
    Input = {
        Start,
        FrameSent,
        ACK,
        NAK,
        CAN,
        Timeout,
        Response(Command),
        ResponseNOK(Command),
        Callback(Command),
        CallbackNOK(Command),
    },
    Effect = {},
    Condition = {
        ExpectsResponse,
        ExpectsCallback,
    },
    Transitions = [
        [Initial => [
            [Start => Sending],
        ]],
        [Sending => [
            [FrameSent => WaitingForACK],
        ]],
        [WaitingForACK => [
            [ACK if ExpectsResponse => WaitingForResponse],
            [ACK if ExpectsCallback => WaitingForCallback],
            [ACK => Done(SerialApiMachineResult::Success(None))],
            [NAK => Done(SerialApiMachineResult::NAK)],
            [CAN => Done(SerialApiMachineResult::CAN)],
            [Timeout => Done(SerialApiMachineResult::ACKTimeout)],
        ]],
        [WaitingForResponse => [
            // TODO:
            [Response(_) if ExpectsCallback => WaitingForCallback],
            [Response(cmd) => Done(SerialApiMachineResult::Success(Some(cmd)))],
            [ResponseNOK(cmd)  => Done(SerialApiMachineResult::ResponseNOK(cmd))],
            [Timeout => Done(SerialApiMachineResult::ResponseTimeout)]
        ]],
        [WaitingForCallback => [
            [Callback(cmd) => Done(SerialApiMachineResult::Success(Some(cmd)))],
            [CallbackNOK(cmd) => Done(SerialApiMachineResult::CallbackNOK(cmd))],
            [Timeout => Done(SerialApiMachineResult::CallbackTimeout)]
        ]],
    ],
    Delays = [],
    Initial = Initial,
    Final = Done(_)
} }
