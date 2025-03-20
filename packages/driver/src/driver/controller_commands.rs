use super::{expect_controller_command_result, ControllerCommandError, ControllerCommandResult, Driver, ExecControllerCommandOptions, ExecutableCommand};
use thiserror::Error;
use typed_builder::TypedBuilder;
use zwave_core::log::Loglevel;
use zwave_core::prelude::*;
use zwave_serial::command::{
    ApplicationUpdateRequest, ApplicationUpdateRequestPayload, Command, CommandBase,
    GetControllerCapabilitiesRequest, GetControllerCapabilitiesResponse, GetControllerIdRequest,
    GetControllerIdResponse, GetControllerVersionRequest, GetControllerVersionResponse,
    GetNodeProtocolInfoRequest, GetProtocolVersionRequest, GetProtocolVersionResponse,
    GetSerialApiCapabilitiesRequest, GetSerialApiCapabilitiesResponse, GetSerialApiInitDataRequest,
    GetSerialApiInitDataResponse, GetSucNodeIdRequest, RequestNodeInfoRequest,
    SerialApiSetupCommand, SerialApiSetupRequest, SerialApiSetupResponsePayload,
    SetSucNodeIdRequest,
};

impl Driver {
    pub async fn get_serial_api_capabilities(
        &self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetSerialApiCapabilitiesResponse> {
        self.controller_log()
            .info(|| "querying Serial API capabilities...");
        let response = self
            .exec_controller_command(GetSerialApiCapabilitiesRequest::default(), options)
            .await;

        let capabilities =
            expect_controller_command_result!(response, GetSerialApiCapabilitiesResponse);

        if self.controller_log().level() < Loglevel::Debug {
            self.controller_log().info(|| {
                LogPayloadText::new("received Serial API capabilities:")
                    .with_nested(capabilities.to_log_payload())
            });
        }

        Ok(capabilities)
    }

    pub async fn get_serial_api_init_data(
        &self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetSerialApiInitDataResponse> {
        self.controller_log()
            .info(|| "querying additional controller information...");
        let response = self
            .exec_controller_command(GetSerialApiInitDataRequest::default(), options)
            .await;

        let init_data = expect_controller_command_result!(response, GetSerialApiInitDataResponse);

        if self.controller_log().level() < Loglevel::Debug {
            self.controller_log().info(|| {
                LogPayloadText::new("received additional controller information:")
                    .with_nested(init_data.to_log_payload())
            });
        }

        Ok(init_data)
    }

    pub async fn get_controller_capabilities(
        &self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetControllerCapabilitiesResponse> {
        self.controller_log()
            .info(|| "querying controller capabilities...");
        let response = self
            .exec_controller_command(GetControllerCapabilitiesRequest::default(), options)
            .await;

        let capabilities =
            expect_controller_command_result!(response, GetControllerCapabilitiesResponse);

        if self.controller_log().level() < Loglevel::Debug {
            self.controller_log().info(|| {
                LogPayloadText::new("received controller capabilities:")
                    .with_nested(capabilities.to_log_payload())
            });
        }

        Ok(capabilities)
    }

    pub async fn get_controller_version(
        &self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetControllerVersionResponse> {
        self.controller_log()
            .info(|| "querying controller version info...");
        let response = self
            .exec_controller_command(GetControllerVersionRequest::default(), options)
            .await;

        let version_info =
            expect_controller_command_result!(response, GetControllerVersionResponse);

        if self.controller_log().level() < Loglevel::Debug {
            self.controller_log().info(|| {
                LogPayloadText::new("received controller version info:")
                    .with_nested(version_info.to_log_payload())
            });
        }

        // FIXME: Store SDK version here too

        Ok(version_info)
    }

    pub async fn get_controller_id(
        &self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetControllerIdResponse> {
        self.controller_log().info(|| "querying controller IDs...");
        let response = self
            .exec_controller_command(GetControllerIdRequest::default(), options)
            .await;

        let ids = expect_controller_command_result!(response, GetControllerIdResponse);

        if self.controller_log().level() < Loglevel::Debug {
            self.controller_log().info(|| {
                LogPayloadText::new("received controller IDs:").with_nested(ids.to_log_payload())
            });
        }

        Ok(ids)
    }

    pub async fn get_protocol_version(
        &self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<GetProtocolVersionResponse> {
        self.controller_log()
            .info(|| "querying protocol version info...");
        let response = self
            .exec_controller_command(GetProtocolVersionRequest::default(), options)
            .await;

        let protocol_version =
            expect_controller_command_result!(response, GetProtocolVersionResponse);

        if self.controller_log().level() < Loglevel::Debug {
            self.controller_log().info(|| {
                LogPayloadText::new("received protocol version info:")
                    .with_nested(protocol_version.to_log_payload())
            });
        }

        // Remember the protocol version
        self.serial_api.storage.set_sdk_version(protocol_version.version);

        Ok(protocol_version)
    }

    pub async fn get_suc_node_id(
        &self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<Option<NodeId>> {
        self.controller_log()
            .info(|| "determining which node is the SUC...");
        let response = self
            .exec_controller_command(GetSucNodeIdRequest::default(), options)
            .await;

        let suc_node_id =
            expect_controller_command_result!(response, GetSucNodeIdResponse).suc_node_id;

        if let Some(suc_node_id) = suc_node_id {
            self.controller_log()
                .info(|| format!("node {} is the SUC", suc_node_id));
        } else {
            self.controller_log().info(|| "no SUC in the network");
        }

        Ok(suc_node_id)
    }

    pub async fn get_supported_serial_api_setup_commands(
        &self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<Vec<SerialApiSetupCommand>> {
        self.controller_log()
            .info(|| "querying supported Serial API setup commands...");
        let response = self
            .exec_controller_command(SerialApiSetupRequest::get_supported_commands(), options)
            .await;
        let response = expect_controller_command_result!(response, SerialApiSetupResponse);

        let ret = expect_serial_api_setup_result!(
            response.payload,
            SerialApiSetupResponsePayload::GetSupportedCommands { commands } => commands
        )?;

        if self.controller_log().level() < Loglevel::Debug {
            self.controller_log().info(|| {
                LogPayloadText::new("supported Serial API setup commands:").with_nested(
                    LogPayloadList::new(ret.iter().map(|cmd| format!("{:?}", cmd).into())),
                )
            });
        }

        Ok(ret)
    }

    pub async fn set_node_id_type(
        &self,
        node_id_type: NodeIdType,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<bool> {
        self.controller_log()
            .info(|| format!("switching serial API to {} node IDs...", node_id_type));
        let response = self
            .exec_controller_command(
                SerialApiSetupRequest::set_node_id_type(node_id_type),
                options,
            )
            .await;
        let response = expect_controller_command_result!(response, SerialApiSetupResponse);

        let success = expect_serial_api_setup_result!(
            response.payload,
            SerialApiSetupResponsePayload::SetNodeIDType { success } => success
        )?;

        self.controller_log().info(|| {
            format!(
                "Switching serial API to {} node IDs {}",
                node_id_type,
                if success { "succeeded" } else { "failed" }
            )
        });

        // Remember the node ID type
        if success {
            self.serial_api.storage.set_node_id_type(node_id_type);
        }

        Ok(success)
    }

    pub async fn get_node_protocol_info(
        &self,
        node_id: &NodeId,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<NodeInformationProtocolData> {
        let log = self.node_log(*node_id, EndpointIndex::Root);
        log.info(|| "querying protocol info...");

        let cmd = GetNodeProtocolInfoRequest { node_id: *node_id };
        let response = self.exec_controller_command(cmd, options).await;
        let response = expect_controller_command_result!(response, GetNodeProtocolInfoResponse);

        if self.controller_log().level() < Loglevel::Debug {
            log.info(|| {
                LogPayloadText::new("received protocol info:")
                    .with_nested(response.to_log_payload())
            });
        }

        Ok(response.protocol_info)
    }

    pub async fn get_rf_region(
        &self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<RfRegion> {
        self.controller_log()
            .info(|| "querying configured RF region...");
        let response = self
            .exec_controller_command(SerialApiSetupRequest::get_rf_region(), options)
            .await;
        let response = expect_controller_command_result!(response, SerialApiSetupResponse);

        let rf_region = expect_serial_api_setup_result!(
            response.payload,
            SerialApiSetupResponsePayload::GetRFRegion { region } => region
        )?;

        // FIXME: Save result when called
        // self.controller().set_rf_region(Some(rf_region));

        self.controller_log()
            .info(|| format!("the controller is using RF region {}", rf_region));

        Ok(rf_region)
    }

    pub async fn get_powerlevel(
        &self,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<Powerlevel> {
        self.controller_log()
            .info(|| "querying configured powerlevel...");
        let response = self
            .exec_controller_command(SerialApiSetupRequest::get_powerlevel(), options)
            .await;
        let response = expect_controller_command_result!(response, SerialApiSetupResponse);

        let powerlevel = expect_serial_api_setup_result!(
            response.payload,
            SerialApiSetupResponsePayload::GetPowerlevel { powerlevel } => powerlevel
        )?;

        // FIXME: Save result when called
        // self.controller().set_powerlevel(Some(powerlevel));

        self.controller_log()
            .info(|| format!("the controller is using powerlevel {}", powerlevel));

        Ok(powerlevel)
    }

    pub async fn set_tx_status_report(
        &self,
        enabled: bool,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<bool> {
        self.controller_log().info(|| {
            format!(
                "{} TX status reports...",
                if enabled { "enabling" } else { "disabling" }
            )
        });
        let response = self
            .exec_controller_command(
                SerialApiSetupRequest::set_tx_status_report(enabled),
                options,
            )
            .await;
        let response = expect_controller_command_result!(response, SerialApiSetupResponse);

        let success = expect_serial_api_setup_result!(
            response.payload,
            SerialApiSetupResponsePayload::SetTxStatusReport { success } => success
        )?;

        // FIXME: use warning/error for failure
        self.controller_log().info(|| {
            format!(
                "{} TX status reports {}",
                if enabled { "enabling" } else { "disabling" },
                if success { "succeeded" } else { "failed" }
            )
        });

        Ok(success)
    }

    pub async fn set_suc_node_id(
        &self,
        own_node_id: NodeId,
        node_id: NodeId,
        enable_suc: bool,
        enable_sis: bool,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<bool> {
        let cmd = SetSucNodeIdRequest::builder()
            .own_node_id(own_node_id)
            .suc_node_id(node_id)
            .enable_suc(enable_suc)
            .enable_sis(enable_sis)
            .build();

        let response = self.exec_controller_command(cmd, options).await;
        let success = match response {
            Ok(Some(Command::SetSucNodeIdResponse(result))) => result.is_ok(),
            Ok(Some(Command::SetSucNodeIdCallback(result))) => result.is_ok(),
            Ok(_) => {
                return Err(ControllerCommandError::Unexpected(
                    "expected SetSucNodeIdResponse or SetSucNodeIdCallback".to_string(),
                ))
            }
            Err(e) => return Err(e.into()),
        };

        // FIXME: Save result when called
        // if success {
        //     self.controller().set_suc_node_id(Some(node_id));
        //     self.controller().set_is_sis(enable_sis);
        //     self.controller().set_is_suc(enable_suc);
        //     if enable_sis {
        //         self.controller().set_sis_present(true);
        //     }
        // }

        Ok(success)
    }

    pub async fn request_node_info(
        &self,
        node_id: &NodeId,
        options: Option<&ExecControllerCommandOptions>,
    ) -> ControllerCommandResult<NodeInformationApplicationData> {
        let log = self.controller_log();

        log.info(|| format!("querying node info for node {}...", node_id));
        let response = self
            .exec_controller_command(RequestNodeInfoRequest::new(*node_id), options)
            .await;

        let application_data = match response {
            Ok(Some(Command::ApplicationUpdateRequest(ApplicationUpdateRequest {
                payload:
                    ApplicationUpdateRequestPayload::NodeInfoReceived {
                        application_data, ..
                    },
                ..
            }))) => {
                log.info(|| format!("Node info received: {:?}", application_data));
                application_data
            }
            Ok(_) => {
                return Err(ControllerCommandError::Unexpected(
                    "expected ApplicationUpdateRequest".to_string(),
                ))
            }
            Err(e) => {
                log.error(|| "querying the node info failed");
                return Err(e.into());
            }
        };

        Ok(application_data)
    }
}

macro_rules! expect_serial_api_setup_result {
    ($actual:expr, $expected:pat => $result:expr) => {
        match $actual {
            $expected => Ok($result),
            SerialApiSetupResponsePayload::Unsupported(cmd) => Err(
                ControllerCommandError::Unsupported(format!("SerialApiSetup::{:?}", cmd)),
            ),
            _ => Err(ControllerCommandError::Unexpected(
                "Unexpected SerialApiSetup response payload".to_string(),
            )),
        }
    };
}
pub(crate) use expect_serial_api_setup_result;
