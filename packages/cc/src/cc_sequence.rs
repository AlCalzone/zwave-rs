use zwave_core::{definitions::NodeId, security};

use crate::{
    commandclass::{WithAddress, CC},
    prelude::{CCAddressable, CCEncodingContext},
};

/// A sequence of CCs to be transmitted
pub trait CCSequence {
    /// Resets the sequence so it can be iterated over again
    fn reset(&mut self);
    /// Returns the next CC in the sequence
    fn next(&mut self, ctx: &CCEncodingContext) -> Option<WithAddress<CC>>;
    /// Returns whether the sequence is finished
    fn is_finished(&self) -> bool;
    /// Used to pass the response to the previous CC back into the sequence
    fn handle_response(&mut self, response: &CC);
}

pub trait IntoCCSequence {
    fn into_cc_sequence(self) -> Box<dyn CCSequence + Sync + Send>;
}

/// An "empty" sequence for all CCs that don't require sequencing
struct NonSequenced {
    cc: WithAddress<CC>,
    finished: bool,
}

impl CCSequence for NonSequenced {
    fn reset(&mut self) {
        self.finished = false;
    }

    fn next(&mut self, _ctx: &CCEncodingContext) -> Option<WithAddress<CC>> {
        if self.finished {
            return None;
        }
        self.finished = true;

        Some(self.cc.clone())
    }

    fn is_finished(&self) -> bool {
        self.finished
    }

    fn handle_response(&mut self, response: &CC) {
        let _ = response;
        // Nothing to do
    }
}

impl IntoCCSequence for WithAddress<CC> {
    fn into_cc_sequence(self) -> Box<dyn CCSequence + Sync + Send> {
        let address = self.address().clone();
        match self.unwrap() {
            CC::SecurityCCCommandEncapsulation(security_cc) => {
                security_cc.with_address(address).into_cc_sequence()
            }
            cc => Box::new(NonSequenced {
                cc: cc.with_address(address),
                finished: false,
            }),
        }
    }
}
