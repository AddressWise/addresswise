use crate::service::AddressService;

#[derive(Clone)]
pub struct AppState {
    pub addresses: AddressService,
}
