use crate::{
    ControllerCommandError, ControllerCommandResult, Driver, EndpointStorage, NodeStorage,
};
use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};
use zwave_core::{definitions::*, log::Loglevel, submodule};
use zwave_serial::command::SerialApiSetupCommand;

submodule!(storage);
submodule!(node_api);
// submodule!(node_commands);

/// The controller API can be in one of multiple states, each of which has a different set of capabilities.
pub trait ControllerState {}

/// The controller isn't fully initialized yet
pub struct Init;
impl ControllerState for Init {}

/// The controller is ready to use normally
#[derive(Clone)]
pub struct Ready {
    storage: Arc<RwLock<ControllerStorage>>,
    nodes: Arc<RwLock<BTreeMap<NodeId, NodeStorage>>>,
    endpoints: Arc<RwLock<BTreeMap<(NodeId, EndpointIndex), EndpointStorage>>>,
}
impl ControllerState for Ready {}

// #[derive(Debug)]
pub struct Controller<'a, S: ControllerState> {
    driver: &'a Driver,
    state: S,
}

impl<'a> Controller<'a, Init> {
    pub fn new(driver: &'a Driver) -> Self {
        Self {
            driver,
            state: Init,
        }
    }

    pub async fn interview(&self) -> ControllerCommandResult<Controller<'a, Ready>> {
        let driver = self.driver;

        // // We execute some of these commands before knowing the controller capabilities, so
        // // we disable enforcing that the controller supports the commands.
        // let command_options = ExecControllerCommandOptions::builder()
        //     .enforce_support(false)
        //     .build();
        // let command_options: Option<&ExecControllerCommandOptions> = Some(&command_options);

        let command_options = None;

        // TODO: Log results
        let api_capabilities = driver.get_serial_api_capabilities(command_options).await?;
        let init_data = driver.get_serial_api_init_data(command_options).await?;
        let version_info = driver.get_controller_version(command_options).await?;
        let capabilities = driver.get_controller_capabilities(command_options).await?;

        // GetProtocolVersion includes the patch version, GetControllerVersion does not.
        // We prefer having this information, so query it if supported.
        let protocol_version = if api_capabilities
            .supported_function_types
            .contains(&FunctionType::GetProtocolVersion)
        {
            driver.get_protocol_version(command_options).await?.version
        } else {
            parse_libary_version(&version_info.library_version).map_err(|e| {
                ControllerCommandError::Unexpected(format!("Failed to parse library version: {e}"))
            })?
        };

        let supported_serial_api_setup_commands = if api_capabilities
            .supported_function_types
            .contains(&FunctionType::SerialApiSetup)
        {
            driver
                .get_supported_serial_api_setup_commands(command_options)
                .await?
        } else {
            vec![]
        };

        // Switch to 16 bit node IDs if supported. We need to do this here, as a controller may still be
        // in 16 bit mode when Z-Wave starts up. This would lead to an invalid node ID being reported.
        if supported_serial_api_setup_commands.contains(&SerialApiSetupCommand::SetNodeIDType) {
            let _ = driver
                .set_node_id_type(NodeIdType::NodeId16Bit, command_options)
                .await;
        }

        // Afterwards, execute the commands that parse node IDs
        let ids = driver.get_controller_id(command_options).await?;
        let suc_node_id = driver.get_suc_node_id(command_options).await?;

        let mut nodes = BTreeMap::new();
        let endpoints = BTreeMap::new();

        // Read the protocol info for each node and store it
        // FIXME: Read this from cache where possible when we have one
        {
            for node_id in &init_data.node_ids {
                let protocol_info = driver
                    .get_node_protocol_info(node_id, command_options)
                    .await?;
                let storage = NodeStorage::new(protocol_info);
                nodes.insert(*node_id, storage);
            }
        }

        // Not the most logical spot to do this, but now we have everything we need to initialize
        // the security managers
        driver.init_security_managers();

        let controller = ControllerStorage::builder()
            .home_id(ids.home_id)
            .own_node_id(ids.own_node_id)
            .suc_node_id(suc_node_id)
            .fingerprint(DeviceFingerprint::new(
                api_capabilities.manufacturer_id,
                api_capabilities.product_type,
                api_capabilities.product_id,
                api_capabilities.firmware_version,
            ))
            .library_type(version_info.library_type)
            .api_version(init_data.api_version)
            .protocol_version(protocol_version)
            .sdk_version(protocol_version_to_sdk_version(&protocol_version))
            .node_type(init_data.node_type)
            .role(capabilities.role)
            .started_this_network(capabilities.started_this_network)
            .sis_present(capabilities.sis_present)
            .is_sis(init_data.is_sis)
            .is_suc(capabilities.is_suc)
            .supported_function_types(api_capabilities.supported_function_types)
            .supported_serial_api_setup_commands(supported_serial_api_setup_commands)
            .supports_timers(init_data.supports_timers)
            .build();

        Ok(Controller {
            driver,
            state: Ready {
                storage: Arc::new(RwLock::new(controller)),
                nodes: Arc::new(RwLock::new(nodes)),
                endpoints: Arc::new(RwLock::new(endpoints)),
            },
        })
    }
}

