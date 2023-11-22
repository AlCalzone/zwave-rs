use zwave_core::prelude::*;
use zwave_serial::prelude::*;
use zwave_core::state_machine::StateMachine;
use zwave_core::state_machine;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialApiMachineResult {
    Success,
    ACKTimeout,
    CAN,
    NAK,
    ResponseTimeout,
    ResponseNOK,
    CallbackTimeout,
    CallbackNOK,
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
        Response(bool), // OK/NOK
        Callback(bool), // OK/NOK
    },
    Effect = {
        SendFrame,
        AbortSending,
        WaitForACK,
        WaitForResponse,
        WaitForCallback,
    },
    Condition = {
        ExpectsResponse,
        ExpectsCallback,
    },
    Transitions = [
        [Initial => [
            [Start => ! SendFrame => Sending],
        ]],
        [Sending => [
            [FrameSent => ! WaitForACK => WaitingForACK],
        ]],
        [WaitingForACK => [
            [ACK if ExpectsResponse => !WaitForResponse => WaitingForResponse],
            [ACK if ExpectsCallback => !WaitForCallback => WaitingForCallback],
            [ACK => Done(SerialApiMachineResult::Success)],
            [NAK => Done(SerialApiMachineResult::NAK)],
            [CAN => Done(SerialApiMachineResult::CAN)],
            [Timeout => Done(SerialApiMachineResult::ACKTimeout)],
        ]],
        [WaitingForResponse => [
            // TODO: 
            [Response(true) if ExpectsCallback => !WaitForCallback => WaitingForCallback],
            [Response(true) => Done(SerialApiMachineResult::Success)],
            // [Response(true) => WaitForCallback],
            [Response(false)  => Done(SerialApiMachineResult::ResponseNOK)],
        ]],
        [WaitingForCallback => [
            [Callback(true) => Done(SerialApiMachineResult::Success)],
            [Callback(false) => Done(SerialApiMachineResult::CallbackNOK)],
        ]],
    ],
    Delays = [],
    // Delays = [
    //     [WaitForACK => [
    //         [@ACK_TIMEOUT => Done(SerialApiMachineResult::ACKTimeout)],
    //     ]],
    //     [WaitForResponse => [
    //         [@RESPONSE_TIMEOUT => Done(SerialApiMachineResult::ResponseTimeout)],
    //     ]],
    //     [WaitForCallback => [
    //         [@CALLBACK_TIMEOUT => Done(SerialApiMachineResult::CallbackTimeout)],
    //     ]],
    // ],
    Initial = Initial,
    Final = Done(_)
} }
