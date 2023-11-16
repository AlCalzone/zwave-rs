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
        WaitForACK,
        WaitForResponse,
        WaitForCallback,
        Done(SerialApiMachineResult),
    },
    Input = {
        Start,
        FrameSent,
        ACK,
        NAK,
        CAN,
        Response(bool), // OK/NOK
        Callback(bool), // OK/NOK
    },
    Effect = {
        SendFrame,
        AbortSendData,
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
            [FrameSent => WaitForACK],
        ]],
        [WaitForACK => [
            [ACK if ExpectsResponse => WaitForResponse],
            [ACK if ExpectsCallback => WaitForCallback],
            [ACK => Done(SerialApiMachineResult::Success)],
            // [ACK => WaitForResponse],
            [NAK => Done(SerialApiMachineResult::NAK)],
            [CAN => Done(SerialApiMachineResult::CAN)],
            // TODO:
        ]],
        [WaitForResponse => [
            // TODO: 
            [Response(true) if ExpectsCallback => WaitForCallback],
            [Response(true) => Done(SerialApiMachineResult::Success)],
            // [Response(true) => WaitForCallback],
            [Response(false)  => Done(SerialApiMachineResult::ResponseNOK)],
        ]],
        [WaitForCallback => [
            [Callback(true) => Done(SerialApiMachineResult::Success)],
            [Callback(false) => Done(SerialApiMachineResult::CallbackNOK)],
        ]],
    ],
    Delays = [
        [WaitForACK => [
            [@ACK_TIMEOUT => Done(SerialApiMachineResult::ACKTimeout)],
        ]],
        [WaitForResponse => [
            [@RESPONSE_TIMEOUT => Done(SerialApiMachineResult::ResponseTimeout)],
        ]],
        [WaitForCallback => [
            [@CALLBACK_TIMEOUT => Done(SerialApiMachineResult::CallbackTimeout)],
        ]],
    ],
    Initial = Initial,
    Final = Done(_)
} }