impl Controller<'_, Ready> {
    pub(crate) async fn configure(&self) -> ControllerCommandResult<()> {
        let driver = self.driver;

        // Get the currently configured RF region and remember it.
        // If it differs from the desired region, change it afterwards.
        if self.supports_serial_api_setup_command(SerialApiSetupCommand::GetRFRegion) {
            let _region = driver.get_rf_region(None).await?;
            // FIXME: set region if desired
        }

        // Get the currently configured powerlevel and remember it.
        // If it differs from the desired powerlevel, change it afterwards.
        if self.supports_serial_api_setup_command(SerialApiSetupCommand::GetPowerlevel) {
            let _powerlevel = driver.get_powerlevel(None).await?;
            // FIXME: set powerlevel if desired
        }

        // Enable TX status reports if supported
        if self.supports_serial_api_setup_command(SerialApiSetupCommand::SetTxStatusReport) {
            driver.set_tx_status_report(true, None).await?;
        }

        // There needs to be a SUC/SIS in the network.
        // If not, we promote ourselves to SUC if all of the following conditions are met:
        // * We are the primary controller
        // * but we are not SUC
        // * there is no SUC and
        // * there is no SIS
        let should_promote = {
            self.role() == ControllerRole::Primary
                && !self.is_suc()
                && !self.is_sis()
                && self.suc_node_id().is_none()
        };

        if should_promote {
            driver
                .controller_log()
                .info(|| "there is no SUC/SIS in the network - promoting ourselves...");
            let own_node_id = self.own_node_id();
            match driver
                .set_suc_node_id(own_node_id, own_node_id, true, true, None)
                .await
            {
                Ok(success) => {
                    driver.controller_log().message(
                        || {
                            format!(
                                "Promotion to SUC/SIS {}",
                                if success { "succeeded" } else { "failed" }
                            )
                        },
                        if success {
                            Loglevel::Info
                        } else {
                            Loglevel::Warn
                        },
                    );
                }
                Err(e) => {
                    driver
                        .controller_log()
                        .error(|| format!("error while promoting to SUC/SIS: {:?}", e));
                }
            }
        } else {
            driver
                .controller_log()
                .info(|| "there is a SUC/SIS in the network - not promoting ourselves");
        }

        Ok(())
    }
}

impl<'a> Controller<'a, Ready> {
    pub fn driver(&self) -> &Driver {
        self.driver
    }

    fn storage(&self) -> RwLockReadGuard<'_, ControllerStorage> {
        self.state
            .storage
            .read()
            .expect("failed to lock controller storage for reading")
    }

    fn storage_mut(&self) -> RwLockWriteGuard<'_, ControllerStorage> {
        self.state
            .storage
            .write()
            .expect("failed to lock controller storage for writing")
    }

    pub(crate) fn node_storage(&self) -> RwLockReadGuard<'_, BTreeMap<NodeId, NodeStorage>> {
        self.state
            .nodes
            .read()
            .expect("failed to lock node storage for reading")
    }

    pub(crate) fn node_storage_mut(&self) -> RwLockWriteGuard<'_, BTreeMap<NodeId, NodeStorage>> {
        self.state
            .nodes
            .write()
            .expect("failed to lock node storage for writing")
    }

    pub(crate) fn endpoint_storage(
        &self,
    ) -> RwLockReadGuard<'_, BTreeMap<(NodeId, EndpointIndex), EndpointStorage>> {
        self.state
            .endpoints
            .read()
            .expect("failed to lock endpoint storage for reading")
    }

    pub(crate) fn endpoint_storage_mut(
        &self,
    ) -> RwLockWriteGuard<'_, BTreeMap<(NodeId, EndpointIndex), EndpointStorage>> {
        self.state
            .endpoints
            .write()
            .expect("failed to lock endpoint storage for writing")
    }

    /// Checks whether a given Z-Wave function type is supported by the controller.
    pub fn supports_function(&self, function_type: FunctionType) -> bool {
        self.storage()
            .supported_function_types
            .contains(&function_type)
    }

    /// Checks whether a given Z-Wave Serial API setup command is supported by the controller.
    pub fn supports_serial_api_setup_command(&self, command: SerialApiSetupCommand) -> bool {
        self.storage()
            .supported_serial_api_setup_commands
            .contains(&command)
    }

    pub fn home_id(&self) -> Id32 {
        self.storage().home_id
    }

    pub fn own_node_id(&self) -> NodeId {
        self.storage().own_node_id
    }

    pub fn suc_node_id(&self) -> Option<NodeId> {
        self.storage().suc_node_id
    }

    pub(crate) fn set_suc_node_id(&mut self, suc_node_id: Option<NodeId>) {
        self.storage_mut().suc_node_id = suc_node_id;
    }

    pub fn is_suc(&self) -> bool {
        self.storage().is_suc
    }

    pub(crate) fn set_is_suc(&mut self, is_suc: bool) {
        self.storage_mut().is_suc = is_suc;
    }

    pub fn is_sis(&self) -> bool {
        self.storage().is_sis
    }

    pub(crate) fn set_is_sis(&mut self, is_sis: bool) {
        self.storage_mut().is_sis = is_sis;
    }

    pub fn sis_present(&self) -> bool {
        self.storage().sis_present
    }

    pub(crate) fn set_sis_present(&mut self, sis_present: bool) {
        self.storage_mut().sis_present = sis_present;
    }

    pub fn role(&self) -> ControllerRole {
        self.storage().role
    }

    pub(crate) fn set_role(&mut self, role: ControllerRole) {
        self.storage_mut().role = role;
    }

    pub fn rf_region(&self) -> Option<RfRegion> {
        self.storage().rf_region
    }

    pub(crate) fn set_rf_region(&mut self, region: Option<RfRegion>) {
        self.storage_mut().rf_region = region;
    }

    pub fn powerlevel(&self) -> Option<Powerlevel> {
        self.storage().powerlevel
    }

    pub(crate) fn set_powerlevel(&mut self, powerlevel: Option<Powerlevel>) {
        self.storage_mut().powerlevel = powerlevel;
    }
}

impl Clone for Controller<'_, Ready> {
    fn clone(&self) -> Self {
        Self {
            driver: self.driver,
            state: self.state.clone(),
        }
    }
}
