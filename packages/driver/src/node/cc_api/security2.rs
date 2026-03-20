use crate::{
    CCAPI, CCAPIResult, EndpointLike, expect_cc_or_timeout, handle_unexpected_cc_or_timeout,
};
use zwave_cc::commandclass::{AsDestination, CCAddressable, security2::*};
use zwave_cc::prelude::CC;
use zwave_core::prelude::*;

pub struct Security2CCAPI<'a> {
    endpoint: &'a dyn EndpointLike<'a>,
}

impl<'a> CCAPI<'a> for Security2CCAPI<'a> {
    fn new(endpoint: &'a dyn EndpointLike<'a>) -> Self
    where
        Self: Sized,
    {
        Self { endpoint }
    }

    fn cc_id(&self) -> CommandClasses {
        CommandClasses::Security2
    }

    fn cc_version(&self) -> u8 {
        1
    }

    async fn interview(&self) -> CCAPIResult<()> {
        let endpoint = self.endpoint;
        let node = endpoint.get_node();
        let driver = node.driver();
        let log = endpoint.logger();

        let Some(security_manager) = driver.storage.security_manager2().cloned() else {
            log.warn(|| "no S2 security manager configured, skipping Security 2 interview");
            return Ok(());
        };

        log.warn(|| "interviewing Security 2 CC...");

        // Only on the highest security class does the response include the supported commands.
        let possible_security_classes: &[SecurityClass] = match node.highest_security_class() {
            Some(security_class) if security_class.is_s2() => &[security_class],
            _ if endpoint.index() == EndpointIndex::Root => {
                // If the highest security class is not known yet, query all possible S2 classes
                // on the root device, working from low to high.
                SecurityClass::ALL_S2_ASCENDING
            }
            _ => {
                log.warn(|| {
                    "cannot query securely supported commands for this endpoint without a known S2 security class"
                });
                return Ok(());
            }
        };

        for &security_class in possible_security_classes {
            // We might not know all assigned security classes yet, so we work our way up from low
            // to high and try to request the supported commands. This way each command is
            // encrypted with the security class currently being tested.
            //
            // If the node does not respond, it was not assigned the security class.
            // If it responds with an empty list, the security class is still supported.
            if !security_manager.has_keys_for_security_class(security_class) {
                log.warn(|| {
                    format!(
                        "cannot query securely supported commands for {:?}: network key is not configured",
                        security_class
                    )
                });
                continue;
            }

            log.info(|| {
                format!(
                    "querying securely supported commands for {:?}...",
                    security_class
                )
            });

            let supported_ccs = self.get_supported_commands(security_class).await?;
            match supported_ccs {
                Some(supported_ccs) => {
                    // Any response means the security class is granted, even if the list of
                    // securely supported commands is empty.
                    node.set_security_class(security_class, true);

                    for cc in supported_ccs {
                        let info = if endpoint.supports_cc(cc) {
                            PartialCommandClassInfo::default().secure()
                        } else {
                            PartialCommandClassInfo::default().supported().secure()
                        };
                        endpoint.modify_cc_info(cc, &info);
                    }
                }
                None if endpoint.index() == EndpointIndex::Root => {
                    // No response means the node was not granted the tested security class.
                    node.set_security_class(security_class, false);
                }
                None => {}
            }
        }

        Ok(())
    }

    async fn refresh_values(&self) -> CCAPIResult<()> {
        Ok(())
    }
}

