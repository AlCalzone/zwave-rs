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

#[cfg(feature = "std")]
mod interview_impl {
    use super::InterviewStage;
    use crate::{error::Result, interview_cc, interview_depends_on, Endpoint, EndpointLike, Node};
    use alloc::{format, string::String, vec::Vec};
    use core::fmt::Write;
    use petgraph::{algo, graphmap::DiGraphMap};
    use zwave_core::definitions::*;

    impl<'a> Node<'a> {
        pub async fn interview(&self) -> Result<()> {
            let log = self.logger();
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
                let node_info = self.driver().request_node_info(&self.id, None).await?;
                for cc in node_info.supported_command_classes {
                    self.modify_cc_info(cc, &PartialCommandClassInfo::default().supported());
                }
                self.set_interview_stage(InterviewStage::CommandClasses);
            }

            if self.interview_stage() == InterviewStage::CommandClasses {
                self.interview_ccs().await?;
            }

            Ok(())
        }

        async fn interview_ccs(&self) -> Result<()> {
            let log = self.logger();

            if self.supports_cc(CommandClasses::Security2) {
                // TODO
            }

            if self.supports_cc(CommandClasses::Security) {
                // TODO
            }

            if self.supports_cc(CommandClasses::ManufacturerSpecific) {
                interview_cc(self, CommandClasses::ManufacturerSpecific)
                    .await
                    .unwrap();
            }

            if self.supports_cc(CommandClasses::Version) {
                interview_cc(self, CommandClasses::Version).await.unwrap();
            }

            if self.supports_cc(CommandClasses::WakeUp) {
                // TODO
            }

            let priority_ccs: &[CommandClasses] = &[
                CommandClasses::Security2,
                CommandClasses::Security,
                CommandClasses::ManufacturerSpecific,
                CommandClasses::Version,
                CommandClasses::WakeUp,
            ];
            let root_interviews_before_endpoints = determine_interview_order(
                self,
                &[priority_ccs, CommandClasses::application_ccs()].concat(),
            )
            .collect::<Vec<_>>();
            log.silly(|| {
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
                self,
                &[priority_ccs, CommandClasses::non_application_ccs()].concat(),
            )
            .collect::<Vec<_>>();
            log.silly(|| {
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

            for cc in root_interviews_before_endpoints {
                interview_cc(self, cc).await.unwrap();
            }

            let endpoint_indizes: &[u8] = &[];
            for endpoint_index in endpoint_indizes {
                let endpoint = self.endpoint(*endpoint_index);
                endpoint.interview_ccs().await?;
            }

            for cc in root_interviews_after_endpoints {
                interview_cc(self, cc).await.unwrap();
            }

            Ok(())
        }
    }

    impl<'a> Endpoint<'a> {
        async fn interview_ccs(&self) -> Result<()> {
            let log = self.logger();

            if self.supports_cc(CommandClasses::Security2) {
                // TODO
            }

            if self.supports_cc(CommandClasses::Security) {
                // TODO
            }

            if self.supports_cc(CommandClasses::Version) {
                interview_cc(self, CommandClasses::Version).await.unwrap();
            }

            let interview_order = determine_interview_order(
                self,
                &[
                    CommandClasses::Security2,
                    CommandClasses::Security,
                    CommandClasses::Version,
                ],
            )
            .collect::<Vec<_>>();
            log.silly(|| {
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
                interview_cc(self, cc).await.unwrap();
            }

            Ok(())
        }
    }

    fn determine_interview_order<'a>(
        endpoint: &'a dyn EndpointLike<'a>,
        except: &[CommandClasses],
    ) -> impl Iterator<Item = CommandClasses> {
        let mut graph = DiGraphMap::new();

        let graph_ccs: Vec<_> = endpoint
            .supported_command_classes()
            .into_iter()
            .filter(|cc| !except.contains(cc))
            .collect();

        for cc in graph_ccs.iter() {
            graph.add_node(*cc);
        }

        for cc in &graph_ccs {
            let Some(deps) = interview_depends_on(endpoint, *cc) else {
                continue;
            };
            for dep in deps {
                graph.add_edge(*dep, *cc, 1);
            }
        }

        let sorted = algo::toposort(&graph, None).expect("CC interview graph is cyclic");

        sorted.into_iter()
    }
}
