use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct ResolveAddressRequest {
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub address: Option<StructuredAddressInput>,
    #[serde(default, alias = "houseNumber")]
    pub house_number: Option<String>,
    #[serde(default, alias = "thoroughfare")]
    pub street: Option<String>,
    #[serde(default, alias = "locality")]
    pub city: Option<String>,
    #[serde(default, alias = "postalCode")]
    pub postal_code: Option<String>,
    #[serde(default, alias = "country_code", alias = "countryCode")]
    pub country: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct StructuredAddressInput {
    #[serde(default, alias = "houseNumber")]
    pub house_number: Option<String>,
    #[serde(default, alias = "thoroughfare")]
    pub street: Option<String>,
    #[serde(default, alias = "locality")]
    pub city: Option<String>,
    #[serde(default, alias = "postalCode")]
    pub postal_code: Option<String>,
    #[serde(default, alias = "country_code", alias = "countryCode")]
    pub country: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResolveAddressResponse {
    pub query: String,
    pub score: f64,
    pub address: Address,
    pub diagnostics: MatchDiagnostics,
}

#[derive(Debug, Serialize)]
pub struct Address {
    pub id: i64,
    pub formatted: String,
    pub country_code: String,
    pub admin_area: Option<String>,
    pub locality: Option<String>,
    pub dependent_locality: Option<String>,
    pub street: Option<String>,
    pub house_number: Option<String>,
    pub house_number_type: Option<String>,
    pub unit: Option<String>,
    pub postal_code: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct MatchDiagnostics {
    pub trigram_score: f64,
    pub levenshtein_distance: Option<i32>,
}
