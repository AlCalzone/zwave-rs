use crate::{
    expect_cc_or_timeout, get_implemented_version, CCAPIResult, CCInterviewContext, EndpointLike,
    CCAPI,
};
use zwave_cc::commandclass::{
    CCAddressable, VersionCCCapabilitiesGet, VersionCCCapabilitiesReport, VersionCCCommandClassGet,
    VersionCCGet, VersionCCReport, VersionCCValues, VersionCCZWaveSoftwareGet,
    VersionCCZWaveSoftwareReport,
};
use zwave_core::{cache::CacheExt, prelude::*};

pub struct VersionCCAPI<'a> {
    endpoint: &'a dyn EndpointLike<'a>,
}

impl<'a> CCAPI<'a> for VersionCCAPI<'a> {
    fn new(endpoint: &'a dyn EndpointLike<'a>) -> Self
    where
        Self: Sized,
    {
        Self { endpoint }
    }

    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Version
    }

    fn cc_version(&self) -> u8 {
        3
    }

    fn interview_depends_on(&self) -> &'static [CommandClasses] {
        &[CommandClasses::ManufacturerSpecific]
    }

    async fn interview<'ctx>(&self, ctx: &CCInterviewContext<'ctx>) -> CCAPIResult<()> {
        let endpoint = self.endpoint;
        let node = endpoint.get_node();
        let cache = node.value_cache();

        // FIXME: Use the root endpoint for all queries

        // In a Multi Channel device, the Version Command Class MUST be supported by the Root Device, while
        // the Version Command Class SHOULD NOT be supported by individual End Points.
        //
        // There may be cases where a given Command Class is not implemented by the Root Device of a Multi
        // Channel device. However, the Root Device MUST respond to Version requests for any Command Class
        // implemented by the Multi Channel device; also in cases where the actual Command Class is only
        // provided by an End Point.

        println!(
            "Node {}, {} - Interviewing Version CC",
            ctx.endpoint.node_id(),
            ctx.endpoint.index(),
        );

        // On the root endpoint, query the VersionCC version and static version information
        if endpoint.index() == EndpointIndex::Root {
            self.query_cc_version(CommandClasses::Version, ctx).await?;
            // TODO: When we use CC versions to check support for features,
            // we might have to update the version after this call

            println!("Querying node versions...");
            if let Some(response) = self.get().await? {
                println!("received response for node versions: {:?}", response);
            }
        }

        // Query all other CC versions
        for cc in endpoint.supported_command_classes() {
            // Skip Version CC itself which we already queried
            if cc == CommandClasses::Version {
                continue;
            }
            // Skip the query of endpoint CCs that are also supported by the root device
            if endpoint.index() > EndpointIndex::Root && node.get_cc_version(cc) > Some(0) {
                continue;
            }

            self.query_cc_version(cc, ctx).await?;
        }

        // On the root device, query Version CC capabilities
        if endpoint.index() == EndpointIndex::Root
            && node.get_cc_version(CommandClasses::Version) >= Some(3)
        {
            println!("Querying Version CC capabilities...");
            if let Some(response) = self.get_capabilities().await? {
                println!(
                    "received Version CC capabilities capabilities: {:?}",
                    response
                );

                if cache.read_bool(&VersionCCValues::supports_zwave_software_get().id) == Some(true)
                {
                    println!("Querying Z-Wave software version...");
                    if let Some(response) = self.get_zwave_software().await? {
                        println!("received Z-Wave software version: {:?}", response);
                    }
                }
            }
        }

        Ok(())
    }

    async fn refresh_values(&self) -> CCAPIResult<()> {
        // Nothing that requires refreshing
        Ok(())
    }
}

impl VersionCCAPI<'_> {
    async fn query_cc_version<'ctx>(
        &self,
        cc: CommandClasses,
        ctx: &CCInterviewContext<'ctx>,
    ) -> CCAPIResult<()> {
        if get_implemented_version(cc).is_none() {
            println!("Skipping query for not yet implemented CC {}", cc);
            return Ok(());
        }

        println!("Querying version for CC {}", cc);
        if let Some(version) = self.get_cc_version(cc).await? {
            println!("received version for CC {}: {}", cc, version);
            if version > 0 {
                println!("... supports CC {} in version {}", cc, version);
                self.endpoint
                    .modify_cc_info(cc, &PartialCommandClassInfo::default().version(version))
            } else {
                // We were lied to - the NIF said this CC is supported, now the node claims it isn't
                // Make sure this is not a critical CC, which must be supported though
                // FIXME: Actually check if the CC is critical and save version 1 to the cache

                let is_critical = false;
                if is_critical {
                    todo!()
                } else {
                    println!("... does not support CC {}", cc);
                    self.endpoint.remove_cc(cc);
                }
            }
        } else {
            println!("CC version query for {} timed out. Assuming version 1", cc);
            self.endpoint
                .modify_cc_info(cc, &PartialCommandClassInfo::default().version(1))
        }

        Ok(())
    }
}

impl VersionCCAPI<'_> {
    // FIXME: Implement a way to query support for API commands

    pub async fn get(&self) -> CCAPIResult<Option<VersionCCReport>> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = VersionCCGet::default().with_destination(node.id().into());
        let response = driver.exec_node_command(&cc, None).await;
        let response = expect_cc_or_timeout!(response, VersionCCReport);

        Ok(response)
    }

    pub async fn get_cc_version(&self, cc: CommandClasses) -> CCAPIResult<Option<u8>> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = VersionCCCommandClassGet::builder()
            .requested_cc(cc)
            .build()
            .with_destination(node.id().into());
        let response = driver.exec_node_command(&cc, None).await;
        let response = expect_cc_or_timeout!(response, VersionCCCommandClassReport);

        Ok(response.map(|r| r.version))
    }

    pub async fn get_capabilities(&self) -> CCAPIResult<Option<VersionCCCapabilitiesReport>> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = VersionCCCapabilitiesGet::default().with_destination(node.id().into());
        let response = driver.exec_node_command(&cc, None).await;
        let response = expect_cc_or_timeout!(response, VersionCCCapabilitiesReport);

        Ok(response)
    }

    pub async fn get_zwave_software(&self) -> CCAPIResult<Option<VersionCCZWaveSoftwareReport>> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = VersionCCZWaveSoftwareGet::default().with_destination(node.id().into());
        let response = driver.exec_node_command(&cc, None).await;
        let response = expect_cc_or_timeout!(response, VersionCCZWaveSoftwareReport);

        Ok(response)
    }
}