impl Security2CCAPI<'_> {
    /// Sends a nonce to the node, either in response to a `NonceGet` or after a failed
    /// decryption so the SPAN can be re-established.
    pub async fn send_nonce(&self) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let Some(security_manager) = driver.storage.security_manager2().cloned() else {
            return Ok(());
        };

        let receiver_ei = security_manager.generate_nonce(Some(node.id()));
        let cc = Security2CCNonceReport::builder()
            .sos(true)
            .mos(false)
            .receiver_ei(receiver_ei)
            .build()
            .with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    /// Notifies the target node that the MPAN state is out of sync.
    pub async fn send_mos(&self) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = Security2CCNonceReport::builder()
            .sos(false)
            .mos(true)
            .build()
            .with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    /// Sends the current MPAN state to the node.
    pub async fn send_mpan(
        &self,
        group_id: u8,
        inner_mpan_state: zwave_core::security::MpanState,
    ) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = Security2CCMessageEncapsulation::builder()
            .extensions(vec![Security2Extension::mpan(group_id, inner_mpan_state)])
            .build()
            .with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    /// Queries the securely supported commands for the given security class. If the chosen class
    /// does not match the current SPAN state, a fresh nonce exchange may be required first.
    pub async fn get_supported_commands(
        &self,
        security_class: SecurityClass,
    ) -> CCAPIResult<Option<Vec<CommandClasses>>> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let inner = Security2CCCommandsSupportedGet::default();
        // Security2CCCommandsSupportedGet is special because the entire encapsulation stack needs
        // to be built here. The default encapsulation path would choose the node's current
        // security class instead of the one we are probing.
        let cc = Security2CCMessageEncapsulation::builder()
            .security_class(security_class)
            .encapsulated(Box::new(inner.into()))
            .build()
            .with_destination(node.as_destination());
        let response = driver.exec_node_command(&cc.into(), None).await;
        let response = match response {
            Ok(Some(CC::Security2CCMessageEncapsulation(encapsulation))) => {
                match encapsulation.encapsulated.as_deref() {
                    Some(CC::Security2CCCommandsSupportedReport(report)) => {
                        Some(report.supported_ccs.clone())
                    }
                    _ => None,
                }
            }
            Ok(Some(CC::Security2CCNonceReport(_))) => None,
            other => handle_unexpected_cc_or_timeout!(
                other,
                Security2CCMessageEncapsulation,
                Security2CCNonceReport,
            ),
        };

        Ok(response)
    }

    pub async fn report_supported_commands(
        &self,
        supported_ccs: Vec<CommandClasses>,
    ) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = Security2CCCommandsSupportedReport::builder()
            .supported_ccs(supported_ccs)
            .build()
            .with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    pub async fn get_key_exchange_parameters(&self) -> CCAPIResult<Option<Security2CCKEXReport>> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = Security2CCKEXGet::default().with_destination(node.as_destination());
        let response = driver.exec_node_command(&cc.into(), None).await;
        let response = expect_cc_or_timeout!(response, Security2CCKEXReport);

        Ok(response)
    }

    pub async fn request_keys(&self, report: Security2CCKEXReport) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = report.with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    pub async fn grant_keys(&self, set: Security2CCKEXSet) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = set.with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    pub async fn abort_key_exchange(&self, fail_type: KEXFailType) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = Security2CCKEXFail::builder()
            .fail_type(fail_type)
            .build()
            .with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    pub async fn send_public_key(
        &self,
        public_key: Vec<u8>,
        including_node: bool,
    ) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = Security2CCPublicKeyReport::builder()
            .including_node(including_node)
            .public_key(public_key)
            .build()
            .with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    pub async fn request_network_key(&self, requested_key: SecurityClass) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = Security2CCNetworkKeyGet::builder()
            .requested_key(requested_key)
            .build()
            .with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    pub async fn send_network_key(
        &self,
        granted_key: SecurityClass,
        network_key: Vec<u8>,
    ) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = Security2CCNetworkKeyReport::builder()
            .granted_key(granted_key)
            .network_key(network_key)
            .build()
            .with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    pub async fn verify_network_key(&self) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = Security2CCNetworkKeyVerify::default().with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    pub async fn confirm_key_verification(&self) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = Security2CCTransferEnd::builder()
            .key_verified(true)
            .key_request_complete(false)
            .build()
            .with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }

    pub async fn end_key_exchange(&self) -> CCAPIResult<()> {
        let node = self.endpoint.get_node();
        let driver = node.driver();
        let cc = Security2CCTransferEnd::builder()
            .key_verified(false)
            .key_request_complete(true)
            .build()
            .with_destination(node.as_destination());
        driver.exec_node_command(&cc.into(), None).await?;
        Ok(())
    }
}
