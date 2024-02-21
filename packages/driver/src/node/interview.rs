use crate::{
    error::Result, interview_cc, interview_depends_on, CCInterviewContext, Endpoint, EndpointLike,
    Node,
};
use petgraph::{algo, graphmap::DiGraphMap};
use std::fmt::Write;
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
        log.info(|| {
            format!(
                "Beginning interview - current stage: {:?}",
                self.interview_stage(),
            )
        });

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

        if self.supports_cc(CommandClasses::Security2) {
            // TODO
        }

        if self.supports_cc(CommandClasses::Security) {
            // TODO
        }

        if self.supports_cc(CommandClasses::ManufacturerSpecific) {
            interview_cc(CommandClasses::ManufacturerSpecific, &ctx)
                .await
                // FIXME: Handle errors
                .unwrap();
        }

        // FIXME:
        // Basic CC MUST only be used/interviewed when no other actuator CC is supported. If Basic CC is not in the NIF
        // or list of supported CCs, we need to add it here manually, so its version can get queried.

        if self.supports_cc(CommandClasses::Version) {
            interview_cc(CommandClasses::Version, &ctx).await.unwrap();
            // FIXME: Load device config file, apply CC related compat flags
        }

        if self.supports_cc(CommandClasses::WakeUp) {
            // TODO
        }

        // FIXME:
        // Don't offer or interview the Basic CC if any actuator CC is supported - except if the config files forbid us
        // to map the Basic CC to other CCs or expose Basic Set as an event

        // Figure out when to interview which CC.
        //
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

        let priority_ccs: &[CommandClasses] = &[
            CommandClasses::Security2,
            CommandClasses::Security,
            CommandClasses::ManufacturerSpecific,
            CommandClasses::Version,
            CommandClasses::WakeUp,
        ];
        let root_interviews_before_endpoints = determine_interview_order(
            &ctx,
            &[priority_ccs, CommandClasses::application_ccs()].concat(),
        )
        .collect::<Vec<_>>();
        ctx.log.silly(|| {
            format!(
                "Root device interviews before endpoints:{}",
                root_interviews_before_endpoints
                    .iter()
                    .fold(String::new(), |mut acc, cc| {
                        write!(acc, "\n· {}", cc).unwrap();
                        acc
                    })
            )
        });
        let root_interviews_after_endpoints = determine_interview_order(
            &ctx,
            &[priority_ccs, CommandClasses::non_application_ccs()].concat(),
        )
        .collect::<Vec<_>>();
        ctx.log.silly(|| {
            format!(
                "Root device interviews after endpoints:{}",
                root_interviews_after_endpoints
                    .iter()
                    .fold(String::new(), |mut acc, cc| {
                        write!(acc, "\n· {}", cc).unwrap();
                        acc
                    })
            )
        });

        // Interview CCs that should be interviewed before endpoints
        for cc in root_interviews_before_endpoints {
            interview_cc(cc, &ctx).await.unwrap();
        }

        // Interview all endpoints
        let endpoint_indizes: &[u8] = &[];
        for endpoint_index in endpoint_indizes {
            let endpoint = self.get_endpoint(*endpoint_index);
            endpoint.interview_ccs().await?;
        }

        // Interview CCs that should be interviewed after endpoints
        for cc in root_interviews_after_endpoints {
            interview_cc(cc, &ctx).await.unwrap();
        }

        Ok(())
    }
}

impl<'a> Endpoint<'a> {
    async fn interview_ccs(&self) -> Result<()> {
        let ctx = CCInterviewContext {
            driver: self.driver,
            endpoint: self,
            log: self.driver.node_log(self.node_id(), self.index()),
        };

        if self.supports_cc(CommandClasses::Security2) {
            // TODO
        }

        if self.supports_cc(CommandClasses::Security) {
            // TODO
        }

        if self.supports_cc(CommandClasses::Version) {
            interview_cc(CommandClasses::Version, &ctx).await.unwrap();
        }

        // FIXME: Modify supported CCs before further interview - see Z-Wave JS
        let interview_order = determine_interview_order(
            &ctx,
            &[
                CommandClasses::Security2,
                CommandClasses::Security,
                CommandClasses::Version,
            ],
        )
        .collect::<Vec<_>>();
        ctx.log.silly(|| {
            format!(
                "Endpoint {} interviews:{}",
                self.index,
                interview_order.iter().fold(String::new(), |mut acc, cc| {
                    write!(acc, "\n· {}", cc).unwrap();
                    acc
                })
            )
        });

        for cc in interview_order {
            interview_cc(cc, &ctx).await.unwrap();
        }

        Ok(())
    }
}

fn determine_interview_order(
    ctx: &CCInterviewContext<'_>,
    except: &[CommandClasses],
) -> impl Iterator<Item = CommandClasses> {
    let mut graph = DiGraphMap::new();

    let graph_ccs: Vec<_> = ctx
        .endpoint
        .supported_command_classes()
        .into_iter()
        .filter(|cc| !except.contains(cc))
        .collect();

    // Add all supported CCs to the graph, except the ones in the ignore list
    for cc in graph_ccs.iter() {
        graph.add_node(*cc);
    }

    // Now that we have all nodes, determine which CCs depend on which
    for cc in &graph_ccs {
        let Some(deps) = interview_depends_on(*cc, ctx) else {
            continue;
        };
        for dep in deps {
            graph.add_edge(*dep, *cc, 1);
        }
    }

    // Topologically sort the graph
    // FIXME: Do not panic
    let sorted = algo::toposort(&graph, None).expect("CC interview graph is cyclic");

    sorted.into_iter()
}
