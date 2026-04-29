use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct AutocompleteResponse {
    pub session_id: String,
    pub query: String,
    pub country_bias: Option<String>,
    pub suggestions: Vec<AutocompleteSuggestion>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AutocompleteRequest {
    #[serde(alias = "sessionId")]
    pub session_id: String,
    pub query: String,
    #[serde(default, alias = "countryBias")]
    pub country_bias: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct AutocompleteSuggestion {
    pub id: i64,
    pub formatted: String,
    pub country_code: String,
    pub street: String,
    pub locality: Option<String>,
    pub postal_code: Option<String>,
}
