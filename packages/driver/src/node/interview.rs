use crate::{error::Result, interview_cc, CCInterviewContext, EndpointLike, Node};
use zwave_core::definitions::*;

/// Specifies the progress of the interview process for a node
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InterviewStage {
    /// The interview process hasn't started yet
    None,

    /// Querying the node's capabilities from the node itself, including supported/controlled command classes
    NodeInfo,

    /// Interviewing all command classes supported by the node
    CommandClasses, // FIXME: Add granularity to show progress

    /// The interview process has finished
    Done,
}

impl<'a> Node<'a> {
    pub async fn interview(&self) -> Result<()> {
        let log = self.driver.node_log(self.id(), EndpointIndex::Root);
        log.info(format!(
            "Beginning interview - current stage: {:?}",
            self.interview_stage(),
        ));

        if self.interview_stage() == InterviewStage::None {
            self.set_interview_stage(InterviewStage::NodeInfo);
        }

        if self.interview_stage() == InterviewStage::NodeInfo {
            // Query the node info and save supported CCs
            let node_info = self.driver.request_node_info(&self.id, None).await?;
            for cc in node_info.supported_command_classes {
                self.modify_cc_info(cc, &PartialCommandClassInfo::default().supported());
            }

            // Done, advance to the next stage
            self.set_interview_stage(InterviewStage::CommandClasses);
        }

        if self.interview_stage() == InterviewStage::CommandClasses {
            self.interview_ccs().await?;
        }

        Ok(())
    }

    async fn interview_ccs(&self) -> Result<()> {
        let ctx = CCInterviewContext {
            driver: self.driver,
            endpoint: self,
            log: self.driver.node_log(self.node_id(), EndpointIndex::Root),
        };

        // FIXME: Correct the order of interviews
        for cc in self.supported_command_classes() {
            interview_cc(cc, &ctx).await.unwrap();
        }

        // Desired order:
        // Root endpoint:
        // * Security S2
        // * Security S0
        // * Manufacturer Specific ✅
        // * Version ✅
        // * Wake Up
        // * ...other non-application CCs
        // Endpoints:
        // * Security S2
        // * Security S0
        // * Version
        // * ... other CCs
        // Root endpoint:
        // * ... all application CCs

        Ok(())
    }
}
