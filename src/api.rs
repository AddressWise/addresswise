use ntex::web;

use crate::error::AppError;
use crate::models::{AutocompleteResponse, ResolveAddressRequest, ResolveAddressResponse};
use crate::state::AppState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(resolve_address);
    cfg.service(autocomplete);
}

#[web::post("/resolve-address")]
async fn resolve_address(
    state: web::types::State<AppState>,
    payload: web::types::Json<ResolveAddressRequest>,
) -> Result<web::types::Json<ResolveAddressResponse>, AppError> {
    let response = state.addresses.resolve(payload.into_inner()).await?;
    Ok(web::types::Json(response))
}

#[web::post("/autocomplete")]
async fn autocomplete() -> Result<web::types::Json<AutocompleteResponse>, AppError> {
    Err(AppError::not_implemented(
        "autocomplete is not implemented yet",
    ))
}
