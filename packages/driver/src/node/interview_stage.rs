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
