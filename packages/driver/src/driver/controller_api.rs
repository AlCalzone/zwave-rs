use crate::{Controller, ControllerStorage, Driver, Ready};

// API access for the controller instance
impl Driver<Ready> {
    pub fn controller(&self) -> Controller {
        Controller::new(self)
    }

    pub(crate) fn get_controller_storage(&self) -> &ControllerStorage {
        &self.state.controller
    }
}
