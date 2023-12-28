use proc_macros::interview;

use crate::CCInterviewContext;

#[interview(CommandClasses::Basic)]
pub async fn interview_basic_cc(ctx: &CCInterviewContext<'_>) {
    println!(
        "Node {}, {} - Interviewing Basic CC",
        ctx.endpoint.node_id(),
        ctx.endpoint.index()
    );
    // ...
}
