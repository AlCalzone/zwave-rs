use crate::InterviewStage;
use std::sync::RwLock;
use zwave_core::prelude::*;

#[derive(Debug)]
/// Internal storage for a node instance. Since this is meant be used from both library and external
/// (application) code, in several locations at once, often simultaneously, we need to use
/// interior mutability to allow for concurrent access without requiring a mutable reference.
pub(crate) struct NodeStorage {
    pub(crate) interview_stage: RwLock<InterviewStage>,
}

impl NodeStorage {
    pub fn new() -> Self {
        Self {
            interview_stage: RwLock::new(InterviewStage::None),
        }
    }
}
